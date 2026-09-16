//! GNOME Shell: recolours the stylesheet shipped by the *installed* gnome-shell (read from its
//! GResource), so it never lags behind the Shell version, and loads it through User Themes.

use crate::colors::{Palette, hex};
use crate::util;
use anyhow::{Context, Result, anyhow};
use gio::prelude::*;
use material_colors::color::Argb;
use std::path::PathBuf;

pub const THEME_NAME: &str = "Flandre";
const RESOURCE: &str = "/usr/share/gnome-shell/gnome-shell-theme.gresource";
const USER_THEME_SCHEMA: &str = "org.gnome.shell.extensions.user-theme";
pub const USER_THEME_UUID: &str = "user-theme@gnome-shell-extensions.gcampax.github.com";

pub fn theme_dir() -> PathBuf {
    util::data_home().join("themes").join(THEME_NAME).join("gnome-shell")
}

fn stock_css(dark: bool) -> Result<String> {
    let res = gio::Resource::load(RESOURCE).with_context(|| format!("loading {RESOURCE}"))?;
    let name = if dark {
        "gnome-shell-dark.css"
    } else {
        "gnome-shell-light.css"
    };
    let path = format!("/org/gnome/shell/theme/{name}");
    let data = res
        .lookup_data(&path, gio::ResourceLookupFlags::NONE)
        .or_else(|_| res.lookup_data("/org/gnome/shell/theme/gnome-shell.css", gio::ResourceLookupFlags::NONE))
        .with_context(|| format!("stylesheet {path} not found in {RESOURCE}"))?;
    Ok(String::from_utf8_lossy(&data).into_owned())
}

fn is_hex_digit(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

/// Rewrites every colour literal in the stylesheet through `Palette::recolor`, and the accent
/// keywords through the primary colour.
pub fn recolor_css(css: &str, p: &Palette) -> String {
    let bytes = css.as_bytes();
    let mut out = String::with_capacity(css.len() + 1024);
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'#' {
            let mut j = i + 1;
            while j < bytes.len() && is_hex_digit(bytes[j]) {
                j += 1;
            }
            let len = j - i - 1;
            let boundary =
                j >= bytes.len() || !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-');
            if (len == 3 || len == 6)
                && boundary
                && let Ok(c) = crate::colors::parse_hex(&css[i..j])
            {
                out.push_str(&hex(p.recolor(c)));
                i = j;
                continue;
            }
            out.push('#');
            i += 1;
            continue;
        }
        if (css[i..].starts_with("rgba(") || css[i..].starts_with("rgb("))
            && let Some(end) = css[i..].find(')')
        {
            let inner = &css[i..i + end + 1];
            let open = inner.find('(').unwrap();
            let parts: Vec<&str> = inner[open + 1..inner.len() - 1].split(',').map(str::trim).collect();
            if parts.len() >= 3 {
                let comp = |s: &str| s.parse::<f64>().ok().map(|v| v.round().clamp(0.0, 255.0) as u8);
                if let (Some(r), Some(g), Some(b)) = (comp(parts[0]), comp(parts[1]), comp(parts[2])) {
                    let c = p.recolor(Argb::new(255, r, g, b));
                    if parts.len() == 4 {
                        out.push_str(&format!("rgba({}, {}, {}, {})", c.red, c.green, c.blue, parts[3]));
                    } else {
                        out.push_str(&format!("rgb({}, {}, {})", c.red, c.green, c.blue));
                    }
                    i += end + 1;
                    continue;
                }
            }
        }
        // copy one UTF-8 char
        let ch = css[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    // Accent keywords last, so the primary is not harmonised a second time.
    out.replace("-st-accent-fg-color", &hex(p.on_primary))
        .replace("-st-accent-color", &hex(p.primary))
}

pub fn extra_css(p: &Palette) -> String {
    format!(
        "\n/* Flandre extras */\n\
.popup-menu-item:checked, .popup-menu-item:active, .popup-menu-item.selected {{ background-color: {sel}; }}\n\
.quick-toggle:checked, .quick-menu-toggle:checked {{ background-color: {primary}; color: {on_primary}; }}\n\
StScrollBar StButton#vhandle, StScrollBar StButton#hhandle {{ background-color: {outline}; }}\n\
.workspace-thumbnail-indicator, .workspace-indicator .active {{ border-color: {primary}; }}\n\
.calendar .calendar-today, .calendar .calendar-today:hover, .calendar .calendar-today:focus {{ background-color: {primary}; color: {on_primary}; }}\n\
.notification-banner, .message {{ background-color: {popover}; }}\n\
.search-entry:focus {{ border-color: {primary}; }}\n\
.osd-window {{ background-color: {popover}; }}\n\
.login-dialog, .unlock-dialog {{ background-color: {surface}; }}\n",
        sel = hex_alpha(p.primary, if p.dark { 0.32 } else { 0.24 }),
        primary = hex(p.primary),
        on_primary = hex(p.on_primary),
        outline = hex(p.outline),
        popover = hex(p.popover_bg),
        surface = hex(p.surface),
    )
}

fn hex_alpha(c: Argb, a: f64) -> String {
    format!("rgba({}, {}, {}, {a})", c.red, c.green, c.blue)
}

pub fn apply(p: &Palette) -> Result<PathBuf> {
    let stock = stock_css(p.dark)?;
    let mut css = String::from("/* Generated by Flandre from the installed gnome-shell stylesheet */\n");
    css.push_str(&recolor_css(&stock, p));
    css.push_str(&extra_css(p));
    let dir = theme_dir();
    let file = dir.join("gnome-shell.css");
    util::write_atomic(&file, css.as_bytes())?;
    reload()?;
    Ok(file)
}

/// User Themes reloads when the setting changes, so toggle it.
pub fn reload() -> Result<()> {
    let s = util::settings(USER_THEME_SCHEMA)
        .ok_or_else(|| anyhow!("User Themes schema not installed (package gnome-shell-extensions)"))?;
    if s.string("name") == THEME_NAME {
        s.set_string("name", "").map_err(|e| anyhow!("{e}"))?;
        gio::Settings::sync();
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
    s.set_string("name", THEME_NAME).map_err(|e| anyhow!("{e}"))?;
    gio::Settings::sync();
    Ok(())
}

pub fn user_theme_enabled() -> bool {
    util::run("gnome-extensions", &["info", USER_THEME_UUID])
        .map(|o| crate::setup::ext_state_is(&o, "ACTIVE") || crate::setup::ext_state_is(&o, "ENABLED"))
        .unwrap_or(false)
}
