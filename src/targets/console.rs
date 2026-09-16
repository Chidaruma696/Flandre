//! GNOME Console (kgx): a custom "livery" stored in `org.gnome.Console custom-liveries`
//! (a{sv}: uuid -> {uuid, name, night, day}) and selected through the `livery` key.

use crate::colors::{Palette, rgb_tuple};
use crate::util;
use anyhow::{Result, anyhow};
use gio::prelude::*;
use glib::{Variant, VariantDict};

pub const LIVERY_UUID: &str = "f1a4d2e0-5c0d-4e1a-9d0b-f1a4d2e00001";
const SCHEMA: &str = "org.gnome.Console";
const FLATPAK_ID: &str = "org.gnome.Console";

fn palette_variant(p: &Palette) -> Variant {
    let d = VariantDict::new(None);
    d.insert("foreground", rgb_tuple(p.term_fg));
    d.insert("background", rgb_tuple(p.term_bg));
    let colours: Vec<(f64, f64, f64)> = p.ansi.iter().map(|c| rgb_tuple(*c)).collect();
    d.insert("colours", colours);
    d.insert("transparency", 0.0f64);
    d.end()
}

pub fn livery_variant(dark: &Palette, light: &Palette) -> Variant {
    let d = VariantDict::new(None);
    d.insert("uuid", LIVERY_UUID);
    d.insert("name", "Flandre");
    d.insert_value("night", &palette_variant(dark).to_variant());
    d.insert_value("day", &palette_variant(light).to_variant());
    d.end()
}

pub enum ConsoleResult {
    NotInstalled,
    Host,
    Flatpak,
}

pub fn apply(dark: &Palette, light: &Palette) -> Result<ConsoleResult> {
    let livery = livery_variant(dark, light);
    if let Some(s) = util::settings(SCHEMA) {
        if !s
            .settings_schema()
            .map(|sc| sc.has_key("custom-liveries"))
            .unwrap_or(false)
        {
            return Err(anyhow!(
                "this GNOME Console has no custom liveries (needs Console 49 or newer)"
            ));
        }
        // keep other custom liveries
        let existing = s.value("custom-liveries");
        let merged = VariantDict::new(None);
        if let Some(iter) = existing.iter().into() {
            for entry in iter {
                if let Some((k, v)) = entry.get::<(String, Variant)>()
                    && k != LIVERY_UUID
                {
                    merged.insert_value(&k, &v);
                }
            }
        }
        merged.insert_value(LIVERY_UUID, &livery.to_variant());
        let value = merged.end();
        s.set_value("custom-liveries", &value)
            .map_err(|e| anyhow!("custom-liveries: {e}"))?;
        s.set_string("livery", LIVERY_UUID)
            .map_err(|e| anyhow!("livery: {e}"))?;
        gio::Settings::sync();
        return Ok(ConsoleResult::Host);
    }
    if util::flatpak_app_installed(FLATPAK_ID) {
        let text = livery.print(true);
        // Wrap as a{sv} with a single entry; other liveries in the Flatpak are not merged.
        let dict = format!("{{'{LIVERY_UUID}': <{text}>}}");
        util::run(
            "flatpak",
            &[
                "run",
                "--command=gsettings",
                FLATPAK_ID,
                "set",
                SCHEMA,
                "custom-liveries",
                &dict,
            ],
        )?;
        util::run(
            "flatpak",
            &[
                "run",
                "--command=gsettings",
                FLATPAK_ID,
                "set",
                SCHEMA,
                "livery",
                LIVERY_UUID,
            ],
        )?;
        return Ok(ConsoleResult::Flatpak);
    }
    Ok(ConsoleResult::NotInstalled)
}
