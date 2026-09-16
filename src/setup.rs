//! One-time wiring: binary in ~/.local/bin, systemd user unit, Flatpak overrides, User Themes,
//! retiring the Material You extension, shell rc snippet.

use crate::config::Config;
use crate::targets::shell;
use crate::util;
use anyhow::{Context, Result};
use std::path::PathBuf;

const OLD_EXTENSION: &str = "material-you-colors@francescocaracciolo.github.io";
pub const RC_MARKER: &str = "# flandre: terminal colours";

pub fn bin_path() -> PathBuf {
    util::home().join(".local/bin/flandre")
}

pub fn unit_path() -> PathBuf {
    util::config_home().join("systemd/user/flandre.service")
}

pub fn run(no_rc: bool, no_flatpak: bool) -> Result<Vec<String>> {
    let mut log = Vec::new();

    if Config::write_default_if_missing()? {
        log.push(format!("wrote default config {}", crate::config::path().display()));
    }

    // 1. binary
    let me = std::env::current_exe().context("locating this binary")?;
    let target = bin_path();
    if me != target {
        std::fs::create_dir_all(target.parent().unwrap())?;
        std::fs::copy(&me, &target).with_context(|| format!("copying to {}", target.display()))?;
        log.push(format!("installed {}", target.display()));
    }

    // 2. systemd user unit
    let unit = format!(
        "[Unit]\nDescription=Flandre - wallpaper colours for GNOME, GTK and terminals\nPartOf=graphical-session.target\nAfter=graphical-session.target\n\n\
[Service]\nType=simple\nExecStart={} watch\nRestart=on-failure\nRestartSec=3\n\n[Install]\nWantedBy=graphical-session.target\n",
        target.display()
    );
    util::write_atomic(&unit_path(), unit.as_bytes())?;
    util::run_quiet("systemctl", &["--user", "daemon-reload"]);
    if util::run_quiet("systemctl", &["--user", "enable", "--now", "flandre.service"]) {
        util::run_quiet("systemctl", &["--user", "restart", "flandre.service"]);
        log.push("systemd user service flandre.service enabled and started".into());
    } else {
        log.push("warning: could not enable flandre.service (systemctl --user)".into());
    }

    // 3. User Themes on, Material You off
    if util::exists_in_path("gnome-extensions") {
        if util::run_quiet("gnome-extensions", &["enable", shell::USER_THEME_UUID]) {
            log.push("User Themes extension enabled".into());
        } else {
            log.push("warning: User Themes extension not found (install gnome-shell-extensions)".into());
        }
        if util::run("gnome-extensions", &["info", OLD_EXTENSION]).is_ok() {
            util::run_quiet("gnome-extensions", &["disable", OLD_EXTENSION]);
            log.push("Material You Colors extension disabled (Flandre replaces it)".into());
        }
    }

    // 4. Flatpak: let sandboxed apps read the GTK CSS, icons and the Shell theme.
    if !no_flatpak && util::exists_in_path("flatpak") {
        let ok = util::run_quiet(
            "flatpak",
            &[
                "override",
                "--user",
                "--filesystem=xdg-config/gtk-4.0:ro",
                "--filesystem=xdg-config/gtk-3.0:ro",
                "--filesystem=xdg-data/icons:ro",
                "--filesystem=xdg-data/themes:ro",
            ],
        );
        log.push(if ok {
            "flatpak overrides added (gtk css, icons, themes)".into()
        } else {
            "warning: flatpak override failed".into()
        });
    }

    // 5. shell rc: repaint new terminals (Console has no palette API before 49).
    if !no_rc {
        let snippet = format!(
            "\n{RC_MARKER}\n[ -r \"${{XDG_CACHE_HOME:-$HOME/.cache}}/flandre/sequences\" ] && [ -t 1 ] && cat \"${{XDG_CACHE_HOME:-$HOME/.cache}}/flandre/sequences\"\n"
        );
        for rc in [".bashrc", ".zshrc"] {
            let path = util::home().join(rc);
            if !path.exists() {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            if text.contains(RC_MARKER) {
                continue;
            }
            let mut text = text;
            text.push_str(&snippet);
            std::fs::write(&path, text)?;
            log.push(format!("added terminal colours snippet to ~/{rc}"));
        }
        let fish = util::config_home().join("fish/config.fish");
        if fish.exists() {
            let text = std::fs::read_to_string(&fish)?;
            if !text.contains(RC_MARKER) {
                let snippet = format!(
                    "\n{RC_MARKER}\nif status is-interactive; and test -r ~/.cache/flandre/sequences\n    cat ~/.cache/flandre/sequences\nend\n"
                );
                std::fs::write(&fish, format!("{text}{snippet}"))?;
                log.push("added terminal colours snippet to fish config".into());
            }
        }
    }
    Ok(log)
}

pub fn doctor() -> Vec<(String, bool, String)> {
    let mut rows = Vec::new();
    let check = |rows: &mut Vec<(String, bool, String)>, what: &str, ok: bool, detail: String| {
        rows.push((what.to_string(), ok, detail))
    };
    check(
        &mut rows,
        "GNOME session",
        util::schema_exists("org.gnome.desktop.background"),
        "org.gnome.desktop.background schema".into(),
    );
    check(
        &mut rows,
        "gnome-shell stylesheet",
        std::path::Path::new("/usr/share/gnome-shell/gnome-shell-theme.gresource").exists(),
        "/usr/share/gnome-shell/gnome-shell-theme.gresource".into(),
    );
    check(
        &mut rows,
        "User Themes schema",
        util::schema_exists("org.gnome.shell.extensions.user-theme"),
        "package gnome-shell-extensions".into(),
    );
    check(
        &mut rows,
        "User Themes enabled",
        shell::user_theme_enabled(),
        shell::USER_THEME_UUID.into(),
    );
    let old = util::run("gnome-extensions", &["info", OLD_EXTENSION])
        .map(|o| ext_state_is(&o, "ACTIVE"))
        .unwrap_or(false);
    check(
        &mut rows,
        "Material You Colors off",
        !old,
        if old {
            "still active: it will fight over gtk.css".into()
        } else {
            "not active".into()
        },
    );
    let cfg = Config::load();
    let base = cfg
        .as_ref()
        .map(|c| c.icons.base.clone())
        .unwrap_or_else(|_| "Tela".into());
    let tela = ["/usr/share/icons", &format!("{}/icons", util::data_home().display())]
        .iter()
        .any(|d| std::path::Path::new(d).join(&base).join("index.theme").exists());
    check(&mut rows, "icon family", tela, format!("{base} (+ {base}-dark/-light)"));
    check(
        &mut rows,
        "adw-gtk3",
        util::get_string("org.gnome.desktop.interface", "gtk-theme").is_some_and(|t| t.starts_with("adw-gtk3")),
        "GTK3 theme should be adw-gtk3 / adw-gtk3-dark".into(),
    );
    // Terminals are optional: missing ones are reported, not failed.
    let ptyxis = util::exists_in_path("ptyxis") || util::flatpak_app_installed("app.devsuite.Ptyxis");
    check(
        &mut rows,
        "Ptyxis",
        true,
        if ptyxis {
            "found".into()
        } else {
            "not installed (skipped)".into()
        },
    );
    let console = util::schema_exists("org.gnome.Console") || util::flatpak_app_installed("org.gnome.Console");
    check(
        &mut rows,
        "GNOME Console",
        true,
        if console {
            "found (liveries need Console 49+)".into()
        } else {
            "not installed (skipped)".into()
        },
    );
    let blackbox = util::exists_in_path("blackbox") || util::flatpak_app_installed("com.raggesilver.BlackBox");
    check(
        &mut rows,
        "Black Box",
        true,
        if blackbox {
            "found".into()
        } else {
            "not installed (skipped)".into()
        },
    );
    check(
        &mut rows,
        "service",
        util::run("systemctl", &["--user", "is-active", "flandre.service"])
            .map(|s| s == "active")
            .unwrap_or(false),
        unit_path().display().to_string(),
    );
    check(
        &mut rows,
        "config",
        cfg.is_ok(),
        crate::config::path().display().to_string(),
    );
    let rc = std::fs::read_to_string(util::home().join(".bashrc"))
        .map(|t| t.contains(RC_MARKER))
        .unwrap_or(false);
    check(&mut rows, "shell rc snippet", rc, "~/.bashrc".into());
    rows
}

/// `gnome-extensions info` is localised; the state value itself is not.
pub fn ext_state_is(info: &str, state: &str) -> bool {
    info.lines().any(|l| l.trim_end().ends_with(&format!(": {state}")))
}
