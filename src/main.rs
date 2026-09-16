//! Flandre: wallpaper colours for GNOME Shell, libadwaita/GTK, icons and terminals.

mod apply;
mod colors;
mod config;
mod gui;
mod setup;
mod targets;
mod util;
mod watch;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{Config, Tint, Variant};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "flandre",
    version,
    about = "Wallpaper colours for GNOME Shell, GTK apps, icons and terminals"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate the theme from the current wallpaper and apply it everywhere
    Apply {
        /// Use this image instead of the GNOME wallpaper
        #[arg(short, long)]
        wallpaper: Option<PathBuf>,
        /// Use a flat colour (#rrggbb) instead of an image
        #[arg(short, long)]
        color: Option<String>,
        /// Material scheme variant (overrides the config)
        #[arg(long, value_enum)]
        variant: Option<Variant>,
        /// How strong the wash is (overrides the config)
        #[arg(long, value_enum)]
        tint: Option<Tint>,
        /// Force a dark theme regardless of the system preference
        #[arg(long, conflicts_with = "light")]
        dark: bool,
        /// Force a light theme regardless of the system preference
        #[arg(long)]
        light: bool,
        /// Regenerate even if nothing changed
        #[arg(short, long)]
        force: bool,
        /// No desktop notification
        #[arg(short, long)]
        quiet: bool,
    },
    /// Stay running and re-apply on wallpaper or light/dark changes
    Watch,
    /// Install the service, Flatpak overrides, User Themes and shell snippet, then apply
    Setup {
        /// Do not touch ~/.bashrc, ~/.zshrc or fish config
        #[arg(long)]
        no_rc: bool,
        /// Do not add Flatpak overrides
        #[arg(long)]
        no_flatpak: bool,
    },
    /// Check every piece Flandre depends on
    Doctor,
    /// Print the generated palette (hex) for the current wallpaper
    Colors {
        #[arg(long)]
        light: bool,
        #[arg(long)]
        json: bool,
    },
    /// Print the escape sequences that repaint a terminal (for shell rc files)
    Sequences,
    /// Open the settings window (preview, tint, icons, top bar)
    Settings,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Apply {
            wallpaper,
            color,
            variant,
            tint,
            dark,
            light,
            force,
            quiet,
        } => {
            let cfg = Config::load()?;
            let opts = apply::Options {
                wallpaper,
                color,
                variant,
                tint,
                dark: if dark {
                    Some(true)
                } else if light {
                    Some(false)
                } else {
                    None
                },
                force,
                quiet,
            };
            let r = apply::run(&cfg, &opts)?;
            for l in r.lines {
                println!("{l}");
            }
            for w in r.warnings {
                eprintln!("warning: {w}");
            }
            if r.skipped {
                std::process::exit(2);
            }
        }
        Cmd::Watch => watch::run()?,
        Cmd::Setup { no_rc, no_flatpak } => {
            for l in setup::run(no_rc, no_flatpak)? {
                println!("{l}");
            }
            let cfg = Config::load()?;
            let r = apply::run(
                &cfg,
                &apply::Options {
                    force: true,
                    ..Default::default()
                },
            )?;
            for l in r.lines {
                println!("{l}");
            }
            for w in r.warnings {
                eprintln!("warning: {w}");
            }
        }
        Cmd::Doctor => {
            let mut bad = 0;
            for (what, ok, detail) in setup::doctor() {
                println!("{} {:<26} {}", if ok { "ok  " } else { "FAIL" }, what, detail);
                if !ok {
                    bad += 1;
                }
            }
            if bad > 0 {
                std::process::exit(1);
            }
        }
        Cmd::Colors { light, json } => {
            let cfg = Config::load()?;
            let dark = if light { false } else { apply::prefers_dark() };
            let w = apply::current_wallpaper(dark)?;
            let source = colors::source_from_image(&w)?;
            let theme = colors::build_theme(source, cfg.variant);
            let p = colors::Palette::build(&theme, dark, cfg.tint, cfg.darken);
            if json {
                print!("{}", targets::osc::colors_json(&p));
            } else {
                print!("{}", targets::osc::colors_sh(&p));
            }
        }
        Cmd::Settings => gui::run()?,
        Cmd::Sequences => {
            let cfg = Config::load()?;
            let dark = apply::prefers_dark();
            let w = apply::current_wallpaper(dark)?;
            let source = colors::source_from_image(&w)?;
            let theme = colors::build_theme(source, cfg.variant);
            let p = colors::Palette::build(&theme, dark, cfg.tint, cfg.darken);
            print!("{}", targets::osc::sequences(&p));
        }
    }
    Ok(())
}
