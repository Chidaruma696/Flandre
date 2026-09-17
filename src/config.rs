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

/// Base colour scheme for terminals. Everything but `Flandre` is a well-known palette
/// (`src/schemes.rs`) that is then pulled towards the wallpaper by `terminals.blend`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TermScheme {
    /// Built entirely from the Material scheme: blue = primary, magenta = tertiary,
    /// red = error; green, yellow and cyan are fixed hues harmonised with the wallpaper.
    Flandre,
    Gnome,
    Tango,
    Solarized,
    Monokai,
    Gruvbox,
    Dracula,
    Nord,
    Catppuccin,
    TokyoNight,
    Everforest,
    RosePine,
    Ayu,
    Kanagawa,
}

impl TermScheme {
    pub const ALL: [TermScheme; 14] = [
        TermScheme::Flandre,
        TermScheme::Gnome,
        TermScheme::Tango,
        TermScheme::Solarized,
        TermScheme::Monokai,
        TermScheme::Gruvbox,
        TermScheme::Dracula,
        TermScheme::Nord,
        TermScheme::Catppuccin,
        TermScheme::TokyoNight,
        TermScheme::Everforest,
        TermScheme::RosePine,
        TermScheme::Ayu,
        TermScheme::Kanagawa,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TermScheme::Flandre => "Flandre",
            TermScheme::Gnome => "GNOME",
            TermScheme::Tango => "Tango",
            TermScheme::Solarized => "Solarized",
            TermScheme::Monokai => "Monokai",
            TermScheme::Gruvbox => "Gruvbox",
            TermScheme::Dracula => "Dracula",
            TermScheme::Nord => "Nord",
            TermScheme::Catppuccin => "Catppuccin",
            TermScheme::TokyoNight => "Tokyo Night",
            TermScheme::Everforest => "Everforest",
            TermScheme::RosePine => "Rosé Pine",
            TermScheme::Ayu => "Ayu",
            TermScheme::Kanagawa => "Kanagawa",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Terminals {
    /// Background opacity for Ptyxis, Console and Black Box: 0.0 = see-through, 1.0 = solid.
    pub opacity: f64,
    /// Base palette for the 16 ANSI colours, foreground and background.
    pub scheme: TermScheme,
    /// How far a classic scheme is pulled towards the wallpaper: 0.0 = as published,
    /// 1.0 = fully harmonised. Ignored by `flandre`, which is already built from it.
    pub blend: f64,
}

impl Default for Terminals {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            scheme: TermScheme::Flandre,
            blend: 0.5,
        }
    }
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
    /// Opacity of the top bar colour (black or coloured), 0.0 = see-through, 1.0 = solid.
    pub panel_opacity: f64,
    /// Background of the Shell's menus (calendar, quick settings, popovers, dialogs, dash):
    /// 0.0 = Adwaita's greys, 1.0 = near black in dark mode / a dim grey in light mode.
    pub menus: f64,
    /// Which colour of the scheme highlights the Shell: checked quick toggles, today in the
    /// calendar, selected items, slider fills.
    pub accent: AccentRole,
    /// Tone (0 = black, 100 = white) used when `accent = "custom"`.
    pub accent_tone: f64,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            panel: PanelStyle::Black,
            panel_opacity: 1.0,
            menus: 0.0,
            accent: AccentRole::Primary,
            accent_tone: 60.0,
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
    /// Window decoration (headerbar) darkness: 0.0 = Adwaita's, a notch lighter than the window,
    /// 1.0 = near black in dark mode, a dim grey in light mode.
    pub headerbar: f64,
    /// Extra shell command run after every apply (like Material You's `extra-command`).
    pub extra_command: String,
    pub notify: bool,
    pub targets: Targets,
    pub icons: Icons,
    pub shell: Shell,
    pub terminals: Terminals,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            variant: Variant::FruitSalad,
            tint: Tint::Strong,
            darken: 0.5,
            headerbar: 0.0,
            extra_command: String::new(),
            notify: true,
            targets: Targets::default(),
            icons: Icons::default(),
            shell: Shell::default(),
            terminals: Terminals::default(),
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
             # tint: soft | normal | strong    darken / headerbar: 0.0-1.0\n\
             # icons.family: auto | tela | papirus    icons.accent: primary-container | primary | secondary | tertiary | custom\n\
             # shell.panel: black | colored | transparent    shell.panel_opacity / terminals.opacity: 0.0-1.0\n\
             # shell.menus: 0.0-1.0 (menu background darkness)    shell.accent: same roles as icons.accent + shell.accent_tone\n\
             # terminals.scheme: flandre | gnome | tango | solarized | monokai | gruvbox | dracula | nord | catppuccin |\n\
             #   tokyo-night | everforest | rose-pine | ayu | kanagawa    terminals.blend: 0.0-1.0 (pull towards the wallpaper)\n\n{}",
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
