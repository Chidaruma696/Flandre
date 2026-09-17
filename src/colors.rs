//! Wallpaper -> Material scheme -> the concrete colours every target needs.
//!
//! The role mapping (Adwaita named colours <- Material roles) started from the
//! tables in adwaita-material-you by Francesco Caracciolo and was reworked so
//! the tint level can be dialled up.

use crate::config::{AccentRole, Icons, Terminals, Tint, Variant};
use crate::schemes;
use anyhow::{Context, Result, anyhow};
use material_colors::{
    blend,
    color::Argb,
    dynamic_color::Variant as MVariant,
    hct::Hct,
    image::{FilterType, ImageReader},
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
    let mut img = ImageReader::open(path).with_context(|| format!("reading {}", path.display()))?;
    img.resize(128, 128, FilterType::Lanczos3);
    Ok(ImageReader::extract_color(&img))
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
}

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
    pub fn build(theme: &Theme, dark: bool, tint: Tint, darken: f64, term: &Terminals) -> Self {
        let darken = darken.clamp(0.0, 1.0);
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
        let headerbar_bg = wash(high, p, 0.6 * wash_amount);
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

        Self {
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
        }
    }

    /// Icon colour: the scheme role the user picked, applied as is (like Material You did), with an
    /// optional chroma floor so pale schemes still give coloured folders.
    pub fn icon_accent(&self, icons: &Icons) -> Argb {
        let base = match icons.accent {
            AccentRole::PrimaryContainer => self.primary_container,
            AccentRole::Primary => self.primary,
            AccentRole::Secondary => self.secondary,
            AccentRole::Tertiary => self.tertiary,
            AccentRole::Custom => {
                let h = Hct::new(self.primary);
                Hct::from(h.get_hue(), h.get_chroma(), icons.tone.clamp(5.0, 95.0)).into()
            }
        };
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
    pub fn recolor(&self, c: Argb) -> Argb {
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
