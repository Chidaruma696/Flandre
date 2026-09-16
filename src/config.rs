//! `~/.config/flandre/config.toml`

use crate::util;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Variant {
    TonalSpot,
    Vibrant,
    Expressive,
    FruitSalad,
    Rainbow,
    Neutral,
    Monochrome,
    Fidelity,
    Content,
}

/// How far the wallpaper colour is pushed into neutral surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Tint {
    /// Material defaults: barely tinted greys.
    Soft,
    /// Noticeably tinted surfaces.
    Normal,
    /// Everything neutral takes the wallpaper hue; headerbars and sidebars get a visible wash.
    Strong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Targets {
    pub gtk: bool,
    pub shell: bool,
    pub icons: bool,
    pub ptyxis: bool,
    pub console: bool,
    pub blackbox: bool,
    pub terminals: bool,
}

impl Default for Targets {
    fn default() -> Self {
        Self {
            gtk: true,
            shell: true,
            icons: true,
            ptyxis: true,
            console: true,
            blackbox: true,
            terminals: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Icons {
    /// Icon theme family to recolour (needs `<base>` and `<base>-dark`/`<base>-light` installed).
    pub base: String,
    /// Colour the recoloured icons are painted with when the family uses a different accent.
    pub source_accent: String,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            base: "Tela".into(),
            source_accent: "#5294e2".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub variant: Variant,
    pub tint: Tint,
    /// Extra shell command run after every apply (like Material You's `extra-command`).
    pub extra_command: String,
    pub notify: bool,
    pub targets: Targets,
    pub icons: Icons,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            variant: Variant::FruitSalad,
            tint: Tint::Strong,
            extra_command: String::new(),
            notify: true,
            targets: Targets::default(),
            icons: Icons::default(),
        }
    }
}

pub fn path() -> PathBuf {
    util::flandre_config_dir().join("config.toml")
}

impl Config {
    pub fn load() -> Result<Self> {
        let p = path();
        if !p.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", p.display()))
    }

    pub fn write_default_if_missing() -> Result<bool> {
        let p = path();
        if p.exists() {
            return Ok(false);
        }
        let text = format!(
            "# Flandre configuration. Every key is optional.\n\
             # variant: tonal-spot | vibrant | expressive | fruit-salad | rainbow | neutral | monochrome | fidelity | content\n\
             # tint: soft | normal | strong\n\n{}",
            toml::to_string_pretty(&Self::default())?
        );
        util::write_atomic(&p, text.as_bytes())?;
        Ok(true)
    }
}
