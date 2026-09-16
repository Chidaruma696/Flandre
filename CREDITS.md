# Credits

Flandre replaces a pile of glue that other people built first. None of their code is copied
here, but the ideas are theirs.

| Project | Author | Licence | What Flandre took from it |
|---|---|---|---|
| [Material You Colors](https://github.com/FrancescoCaracciolo/material-you-colors) and its backend [adwaita-material-you](https://github.com/francescocaracciolo/adwaita-material-you) | Francesco Caracciolo | GPL | The whole "wallpaper → Material scheme → Adwaita named colours → User Themes" pipeline, and the role-mapping tables (`color_mappings.json`) that the tint levels in `src/colors.rs` grew out of. |
| [Material You Color Theming](https://github.com/avanishsubbiah/material-you-theme) | Avanish Subbiah | GPL-3.0 | The original GNOME extension the one above forked. |
| [NyarchLinux](https://github.com/NyarchLinux/NyarchLinux) (`after_wp_change.sh`) | Nyarch team | GPL-3.0 | Recolouring Tela from the accent after every wallpaper change, with a hash-named theme so GTK drops its icon cache. |
| [Tela icon theme](https://github.com/vinceliuice/Tela-icon-theme) and [Tela-circle](https://github.com/vinceliuice/Tela-circle-icon-theme) | Vince Liuice | GPL-3.0 | The icons themselves (recoloured at runtime on the user's machine, never redistributed) and the `install.sh` hex-colour substitution that `src/targets/icons.rs` reimplements. |
| [material-color-utilities](https://github.com/material-foundation/material-color-utilities) | Google | Apache-2.0 | The HCT colour space, scheme variants (Tonal Spot, Vibrant, Expressive, Fruit Salad, ...) and harmonisation. |
| [material-colors](https://crates.io/crates/material-colors) | Aiving | MIT / Apache-2.0 | Rust port of the above, used as a library. |
| [pywal](https://github.com/dylanaraps/pywal) | Dylan Araps | MIT | Broadcasting OSC colour sequences to every open pty and caching them for shell rc files. |
| [adw-gtk3](https://github.com/lassekongo83/adw-gtk3) | lassekongo83 | LGPL-2.1 | The GTK 3 theme that reads the same named colours. |
| [GNOME Shell](https://gitlab.gnome.org/GNOME/gnome-shell) | GNOME | GPL-2.0-or-later | The stock stylesheet is read from the installed GResource and recoloured locally; nothing is shipped. |
| [Ptyxis](https://gitlab.gnome.org/chergert/ptyxis) | Christian Hergert | GPL-3.0 | Palette file format. |
| [GNOME Console](https://gitlab.gnome.org/GNOME/console) | GNOME | GPL-3.0 | Livery GVariant format. |
| [Black Box](https://gitlab.gnome.org/raggesilver/blackbox) | Paulo Queiroz | GPL-3.0 | Scheme JSON format. |
| [gtk-rs](https://gtk-rs.org/) (`gio`, `glib`) | gtk-rs team | MIT | GSettings, GResource and the main loop. |

Touhou Project and its characters belong to Team Shanghai Alice (ZUN); this is an unofficial fan
work made under their guidelines for derivative works, with no affiliation or endorsement.
