//! One full pass: wallpaper -> scheme -> every enabled target.

use crate::colors::{self, Palette, hex};
use crate::config::{Config, Tint, Variant};
use crate::targets;
use crate::util;
use anyhow::{Context, Result, anyhow};
use material_colors::color::Argb;
use std::path::{Path, PathBuf};

const BG_SCHEMA: &str = "org.gnome.desktop.background";
const IFACE_SCHEMA: &str = "org.gnome.desktop.interface";

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub wallpaper: Option<PathBuf>,
    pub color: Option<String>,
    pub variant: Option<Variant>,
    pub tint: Option<Tint>,
    pub dark: Option<bool>,
    pub force: bool,
    pub quiet: bool,
}

pub fn prefers_dark() -> bool {
    util::get_string(IFACE_SCHEMA, "color-scheme").is_some_and(|s| s == "prefer-dark")
}

pub fn current_wallpaper(dark: bool) -> Result<PathBuf> {
    let key = if dark { "picture-uri-dark" } else { "picture-uri" };
    let uri = util::get_string(BG_SCHEMA, key).ok_or_else(|| anyhow!("schema {BG_SCHEMA} not available"))?;
    let uri = if uri.is_empty() && dark {
        util::get_string(BG_SCHEMA, "picture-uri").unwrap_or_default()
    } else {
        uri
    };
    let path = util::uri_to_path(&uri).ok_or_else(|| anyhow!("wallpaper is not a file URI: {uri}"))?;
    if !path.is_file() {
        return Err(anyhow!("wallpaper file does not exist: {}", path.display()));
    }
    Ok(path)
}

fn stamp_path() -> PathBuf {
    util::flandre_cache_dir().join("last-apply")
}

fn stamp(source: Argb, dark: bool, cfg: &Config, wallpaper: Option<&Path>) -> String {
    let mtime = wallpaper
        .and_then(|w| std::fs::metadata(w).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!(
        "{} {} {:?} {:?} {} {} {:?} {:?} {:?} {} {} {:?} {}",
        hex(source),
        dark,
        cfg.variant,
        cfg.tint,
        cfg.darken,
        cfg.headerbar,
        cfg.icons,
        cfg.shell,
        cfg.terminals,
        wallpaper.map(|w| w.display().to_string()).unwrap_or_default(),
        mtime,
        cfg.targets_key(),
        env!("CARGO_PKG_VERSION")
    )
}

impl Config {
    fn targets_key(&self) -> [bool; 7] {
        let t = &self.targets;
        [t.gtk, t.shell, t.icons, t.ptyxis, t.console, t.blackbox, t.terminals]
    }
}

pub struct Report {
    pub lines: Vec<String>,
    pub warnings: Vec<String>,
    pub skipped: bool,
}

pub fn run(cfg: &Config, opts: &Options) -> Result<Report> {
    let mut cfg = cfg.clone();
    if let Some(v) = opts.variant {
        cfg.variant = v;
    }
    if let Some(t) = opts.tint {
        cfg.tint = t;
    }
    let dark = opts.dark.unwrap_or_else(prefers_dark);
    let mut lines = Vec::new();
    let mut warnings = Vec::new();

    let (source, wallpaper) = if let Some(c) = &opts.color {
        (colors::parse_hex(c)?, None)
    } else {
        let w = match &opts.wallpaper {
            Some(w) => w.clone(),
            None => current_wallpaper(dark)?,
        };
        let src = colors::source_from_image(&w)?;
        (src, Some(w))
    };
    let st = stamp(source, dark, &cfg, wallpaper.as_deref());
    if !opts.force && std::fs::read_to_string(stamp_path()).map(|s| s == st).unwrap_or(false) {
        return Ok(Report {
            lines: vec!["nothing changed since the last apply (use --force)".into()],
            warnings,
            skipped: true,
        });
    }
    lines.push(format!(
        "source {} from {} ({} / {:?} / {:?})",
        hex(source),
        wallpaper
            .as_ref()
            .map(|w| w.display().to_string())
            .unwrap_or_else(|| "colour".into()),
        if dark { "dark" } else { "light" },
        cfg.variant,
        cfg.tint
    ));

    let theme = colors::build_theme(source, cfg.variant);
    let pal_dark = Palette::build(&theme, true, &cfg);
    let pal_light = Palette::build(&theme, false, &cfg);
    let p = if dark { &pal_dark } else { &pal_light };
    lines.push(format!(
        "primary {}  surface {}  headerbar {}  terminal {} on {}",
        hex(p.primary),
        hex(p.surface),
        hex(p.headerbar_bg),
        hex(p.term_fg),
        hex(p.term_bg)
    ));

    if cfg.targets.gtk {
        match targets::gtk::apply(p, &pal_dark, &pal_light) {
            Ok(files) => lines.push(format!("gtk: {}", files.join(", "))),
            Err(e) => warnings.push(format!("gtk: {e:#}")),
        }
    }
    if cfg.targets.shell {
        match targets::shell::apply(p, &cfg.shell) {
            Ok(f) => lines.push(format!("shell: {}", f.display())),
            Err(e) => warnings.push(format!("shell: {e:#}")),
        }
    }
    if cfg.targets.icons {
        match targets::icons::apply(p, &cfg.icons) {
            Ok(r) => lines.push(format!(
                "icons: {} ({} SVGs recoloured, {})",
                r.name,
                r.recolored,
                r.dir.display()
            )),
            Err(e) => warnings.push(format!("icons: {e:#}")),
        }
    }
    if cfg.targets.ptyxis {
        match targets::ptyxis::apply(&pal_dark, &pal_light, cfg.terminals.opacity) {
            Ok(r) if r.files.is_empty() => lines.push("ptyxis: not installed".into()),
            Ok(r) => lines.push(format!(
                "ptyxis: {} ({} profile(s) switched)",
                r.files
                    .iter()
                    .map(|f| f.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                r.profiles
            )),
            Err(e) => warnings.push(format!("ptyxis: {e:#}")),
        }
    }
    if cfg.targets.console {
        match targets::console::apply(&pal_dark, &pal_light, cfg.terminals.opacity) {
            Ok(targets::console::ConsoleResult::NotInstalled) => lines.push("console: not installed".into()),
            Ok(targets::console::ConsoleResult::Host) => {
                lines.push("console: livery written to org.gnome.Console".into())
            }
            Ok(targets::console::ConsoleResult::Flatpak) => lines.push("console: livery written to the Flatpak".into()),
            Err(e) => warnings.push(format!("console: {e:#}")),
        }
    }
    if cfg.targets.blackbox {
        match targets::blackbox::apply(&pal_dark, &pal_light, cfg.terminals.opacity) {
            Ok(r) if r.files.is_empty() => lines.push("blackbox: not installed".into()),
            Ok(r) => lines.push(format!(
                "blackbox: {}",
                r.files
                    .iter()
                    .map(|f| f.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Err(e) => warnings.push(format!("blackbox: {e:#}")),
        }
    }
    if cfg.targets.terminals {
        match targets::osc::apply(p, true) {
            Ok(r) => lines.push(format!(
                "terminals: {} open pty(s) repainted, {}",
                r.ptys,
                r.files[0].display()
            )),
            Err(e) => warnings.push(format!("terminals: {e:#}")),
        }
    }
    if !cfg.extra_command.trim().is_empty() {
        let ok = std::process::Command::new("bash")
            .arg("-c")
            .arg(&cfg.extra_command)
            .env("FLANDRE_PRIMARY", hex(p.primary))
            .env("FLANDRE_MODE", if dark { "dark" } else { "light" })
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            lines.push("extra command: ok".into());
        } else {
            warnings.push("extra command failed".into());
        }
    }

    util::write_atomic(&stamp_path(), st.as_bytes()).context("writing apply stamp")?;
    if cfg.notify && !opts.quiet {
        util::notify(
            "Flandre",
            &format!("Theme regenerated from the wallpaper (accent {})", hex(p.primary)),
        );
    }
    Ok(Report {
        lines,
        warnings,
        skipped: false,
    })
}
