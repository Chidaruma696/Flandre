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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum IconFamily {
    /// Tela if installed, else Papirus.
    Auto,
    Tela,
    Papirus,
}

/// Which colour of the scheme the icons are painted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum AccentRole {
    /// Material You's own choice for folders: deep in dark mode, pastel in light mode.
    PrimaryContainer,
    Primary,
    Secondary,
    Tertiary,
    /// The primary hue at the tone chosen in `icons.tone`.
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Icons {
    pub family: IconFamily,
    pub accent: AccentRole,
    /// Tone (0 = black, 100 = white) used when `accent = "custom"`.
    pub tone: f64,
    /// Minimum chroma (colourfulness) for the icon colour; 0 keeps the scheme's own.
    pub chroma: f64,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            family: IconFamily::Auto,
            accent: AccentRole::PrimaryContainer,
            tone: 45.0,
            chroma: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PanelStyle {
    /// Stock GNOME: black bar.
    Black,
    /// Painted with the tinted surface colour.
    Colored,
    Transparent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Shell {
    pub panel: PanelStyle,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            panel: PanelStyle::Black,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub variant: Variant,
    pub tint: Tint,
    /// 0.0 = Material's own tones, 1.0 = as dark as it goes (dark mode only).
    pub darken: f64,
    /// Extra shell command run after every apply (like Material You's `extra-command`).
    pub extra_command: String,
    pub notify: bool,
    pub targets: Targets,
    pub icons: Icons,
    pub shell: Shell,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            variant: Variant::FruitSalad,
            tint: Tint::Strong,
            darken: 0.5,
            extra_command: String::new(),
            notify: true,
            targets: Targets::default(),
            icons: Icons::default(),
            shell: Shell::default(),
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

    pub fn save(&self) -> Result<()> {
        let text = format!(
            "# Flandre configuration. Every key is optional; `flandre settings` edits this file.\n\
             # variant: tonal-spot | vibrant | expressive | fruit-salad | rainbow | neutral | monochrome | fidelity | content\n\
             # tint: soft | normal | strong    darken: 0.0-1.0\n\
             # icons.family: auto | tela | papirus    icons.accent: primary-container | primary | secondary | tertiary | custom\n\
             # shell.panel: black | colored | transparent\n\n{}",
            toml::to_string_pretty(self)?
        );
        util::write_atomic(&path(), text.as_bytes())
    }

    pub fn write_default_if_missing() -> Result<bool> {
        if path().exists() {
            return Ok(false);
        }
        Self::default().save()?;
        Ok(true)
    }
}
