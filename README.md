[English](README.md) · [Español](README.es.md)

# Flandre 〜 フランドール

**Wallpaper colours for the whole GNOME desktop, in one binary.** GNOME Shell, libadwaita and
GTK 3 apps, the icon theme, Ptyxis, GNOME Console, Black Box and any terminal already open: all
regenerated from the wallpaper every time it (or the light/dark preference) changes.

![Rust](https://img.shields.io/badge/Rust-2024-orange) ![GNOME](https://img.shields.io/badge/GNOME-47%E2%80%9350-blue) ![Licence](https://img.shields.io/badge/licence-MIT-green)

## Why

The usual way to get Material You on GNOME is a GNOME extension with a Python backend that clones
itself a venv, a User Themes copy of a stylesheet compiled for one Shell version, a Nyarch shell
script that re-installs a 118 MB icon theme with `sed`, and pywal writing escape codes into every
pty. It works until it doesn't. Flandre is that pipeline as a single Rust program with no runtime
dependencies beyond GLib, and it reaches further:

| Target | How |
|---|---|
| **libadwaita (GTK 4)** and **adw-gtk3 (GTK 3)** | Every named colour libadwaita exposes (accent, surfaces, headerbar, sidebars, cards, dialogs, popovers, the `blue_1`…`dark_5` palette rows) in `~/.config/gtk-{3,4}.0/gtk.css`, plus a few rules that push the wash into selections, tabs, switches and OSDs. GTK 4 gets both modes as CSS variables under `@media (prefers-color-scheme)`, so running apps follow a light/dark switch without restarting. |
| **GNOME Shell** | The stylesheet of the *installed* gnome-shell is read from its GResource and every colour literal is rewritten (greys take the wallpaper hue, saturated colours are harmonised, the accent keywords become the primary). Loaded through User Themes, so it never lags behind a Shell update. |
| **Icons** | A `Flandre-<family>-<hex>` theme built on top of Tela or Papirus: every SVG carrying the family's folder colour is recoloured (all sizes, symlinked names included) with the scheme colour you pick (primary container by default, like Material You), everything else is inherited. Under a second. |
| **Ptyxis** | A native `.palette` (light + dark, titlebar included) selected in every profile. Host package and Flatpak. |
| **GNOME Console** | A custom *livery* written to `org.gnome.Console custom-liveries` and selected (Console 49+). |
| **Black Box** | `Flandre Dark` / `Flandre Light` JSON schemes plus `theme-dark` / `theme-light` / `pretty`. Host package and Flatpak. |
| **Terminal colours** | `Flandre` (default) builds the 16 ANSI colours from the scheme itself: blue = primary, magenta = tertiary, red = error, green, yellow and cyan harmonised with the wallpaper. Or pick a classic base (GNOME, Tango, Solarized, Monokai, Gruvbox, Dracula, Nord, Catppuccin, Tokyo Night, Everforest, Rosé Pine, Ayu, Kanagawa) and a *blend* slider decides how far it is pulled towards the wallpaper. Light and dark variants; synthesised for schemes that only have a dark one. |
| **Terminal opacity** | One slider for Ptyxis (`opacity` in every profile), Console (livery transparency) and Black Box (`opacity`). |
| **Open terminals** | OSC 4/10/11/12 sequences broadcast to every pty, cached in `~/.cache/flandre/sequences` for shell rc files, with `colors.json` / `colors.sh` for your own scripts. |
| **Flatpaks** | `flatpak override --user` so sandboxed apps can read the GTK CSS, icons and themes. |

Three tint levels (`soft`, `normal`, `strong`) control how far the wallpaper hue goes into the
neutral greys and how much of the accent washes headerbars, sidebars and popovers. `strong` is the
default and is meant to be aggressive; it keeps dark surfaces dark by tinting hue at constant tone,
and a `darken` slider takes the dark tones from Material's down to near black. A separate
`headerbar` slider darkens the window decoration on its own, from Adwaita's (a notch lighter than
the window) to near black.

## Settings window

`flandre settings` (also "Flandre" in the app grid after `setup`) is a libadwaita window with a live
preview: a mock desktop (top bar, headerbar, sidebar, card, popover, buttons), the palette, the
recoloured folder icons and the terminal colours. It edits scheme, tint, darkness, title bar darkness, icon family
(Tela / Papirus), which scheme colour paints the icons (or a custom tone), the top bar style
(black, coloured, transparent) and the targets, then applies with one button.

## Install

```sh
git clone https://github.com/Chidaruma696/Flandre
cd Flandre
cargo build --release
./target/release/flandre setup
```

`setup` copies the binary to `~/.local/bin/flandre`, installs and starts a systemd user service
(`flandre watch`), enables User Themes, disables the Material You Colors extension if present
(they would fight over `gtk.css`), adds the Flatpak overrides, appends a snippet to `~/.bashrc`
(and `~/.zshrc` / fish if they exist) that repaints new terminals, writes a default config and
applies the theme once. `--no-rc` and `--no-flatpak` skip those two steps.

Requirements: GNOME 47 or newer, the `gnome-shell-extensions` package (User Themes),
`adw-gtk3` and Tela or Papirus (`tela-icon-theme` / `papirus-icon-theme` on Arch). Build needs
Rust, GTK 4 and libadwaita development files (`gtk4`, `libadwaita` on Arch; `libgtk-4-dev`,
`libadwaita-1-dev` on Debian).

## Use

```
flandre apply            # regenerate from the current wallpaper (skips if nothing changed)
flandre apply --force    # regenerate anyway
flandre apply --wallpaper ~/Pictures/x.jpg --variant vibrant --tint normal --light
flandre apply --color '#6750a4'
flandre watch            # what the service runs
flandre doctor           # check every piece
flandre colors [--json]  # print the palette
flandre sequences        # print the escape codes
flandre settings         # the preview window
```

Configuration lives in `~/.config/flandre/config.toml`; see [`config.example.toml`](config.example.toml).

## What it will not do

libadwaita only lets a theme override its named colours; an app that hard-codes colours in its own
CSS stays as it is. Icons other than folders, places, devices and the coloured action icons keep
Tela's own colours. GNOME Console older than 49 has no palette API, so it is only repainted through
the escape sequences (and the shell rc snippet for new tabs).

## Credits

See [CREDITS.md](CREDITS.md). The pipeline was first put together by Material You Colors,
adwaita-material-you and NyarchLinux; Flandre rewrites it rather than copying it.

## Licence

MIT. Touhou Project and its characters belong to Team Shanghai Alice (ZUN); this is an unofficial
fan work made under their guidelines for derivative works, with no affiliation or endorsement.
