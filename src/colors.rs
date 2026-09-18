//! Wallpaper -> Material scheme -> the concrete colours every target needs.
//!
//! The role mapping (Adwaita named colours <- Material roles) started from the
//! tables in adwaita-material-you by Francesco Caracciolo and was reworked so
//! the tint level can be dialled up.

use crate::config::{AccentRole, Config, Icons, Terminals, Tint, Variant};
use crate::schemes;
use anyhow::{Context, Result, anyhow};
use material_colors::{
    blend,
    color::Argb,
    dynamic_color::Variant as MVariant,
    hct::Hct,
    image::{FilterType, Image, ImageReader},
    palette::TonalPalette,
    theme::{Theme, ThemeBuilder},
};
use std::path::Path;
use std::str::FromStr;

pub fn hex(c: Argb) -> String {
    format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue)
}

pub fn parse_hex(s: &str) -> Result<Argb> {
    Argb::from_str(s.trim()).map_err(|_| anyhow!("invalid colour {s:?}"))
}

fn mvariant(v: Variant) -> MVariant {
    match v {
        Variant::TonalSpot => MVariant::TonalSpot,
        Variant::Vibrant => MVariant::Vibrant,
        Variant::Expressive => MVariant::Expressive,
        Variant::FruitSalad => MVariant::FruitSalad,
        Variant::Rainbow => MVariant::Rainbow,
        Variant::Neutral => MVariant::Neutral,
        Variant::Monochrome => MVariant::Monochrome,
        Variant::Fidelity => MVariant::Fidelity,
        Variant::Content => MVariant::Content,
    }
}

/// Picks the dominant, theme-worthy colour of an image (quantised at 128x128 like Material does).
pub fn source_from_image(path: &Path) -> Result<Argb> {
    let rgba = decode_wallpaper(path).with_context(|| format!("reading {}", path.display()))?;
    let mut img = Image::new(rgba);
    img.resize(128, 128, FilterType::Lanczos3);
    Ok(ImageReader::extract_color(&img))
}

/// Decodes a wallpaper ourselves instead of through `material_colors::ImageReader::open`, which
/// panics on anything the `image` crate cannot read. GNOME ships its stock backgrounds as JPEG XL
/// since 45, so those go through jxl-oxide.
fn decode_wallpaper(path: &Path) -> Result<image::RgbaImage> {
    let is_jxl = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("jxl"));
    let decoded = if is_jxl {
        decode_jxl(path)
    } else {
        match image::ImageReader::open(path)?.with_guessed_format()?.decode() {
            Ok(img) => Ok(img),
            // Unknown format, or a JXL hiding behind a .jpg extension (the `image` crate then
            // trusts the extension and fails inside the JPEG decoder): try jxl-oxide before
            // giving up, and report the original error if that fails too.
            Err(e) => decode_jxl(path).map_err(|_| e.into()),
        }
    }?;
    Ok(decoded.into_rgba8())
}

fn decode_jxl(path: &Path) -> Result<image::DynamicImage> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    let decoder = jxl_oxide::integration::JxlDecoder::new(file)?;
    Ok(image::DynamicImage::from_decoder(decoder)?)
}

pub fn build_theme(source: Argb, variant: Variant) -> Theme {
    ThemeBuilder::with_source(source).variant(mvariant(variant)).build()
}

/// One resolved mode (dark or light) with every colour the targets consume.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Palette {
    pub dark: bool,
    pub source: Argb,
    pub tint: Tint,
    pub darken: f64,
    // Material roles
    pub primary: Argb,
    pub on_primary: Argb,
    pub primary_container: Argb,
    pub on_primary_container: Argb,
    pub secondary: Argb,
    pub on_secondary: Argb,
    pub secondary_container: Argb,
    pub on_secondary_container: Argb,
    pub tertiary: Argb,
    pub on_tertiary: Argb,
    pub tertiary_container: Argb,
    pub on_tertiary_container: Argb,
    pub error: Argb,
    pub on_error: Argb,
    pub error_container: Argb,
    pub on_error_container: Argb,
    pub surface: Argb,
    pub surface_dim: Argb,
    pub surface_bright: Argb,
    pub surface_container_lowest: Argb,
    pub surface_container_low: Argb,
    pub surface_container: Argb,
    pub surface_container_high: Argb,
    pub surface_container_highest: Argb,
    pub on_surface: Argb,
    pub on_surface_variant: Argb,
    pub outline: Argb,
    pub outline_variant: Argb,
    pub inverse_surface: Argb,
    pub inverse_on_surface: Argb,
    pub inverse_primary: Argb,
    // Adwaita roles
    pub window_bg: Argb,
    pub view_bg: Argb,
    pub headerbar_bg: Argb,
    pub sidebar_bg: Argb,
    pub secondary_sidebar_bg: Argb,
    pub card_bg: Argb,
    pub dialog_bg: Argb,
    pub popover_bg: Argb,
    pub thumbnail_bg: Argb,
    pub success: Argb,
    pub warning: Argb,
    pub success_bg: Argb,
    pub warning_bg: Argb,
    // Adwaita palette rows (index 0 = _1 ... 4 = _5)
    pub blue: [Argb; 5],
    pub green: [Argb; 5],
    pub yellow: [Argb; 5],
    pub orange: [Argb; 5],
    pub red: [Argb; 5],
    pub purple: [Argb; 5],
    pub brown: [Argb; 5],
    pub light: [Argb; 5],
    pub darks: [Argb; 5],
    // Terminal
    pub term_bg: Argb,
    pub term_fg: Argb,
    pub cursor: Argb,
    pub ansi: [Argb; 16],
    // Neutral hue/chroma used to recolour arbitrary greys (Shell CSS)
    pub neutral_hue: f64,
    pub neutral_chroma: f64,
    // GNOME Shell menus (`shell.menus` / `shell.accent`)
    pub shell_menus: f64,
    /// Tone of the stock popover background, the anchor of the `shell.menus` curve.
    pub shell_anchor: f64,
    pub shell_accent: Argb,
    pub shell_on_accent: Argb,
    /// Stock popover background (`.popup-menu-content`) after recolouring, for the preview.
    pub shell_menu_bg: Argb,
    /// Stock quick-toggle background after recolouring, for the preview.
    pub shell_item_bg: Argb,
}

/// Stock backgrounds of the Shell's popovers and quick toggles (gnome-shell-{dark,light}.css).
const SHELL_POPOVER_BG: (&str, &str) = ("#36363a", "#fafafb");
const SHELL_ITEM_BG: (&str, &str) = ("#47474c", "#ffffff");

fn tone(p: &TonalPalette, t: f64) -> Argb {
    Hct::from(p.hue(), p.chroma(), t).into()
}

/// Pushes `base` towards the hue of `accent` while keeping its tone, so dark surfaces stay dark.
fn wash(base: Argb, accent: Argb, amount: f64) -> Argb {
    if amount <= 0.0 {
        return base;
    }
    let b = Hct::new(base);
    let a = Hct::new(accent);
    let target: Argb = Hct::from(a.get_hue(), a.get_chroma().max(40.0), b.get_tone()).into();
    blend::cam16_ucs(base, target, amount)
}

fn harmonize(design: &str, source: Argb) -> Argb {
    blend::harmonize(Argb::from_str(design).expect("static colour"), source)
}

/// Background, foreground, cursor and the 16 ANSI colours for `term.scheme`.
///
/// `Flandre` is built from the Material palettes alone. A classic scheme starts from its
/// published colours (`schemes.rs`; a light variant is synthesised when it has none) and each
/// colour is then moved towards the wallpaper by `term.blend`: saturated ones are harmonised
/// (hue shifted towards the source), greys, background and foreground take the wallpaper hue.
fn terminal_colors(
    theme: &Theme,
    dark: bool,
    source: Argb,
    primary: Argb,
    term: &Terminals,
    n: &impl Fn(f64) -> Argb,
    wash_amount: f64,
) -> (Argb, Argb, Argb, [Argb; 16]) {
    let blend_amount = term.blend.clamp(0.0, 1.0);
    let (bg_tone, fg_tone) = if dark { (4.0, 92.0) } else { (99.0, 12.0) };
    let bg_wash = if dark { 0.3 } else { 0.2 } * wash_amount;

    let Some(scheme) = schemes::builtin(term.scheme) else {
        // Flandre: hues from the scheme, chroma from the wallpaper, tones fixed per mode.
        let pp = &theme.palettes.primary;
        let tp = &theme.palettes.tertiary;
        let ep = &theme.palettes.error;
        let chroma = pp.chroma().clamp(36.0, 64.0);
        let fixed = |hue: f64, t: f64| -> Argb { blend::harmonize(Hct::from(hue, chroma, t).into(), source) };
        let (lo, hi) = if dark { (65.0, 80.0) } else { (40.0, 52.0) };
        // Yellow only reads as yellow at high tones; cyan is a fixed hue because Material's
        // secondary is too grey for a terminal.
        let (ylo, yhi) = if dark { (78.0, 88.0) } else { (48.0, 58.0) };
        let (black, bright_black, white, bright_white) = if dark {
            (20.0, 45.0, 80.0, 96.0)
        } else {
            (15.0, 45.0, 80.0, 98.0)
        };
        let ansi = [
            n(black),
            tone(ep, lo),
            fixed(145.0, lo),
            fixed(95.0, ylo),
            tone(pp, lo),
            tone(tp, lo),
            fixed(200.0, lo),
            n(white),
            n(bright_black),
            tone(ep, hi),
            fixed(145.0, hi),
            fixed(95.0, yhi),
            tone(pp, hi),
            tone(tp, hi),
            fixed(200.0, hi),
            n(bright_white),
        ];
        return (wash(n(bg_tone), primary, bg_wash), n(fg_tone), primary, ansi);
    };

    let parse = |s: &str| Argb::from_str(s).expect("static colour");
    let (bg0, fg0, cursor0, ansi0): (Argb, Argb, Argb, [Argb; 16]) = match (dark, scheme.light.as_ref()) {
        (true, _) => (
            parse(scheme.dark.bg),
            parse(scheme.dark.fg),
            parse(scheme.dark.cursor),
            scheme.dark.ansi.map(parse),
        ),
        (false, Some(l)) => (parse(l.bg), parse(l.fg), parse(l.cursor), l.ansi.map(parse)),
        (false, None) => {
            // No light variant: keep the hues, drop the tones so they read on a light background.
            let mut ansi = scheme.dark.ansi.map(parse);
            for (i, c) in ansi.iter_mut().enumerate() {
                *c = match i {
                    0 => n(15.0),
                    7 => n(80.0),
                    8 => n(45.0),
                    15 => n(98.0),
                    _ => {
                        let h = Hct::new(*c);
                        Hct::from(h.get_hue(), h.get_chroma().max(30.0), if i < 8 { 40.0 } else { 50.0 }).into()
                    }
                };
            }
            (n(99.0), n(12.0), n(12.0), ansi)
        }
    };
    if blend_amount <= 0.0 {
        return (bg0, fg0, cursor0, ansi0);
    }
    let grey = |c: Argb, amount: f64| wash(c, primary, amount * blend_amount);
    let mut ansi = ansi0;
    for (i, c) in ansi.iter_mut().enumerate() {
        *c = match i {
            0 | 7 | 8 | 15 => grey(*c, 0.5),
            _ => blend::cam16_ucs(*c, blend::harmonize(*c, source), blend_amount),
        };
    }
    let bg = grey(bg0, if dark { 0.6 } else { 0.4 });
    let fg = grey(fg0, 0.4);
    let cursor = blend::cam16_ucs(cursor0, primary, blend_amount);
    (bg, fg, cursor, ansi)
}

impl Palette {
    pub fn build(theme: &Theme, dark: bool, cfg: &Config) -> Self {
        let tint = cfg.tint;
        let term = &cfg.terminals;
        let darken = cfg.darken.clamp(0.0, 1.0);
        let headerbar = cfg.headerbar.clamp(0.0, 1.0);
        let s = if dark {
            &theme.schemes.dark
        } else {
            &theme.schemes.light
        };
        let neutral_hue = theme.palettes.neutral.hue();
        // Material's own neutral chroma is ~4-10 depending on the variant. We push it for stronger tints.
        let neutral_chroma = match tint {
            Tint::Soft => theme.palettes.neutral.chroma().min(6.0),
            Tint::Normal => theme.palettes.neutral.chroma().max(10.0),
            Tint::Strong => theme.palettes.neutral.chroma().max(18.0),
        };
        let neutral = TonalPalette::of(neutral_hue, neutral_chroma);
        let n = |t: f64| tone(&neutral, t);
        let rebuild = tint != Tint::Soft;
        let pick = |scheme_color: Argb, t: f64| if rebuild { n(t) } else { scheme_color };

        // Dark tones sit a notch under Material's (6/10/12/17/22): the user wants a dark desktop,
        // not a grey-blue one.
        // `darken` slides the dark tones between Material's (6/10/12/17/22) and a near-black set.
        let dt = |material: f64, darkest: f64| material - (material - darkest) * darken;
        let (surface, surface_dim, surface_bright, lowest, low, container, high, highest) = if dark {
            (
                pick(s.surface, dt(6.0, 3.0)),
                pick(s.surface_dim, dt(6.0, 3.0)),
                pick(s.surface_bright, dt(24.0, 19.0)),
                pick(s.surface_container_lowest, dt(4.0, 2.0)),
                pick(s.surface_container_low, dt(10.0, 6.0)),
                pick(s.surface_container, dt(12.0, 9.0)),
                pick(s.surface_container_high, dt(17.0, 12.0)),
                pick(s.surface_container_highest, dt(22.0, 16.0)),
            )
        } else {
            (
                pick(s.surface, 98.0),
                pick(s.surface_dim, 87.0),
                pick(s.surface_bright, 98.0),
                pick(s.surface_container_lowest, 100.0),
                pick(s.surface_container_low, 96.0),
                pick(s.surface_container, 94.0),
                pick(s.surface_container_high, 92.0),
                pick(s.surface_container_highest, 90.0),
            )
        };
        let on_surface = if rebuild {
            n(if dark { 92.0 } else { 10.0 })
        } else {
            s.on_surface
        };
        let on_surface_variant = if rebuild {
            n(if dark { 80.0 } else { 30.0 })
        } else {
            s.on_surface_variant
        };
        let outline = if rebuild {
            n(if dark { 60.0 } else { 50.0 })
        } else {
            s.outline
        };
        let outline_variant = if rebuild {
            n(if dark { 30.0 } else { 80.0 })
        } else {
            s.outline_variant
        };

        // Wash amount for Adwaita surfaces: how much of the primary hue bleeds into each surface.
        let wash_amount = match tint {
            Tint::Soft => 0.0,
            Tint::Normal => 0.5,
            Tint::Strong => 1.0,
        } * if dark { 1.0 } else { 0.7 };
        let p = s.primary;
        let window_bg = wash(low, p, 0.35 * wash_amount);
        let view_bg = wash(surface, p, 0.25 * wash_amount);
        // Window decoration: `headerbar` slides the titlebar tone from Adwaita's (a notch above the
        // window) down to near black in dark mode, or a dim grey in light mode.
        let headerbar_base = if headerbar > 0.0 {
            let from = Hct::new(high).get_tone();
            let to = if dark { 2.0 } else { 74.0 };
            n(from - (from - to) * headerbar)
        } else {
            high
        };
        let headerbar_bg = wash(headerbar_base, p, 0.6 * wash_amount);
        let sidebar_bg = wash(container, p, 0.5 * wash_amount);
        let secondary_sidebar_bg = wash(low, p, 0.35 * wash_amount);
        let card_bg = wash(high, p, 0.4 * wash_amount);
        let dialog_bg = wash(high, p, 0.4 * wash_amount);
        let popover_bg = wash(high, p, 0.5 * wash_amount);
        let thumbnail_bg = high;

        let source = theme.source;
        let pp = &theme.palettes.primary;
        let blue = [85.0, 75.0, 65.0, 55.0, 45.0].map(|t| tone(pp, t));
        let row = |a: &str, b: &str, c: &str, d: &str, e: &str| [a, b, c, d, e].map(|h| harmonize(h, source));
        let green = row("#8ff0a4", "#57e389", "#33d17a", "#2ec27e", "#26a269");
        let yellow = row("#f9f06b", "#f8e45c", "#f6d32d", "#f5c211", "#e5a50a");
        let orange = row("#ffbe6f", "#ffa348", "#ff7800", "#e66100", "#c64600");
        let red = row("#f66151", "#ed333b", "#e01b24", "#c01c28", "#a51d2d");
        let purple = row("#dc8add", "#c061cb", "#9141ac", "#813d9c", "#613583");
        let brown = row("#cdab8f", "#b5835a", "#986a44", "#865e3c", "#63452c");
        let light = [100.0, 96.0, 90.0, 80.0, 64.0].map(n);
        let darks = [50.0, 40.0, 30.0, 20.0, 12.0].map(n);

        // Terminal palette: the chosen base scheme, pulled towards the wallpaper.
        let (bg, fg, cursor, ansi) = terminal_colors(theme, dark, source, p, term, &n, wash_amount);

        let stock = |pair: (&str, &str)| Argb::from_str(if dark { pair.0 } else { pair.1 }).expect("static colour");
        let mut me = Self {
            dark,
            source,
            tint,
            darken,
            primary: s.primary,
            on_primary: s.on_primary,
            primary_container: s.primary_container,
            on_primary_container: s.on_primary_container,
            secondary: s.secondary,
            on_secondary: s.on_secondary,
            secondary_container: s.secondary_container,
            on_secondary_container: s.on_secondary_container,
            tertiary: s.tertiary,
            on_tertiary: s.on_tertiary,
            tertiary_container: s.tertiary_container,
            on_tertiary_container: s.on_tertiary_container,
            error: s.error,
            on_error: s.on_error,
            error_container: s.error_container,
            on_error_container: s.on_error_container,
            surface,
            surface_dim,
            surface_bright,
            surface_container_lowest: lowest,
            surface_container_low: low,
            surface_container: container,
            surface_container_high: high,
            surface_container_highest: highest,
            on_surface,
            on_surface_variant,
            outline,
            outline_variant,
            inverse_surface: s.inverse_surface,
            inverse_on_surface: s.inverse_on_surface,
            inverse_primary: s.inverse_primary,
            window_bg,
            view_bg,
            headerbar_bg,
            sidebar_bg,
            secondary_sidebar_bg,
            card_bg,
            dialog_bg,
            popover_bg,
            thumbnail_bg,
            success: harmonize(if dark { "#8ff0a4" } else { "#1b8553" }, source),
            success_bg: harmonize("#26a269", source),
            warning: harmonize(if dark { "#f8e45c" } else { "#9c6e03" }, source),
            warning_bg: harmonize("#cd9309", source),
            blue,
            green,
            yellow,
            orange,
            red,
            purple,
            brown,
            light,
            darks,
            term_bg: bg,
            term_fg: fg,
            cursor,
            ansi,
            neutral_hue,
            neutral_chroma,
            shell_menus: cfg.shell.menus.clamp(0.0, 1.0),
            shell_anchor: Hct::new(stock(SHELL_POPOVER_BG)).get_tone(),
            shell_accent: s.primary,
            shell_on_accent: s.on_primary,
            shell_menu_bg: stock(SHELL_POPOVER_BG),
            shell_item_bg: stock(SHELL_ITEM_BG),
        };
        me.shell_accent = me.role_color(cfg.shell.accent, cfg.shell.accent_tone);
        me.shell_on_accent = me.on_role(cfg.shell.accent, me.shell_accent);
        me.shell_menu_bg = me.recolor_shell(stock(SHELL_POPOVER_BG));
        me.shell_item_bg = me.recolor_shell(stock(SHELL_ITEM_BG));
        me
    }

    /// One of the scheme's colours by role, or the primary hue at `tone` for `Custom`.
    fn role_color(&self, role: AccentRole, tone: f64) -> Argb {
        match role {
            AccentRole::PrimaryContainer => self.primary_container,
            AccentRole::Primary => self.primary,
            AccentRole::Secondary => self.secondary,
            AccentRole::Tertiary => self.tertiary,
            AccentRole::Custom => {
                let h = Hct::new(self.primary);
                Hct::from(h.get_hue(), h.get_chroma(), tone.clamp(5.0, 95.0)).into()
            }
        }
    }

    /// Text/icon colour that reads on `color` for that role.
    fn on_role(&self, role: AccentRole, color: Argb) -> Argb {
        match role {
            AccentRole::PrimaryContainer => self.on_primary_container,
            AccentRole::Primary => self.on_primary,
            AccentRole::Secondary => self.on_secondary,
            AccentRole::Tertiary => self.on_tertiary,
            AccentRole::Custom => {
                let t = if Hct::new(color).get_tone() > 50.0 { 10.0 } else { 98.0 };
                Hct::from(self.neutral_hue, self.neutral_chroma, t).into()
            }
        }
    }

    /// Tone curve for the Shell's greys (`shell.menus`). Anchored on the stock popover
    /// background, which slides from Adwaita's tone down to near black (dark) or a dim grey
    /// (light) while tone 60 stays put, so hover/selected steps keep their distance instead of
    /// all collapsing into black.
    pub fn shell_tone(&self, t: f64) -> f64 {
        let m = self.shell_menus;
        const PIVOT: f64 = 60.0;
        let a = self.shell_anchor;
        if m <= 0.0 {
            return t;
        }
        let mapped = if self.dark {
            if t >= PIVOT {
                return t;
            }
            let target = a - (a - 2.0) * m;
            target + (t - a) * (PIVOT - target) / (PIVOT - a)
        } else {
            if t <= PIVOT {
                return t;
            }
            let target = a - (a - 74.0) * m;
            target - (a - t) * (target - PIVOT) / (a - PIVOT)
        };
        mapped.clamp(0.0, 100.0)
    }

    /// Moves any colour along the `shell.menus` curve, keeping hue and chroma.
    pub fn shell_grey(&self, c: Argb) -> Argb {
        if self.shell_menus <= 0.0 {
            return c;
        }
        let h = Hct::new(c);
        Hct::from(h.get_hue(), h.get_chroma(), self.shell_tone(h.get_tone())).into()
    }

    /// `recolor` for the Shell stylesheet: greys also follow the `shell.menus` curve.
    pub fn recolor_shell(&self, c: Argb) -> Argb {
        self.recolor_with(c, true)
    }

    /// Icon colour: the scheme role the user picked, applied as is (like Material You did), with an
    /// optional chroma floor so pale schemes still give coloured folders.
    pub fn icon_accent(&self, icons: &Icons) -> Argb {
        let base = self.role_color(icons.accent, icons.tone);
        if icons.chroma > 0.0 {
            let h = Hct::new(base);
            if h.get_chroma() < icons.chroma {
                return Hct::from(h.get_hue(), icons.chroma, h.get_tone()).into();
            }
        }
        base
    }

    /// Recolours an arbitrary colour the way the tint level asks for: greys take the neutral hue,
    /// saturated colours are harmonised towards the wallpaper. Pure black/white are kept.
    fn recolor_with(&self, c: Argb, shell: bool) -> Argb {
        let h = Hct::new(c);
        let t = h.get_tone();
        if t >= 99.5 || t <= 0.5 {
            return c;
        }
        if h.get_chroma() < 14.0 {
            let chroma = if self.tint == Tint::Soft {
                h.get_chroma().max(self.neutral_chroma.min(6.0))
            } else {
                self.neutral_chroma
            };
            let t = if shell { self.shell_tone(t) } else { t };
            // Dark mode backgrounds (tones under 60) go a notch darker than stock.
            let t = if self.dark && t < 60.0 && self.tint != Tint::Soft {
                t * (1.0 - 0.3 * self.darken)
            } else {
                t
            };
            Hct::from(self.neutral_hue, chroma, t).into()
        } else {
            blend::harmonize(c, self.source)
        }
    }
}

pub fn rgb_tuple(c: Argb) -> (f64, f64, f64) {
    (c.red as f64 / 255.0, c.green as f64 / 255.0, c.blue as f64 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_and_rejects_garbage() {
        let c = Argb::new(255, 0x1b, 0x2d, 0x43);
        assert_eq!(hex(c), "#1b2d43");
        assert_eq!(parse_hex(" #1B2D43 ").unwrap(), c);
        assert!(parse_hex("blue").is_err());
        assert!(parse_hex("#12").is_err());
    }

    #[test]
    fn wash_keeps_tone_and_zero_is_identity() {
        let base = Argb::new(255, 0x20, 0x20, 0x24);
        let accent = Argb::new(255, 0xd0, 0x40, 0x30);
        assert_eq!(wash(base, accent, 0.0), base);
        let washed = wash(base, accent, 1.0);
        let t = |c: Argb| Hct::new(c).get_tone();
        assert!((t(washed) - t(base)).abs() < 1.5, "tone moved: {} -> {}", t(base), t(washed));
        assert_ne!(washed, base, "a full wash changes the hue");
    }

    /// Whatever the wallpaper, variant and tint: dark surfaces are dark, light ones light, and
    /// the text on a role always has room to be read.
    #[test]
    fn palettes_are_readable_in_every_mode_variant_and_tint() {
        let tone = |c: Argb| Hct::new(c).get_tone();
        let sources = [
            Argb::new(255, 0x1b, 0x2d, 0x43),
            Argb::new(255, 0xf0, 0x80, 0x20),
            Argb::new(255, 0x80, 0x80, 0x80),
            Argb::new(255, 0x10, 0x90, 0x40),
        ];
        let variants = [
            Variant::TonalSpot, Variant::Vibrant, Variant::Expressive, Variant::FruitSalad, Variant::Rainbow,
            Variant::Neutral, Variant::Monochrome, Variant::Fidelity, Variant::Content,
        ];
        for source in sources {
            for variant in variants {
                let theme = build_theme(source, variant);
                for tint in [Tint::Soft, Tint::Normal, Tint::Strong] {
                    for dark in [true, false] {
                        let mut cfg = Config::default();
                        cfg.tint = tint;
                        let p = Palette::build(&theme, dark, &cfg);
                        let what = format!("{source:?} {variant:?} {tint:?} dark={dark}");
                        if dark {
                            assert!(tone(p.window_bg) < 30.0, "{what}: window {}", tone(p.window_bg));
                            assert!(tone(p.term_bg) < 20.0, "{what}: term bg {}", tone(p.term_bg));
                            assert!(tone(p.on_surface) > 75.0, "{what}");
                        } else {
                            assert!(tone(p.window_bg) > 85.0, "{what}: window {}", tone(p.window_bg));
                            assert!(tone(p.term_bg) > 90.0, "{what}: term bg {}", tone(p.term_bg));
                            assert!(tone(p.on_surface) < 30.0, "{what}");
                        }
                        assert!((tone(p.primary) - tone(p.on_primary)).abs() > 40.0, "{what}: on_primary");
                        assert!((tone(p.term_bg) - tone(p.term_fg)).abs() > 60.0, "{what}: terminal text");
                        assert_eq!(p.ansi.len(), 16);
                    }
                }
            }
        }
    }

    /// `darken` only pushes dark mode down; `headerbar` only the decoration.
    #[test]
    fn darken_and_headerbar_dials() {
        let tone = |c: Argb| Hct::new(c).get_tone();
        let theme = build_theme(Argb::new(255, 0x1b, 0x2d, 0x43), Variant::TonalSpot);
        let base = Palette::build(&theme, true, &Config::default());
        let mut cfg = Config::default();
        cfg.darken = 1.0;
        let dark = Palette::build(&theme, true, &cfg);
        assert!(tone(dark.window_bg) < tone(base.window_bg), "darken lowers the window");
        let light = Palette::build(&theme, false, &cfg);
        let light_base = Palette::build(&theme, false, &Config::default());
        assert_eq!(hex(light.window_bg), hex(light_base.window_bg), "darken is a dark-mode dial");
        let mut hb = Config::default();
        hb.headerbar = 1.0;
        let h = Palette::build(&theme, true, &hb);
        assert!(tone(h.headerbar_bg) < tone(base.headerbar_bg));
        assert_eq!(hex(h.window_bg), hex(base.window_bg), "headerbar leaves the window alone");
    }

    /// Every built-in terminal scheme yields 16 colours in both modes, and `blend` moves them
    /// towards the wallpaper without changing which one is the background.
    #[test]
    fn terminal_schemes_blend_towards_the_wallpaper() {
        let source = Argb::new(255, 0xd0, 0x40, 0x30);
        let theme = build_theme(source, Variant::TonalSpot);
        for scheme in crate::config::TermScheme::ALL {
            for dark in [true, false] {
                let mut none = Config::default();
                none.terminals.scheme = scheme;
                none.terminals.blend = 0.0;
                let mut full = none.clone();
                full.terminals.blend = 1.0;
                let a = Palette::build(&theme, dark, &none);
                let b = Palette::build(&theme, dark, &full);
                let tone = |c: Argb| Hct::new(c).get_tone();
                assert!((tone(a.term_bg) - tone(b.term_bg)).abs() < 12.0, "{scheme:?} dark={dark}: blend keeps the background tone");
                if scheme != crate::config::TermScheme::Flandre {
                    assert_ne!(a.ansi, b.ansi, "{scheme:?} dark={dark}: blend changes the palette");
                }
            }
        }
    }

    #[test]
    fn decodes_gnome_stock_jxl_wallpapers() {
        let stock = Path::new("/usr/share/backgrounds/gnome/adwaita-d.jxl");
        if !stock.is_file() {
            eprintln!("skipped: no stock JXL wallpaper on this machine");
            return;
        }
        let dir = std::env::temp_dir().join(format!("flandre-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Real JXL, JXL hiding behind a .jpg extension, and garbage: the first two must decode,
        // the last must fail with an error instead of aborting the process.
        let renamed = dir.join("renamed.jpg");
        std::fs::copy(stock, &renamed).unwrap();
        let garbage = dir.join("garbage.png");
        std::fs::write(&garbage, b"not an image").unwrap();

        let a = source_from_image(stock).expect("stock jxl");
        let b = source_from_image(&renamed).expect("jxl renamed to .jpg");
        assert_eq!(a, b);
        assert!(source_from_image(&garbage).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `shell.menus` takes the stock popover background to near black while keeping the
    /// hover/selected step above it, and `shell.accent` drives the highlight colour.
    #[test]
    fn shell_menus_and_accent() {
        let theme = build_theme(Argb::new(255, 0x1b, 0x2d, 0x43), Variant::Fidelity);
        let mut cfg = Config::default();
        let zero = Palette::build(&theme, true, &cfg);
        assert_eq!(zero.shell_accent, zero.primary);
        cfg.shell.menus = 1.0;
        cfg.shell.accent = AccentRole::Tertiary;
        let full = Palette::build(&theme, true, &cfg);
        let tone = |c: Argb| Hct::new(c).get_tone();
        assert!(tone(full.shell_menu_bg) < 4.0, "{:?}", full.shell_menu_bg);
        assert!(tone(full.shell_menu_bg) < tone(zero.shell_menu_bg));
        assert!(
            tone(full.shell_item_bg) - tone(full.shell_menu_bg) > 6.0,
            "steps kept apart"
        );
        assert_eq!(full.shell_accent, full.tertiary);
        assert_eq!(full.shell_on_accent, full.on_tertiary);
        // Tone 60 and above is untouched; pure black stays black.
        assert_eq!(full.shell_tone(60.0), 60.0);
        assert_eq!(full.recolor_shell(Argb::new(255, 0, 0, 0)), Argb::new(255, 0, 0, 0));
        // Light mode: a dim grey, never darker than tone 60.
        let light = Palette::build(&theme, false, &cfg);
        let t = tone(light.shell_menu_bg);
        assert!((70.0..80.0).contains(&t), "{t}");
        // Custom tone with readable text on top.
        cfg.shell.accent = AccentRole::Custom;
        cfg.shell.accent_tone = 30.0;
        let custom = Palette::build(&theme, true, &cfg);
        assert!((tone(custom.shell_accent) - 30.0).abs() < 1.5);
        assert!(tone(custom.shell_on_accent) > 90.0);
    }
}
