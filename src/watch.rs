//! Daemon: re-applies whenever the wallpaper or the light/dark preference changes.

use crate::apply::{self, Options};
use crate::config::Config;
use anyhow::{Result, anyhow};
use gio::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub fn run() -> Result<()> {
    let bg = crate::util::settings("org.gnome.desktop.background")
        .ok_or_else(|| anyhow!("org.gnome.desktop.background schema missing: is this a GNOME session?"))?;
    let iface = crate::util::settings("org.gnome.desktop.interface")
        .ok_or_else(|| anyhow!("org.gnome.desktop.interface schema missing"))?;
    let main = glib::MainLoop::new(None, false);
    let pending: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

    let schedule = {
        let pending = pending.clone();
        move |why: &str| {
            let why = why.to_string();
            let mut slot = pending.borrow_mut();
            if let Some(id) = slot.take() {
                id.remove();
            }
            let pending2 = pending.clone();
            *slot = Some(glib::timeout_add_local_once(Duration::from_millis(900), move || {
                *pending2.borrow_mut() = None;
                eprintln!("flandre: {why}, regenerating");
                match Config::load().and_then(|cfg| apply::run(&cfg, &Options::default())) {
                    Ok(r) => {
                        for l in &r.lines {
                            eprintln!("  {l}");
                        }
                        for w in &r.warnings {
                            eprintln!("  warning: {w}");
                        }
                    }
                    Err(e) => eprintln!("flandre: apply failed: {e:#}"),
                }
            }));
        }
    };

    for key in ["picture-uri", "picture-uri-dark"] {
        let s = schedule.clone();
        bg.connect_changed(Some(key), move |_, k| s(&format!("wallpaper changed ({k})")));
    }
    {
        let s = schedule.clone();
        iface.connect_changed(Some("color-scheme"), move |_, _| s("colour scheme changed"));
    }
    // Keep the settings objects alive for the whole loop.
    let _keep = (bg, iface);
    // Initial pass covers what changed while the daemon was not running.
    schedule("startup");
    eprintln!("flandre: watching the wallpaper and colour scheme");
    main.run();
    Ok(())
}
