//! `flandre settings`: a libadwaita window to preview and tune the theme before applying it.

use crate::apply;
use crate::colors::{self, Palette};
use crate::config::{AccentRole, Config, IconFamily, PanelStyle, TermScheme, Tint, Variant};
use crate::targets::icons as icons_target;
use crate::util;
use adw::prelude::*;
use anyhow::Result;
use gtk::{cairo, gdk, glib};
use material_colors::color::Argb;
use material_colors::theme::Theme;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "io.github.chidaruma696.Flandre";

/// Tiny i18n: Spanish when the session runs in Spanish, English otherwise.
fn t(en: &'static str, es: &'static str) -> &'static str {
    thread_local! {
        static ES: bool = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANG"))
            .map(|l| l.starts_with("es"))
            .unwrap_or(false);
    }
    if ES.with(|b| *b) { es } else { en }
}

struct State {
    cfg: Config,
    theme: Theme,
    source: Argb,
    wallpaper: Option<PathBuf>,
    preview_dark: bool,
    palette: Palette,
}

impl State {
    fn rebuild(&mut self) {
        self.theme = colors::build_theme(self.source, self.cfg.variant);
        self.palette = Palette::build(
            &self.theme,
            self.preview_dark,
            self.cfg.tint,
            self.cfg.darken,
            &self.cfg.terminals,
        );
    }
}

type Shared = Rc<RefCell<State>>;

fn rgb(cr: &cairo::Context, c: Argb) {
    cr.set_source_rgb(c.red as f64 / 255.0, c.green as f64 / 255.0, c.blue as f64 / 255.0);
}

fn rounded(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.arc(x + r, y + r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
    cr.close_path();
}

/// A mock desktop: top bar, a window with headerbar, sidebar, list rows, a button and a switch.
fn draw_desktop(cr: &cairo::Context, w: f64, h: f64, st: &State) {
    let p = &st.palette;
    // wallpaper-ish backdrop
    rgb(cr, p.source);
    cr.rectangle(0.0, 0.0, w, h);
    let _ = cr.fill();
    // top panel
    let a = st.cfg.shell.panel_opacity.clamp(0.0, 1.0);
    match st.cfg.shell.panel {
        PanelStyle::Black if p.dark => cr.set_source_rgba(0.0, 0.0, 0.0, a),
        PanelStyle::Black => {
            let c = p.surface;
            cr.set_source_rgba(c.red as f64 / 255.0, c.green as f64 / 255.0, c.blue as f64 / 255.0, a);
        }
        PanelStyle::Colored => {
            let c = p.surface_container;
            cr.set_source_rgba(c.red as f64 / 255.0, c.green as f64 / 255.0, c.blue as f64 / 255.0, a);
        }
        PanelStyle::Transparent => cr.set_source_rgba(0.0, 0.0, 0.0, 0.0),
    }
    cr.rectangle(0.0, 0.0, w, 18.0);
    let _ = cr.fill();
    rgb(cr, p.on_surface);
    cr.rectangle(w / 2.0 - 18.0, 7.0, 36.0, 4.0);
    let _ = cr.fill();
    rgb(cr, p.primary);
    cr.arc(w - 14.0, 9.0, 3.0, 0.0, 6.3);
    let _ = cr.fill();
    // window
    let (x, y, ww, wh) = (24.0, 34.0, w - 48.0, h - 46.0);
    rgb(cr, p.window_bg);
    rounded(cr, x, y, ww, wh, 10.0);
    let _ = cr.fill();
    // headerbar
    rgb(cr, p.headerbar_bg);
    rounded(cr, x, y, ww, 34.0, 10.0);
    let _ = cr.fill();
    rgb(cr, p.headerbar_bg);
    cr.rectangle(x, y + 20.0, ww, 14.0);
    let _ = cr.fill();
    rgb(cr, p.on_surface);
    cr.rectangle(x + ww / 2.0 - 30.0, y + 15.0, 60.0, 4.0);
    let _ = cr.fill();
    rgb(cr, p.on_surface_variant);
    cr.arc(x + ww - 16.0, y + 17.0, 5.0, 0.0, 6.3);
    let _ = cr.fill();
    // sidebar
    rgb(cr, p.sidebar_bg);
    cr.rectangle(x, y + 34.0, 90.0, wh - 34.0);
    let _ = cr.fill();
    for i in 0..5 {
        let ry = y + 44.0 + i as f64 * 20.0;
        if i == 1 {
            let a = p.primary;
            cr.set_source_rgba(a.red as f64 / 255.0, a.green as f64 / 255.0, a.blue as f64 / 255.0, 0.3);
            rounded(cr, x + 6.0, ry - 4.0, 78.0, 16.0, 5.0);
            let _ = cr.fill();
        }
        rgb(cr, if i == 1 { p.on_surface } else { p.on_surface_variant });
        cr.rectangle(x + 14.0, ry, 50.0 - i as f64 * 5.0, 4.0);
        let _ = cr.fill();
    }
    // content: a card with rows
    rgb(cr, p.view_bg);
    cr.rectangle(x + 90.0, y + 34.0, ww - 90.0, wh - 34.0);
    let _ = cr.fill();
    rgb(cr, p.card_bg);
    rounded(cr, x + 102.0, y + 46.0, ww - 124.0, 58.0, 8.0);
    let _ = cr.fill();
    for i in 0..3 {
        rgb(cr, p.on_surface);
        cr.rectangle(x + 112.0, y + 56.0 + i as f64 * 17.0, 70.0 + i as f64 * 8.0, 4.0);
        let _ = cr.fill();
    }
    // switch (on) inside the card
    rgb(cr, p.primary);
    rounded(cr, x + ww - 56.0, y + 54.0, 30.0, 14.0, 7.0);
    let _ = cr.fill();
    rgb(cr, p.on_primary);
    cr.arc(x + ww - 33.0, y + 61.0, 5.0, 0.0, 6.3);
    let _ = cr.fill();
    // buttons
    rgb(cr, p.primary);
    rounded(cr, x + 102.0, y + wh - 30.0, 70.0, 20.0, 6.0);
    let _ = cr.fill();
    rgb(cr, p.on_primary);
    cr.rectangle(x + 122.0, y + wh - 22.0, 30.0, 4.0);
    let _ = cr.fill();
    rgb(cr, p.surface_container_highest);
    rounded(cr, x + 180.0, y + wh - 30.0, 70.0, 20.0, 6.0);
    let _ = cr.fill();
    rgb(cr, p.on_surface);
    cr.rectangle(x + 200.0, y + wh - 22.0, 30.0, 4.0);
    let _ = cr.fill();
    // popover
    rgb(cr, p.popover_bg);
    rounded(cr, x + ww - 120.0, y + 112.0, 100.0, 60.0, 8.0);
    let _ = cr.fill();
    for i in 0..3 {
        rgb(cr, p.on_surface_variant);
        cr.rectangle(x + ww - 110.0, y + 124.0 + i as f64 * 16.0, 60.0, 4.0);
        let _ = cr.fill();
    }
}

fn draw_terminal(cr: &cairo::Context, w: f64, h: f64, st: &State) {
    let p = &st.palette;
    // "wallpaper" behind, so the opacity reads
    rgb(cr, p.source);
    rounded(cr, 0.0, 0.0, w, h, 8.0);
    let _ = cr.fill();
    let c = p.term_bg;
    cr.set_source_rgba(
        c.red as f64 / 255.0,
        c.green as f64 / 255.0,
        c.blue as f64 / 255.0,
        st.cfg.terminals.opacity.clamp(0.0, 1.0),
    );
    rounded(cr, 0.0, 0.0, w, h, 8.0);
    let _ = cr.fill();
    let cell = ((w - 16.0) / 8.0).min(28.0);
    for (i, c) in p.ansi.iter().enumerate() {
        let col = (i % 8) as f64;
        let row = (i / 8) as f64;
        rgb(cr, *c);
        rounded(
            cr,
            8.0 + col * cell,
            8.0 + row * (cell - 4.0),
            cell - 4.0,
            cell - 8.0,
            3.0,
        );
        let _ = cr.fill();
    }
    rgb(cr, p.term_fg);
    let ty = 8.0 + 2.0 * (cell - 4.0) + 6.0;
    cr.rectangle(8.0, ty, 90.0, 3.0);
    let _ = cr.fill();
    rgb(cr, p.cursor);
    cr.rectangle(104.0, ty - 3.0, 8.0, 9.0);
    let _ = cr.fill();
}

fn draw_swatches(cr: &cairo::Context, w: f64, _h: f64, st: &State) {
    let p = &st.palette;
    let items = [
        p.primary,
        p.primary_container,
        p.secondary,
        p.tertiary,
        p.surface,
        p.window_bg,
        p.headerbar_bg,
        p.popover_bg,
        p.on_surface,
        p.error,
    ];
    let n = items.len() as f64;
    let cw = w / n;
    for (i, c) in items.iter().enumerate() {
        rgb(cr, *c);
        cr.rectangle(i as f64 * cw, 0.0, cw, 28.0);
        let _ = cr.fill();
    }
}

/// Writes recoloured sample icons of the chosen family into the cache and returns their paths.
fn sample_icons(st: &State) -> Vec<PathBuf> {
    let Ok(family) = icons_target::resolve_family(st.cfg.icons.family) else {
        return Vec::new();
    };
    let Some(base) = icons_target::find_theme(family.base) else {
        return Vec::new();
    };
    let dir = ["scalable/places", "64x64/places", "48x48/places"]
        .iter()
        .map(|d| base.join(d))
        .find(|d| d.is_dir());
    let Some(dir) = dir else { return Vec::new() };
    let accent = st.palette.icon_accent(&st.cfg.icons);
    let out_dir = util::flandre_cache_dir().join("preview");
    let _ = std::fs::create_dir_all(&out_dir);
    let mut paths = Vec::new();
    for (i, name) in ["folder", "folder-documents", "user-home", "folder-pictures"]
        .iter()
        .enumerate()
    {
        let src = dir.join(format!("{name}.svg"));
        let Ok(text) = std::fs::read_to_string(&src) else {
            continue;
        };
        let new = icons_target::recolor_svg(&text, family, accent);
        let out = out_dir.join(format!("{i}-{}.svg", colors::hex(accent).trim_start_matches('#')));
        if std::fs::write(&out, new).is_ok() {
            paths.push(out);
        }
    }
    paths
}

struct Widgets {
    desktop: gtk::DrawingArea,
    terminal: gtk::DrawingArea,
    swatches: gtk::DrawingArea,
    icons_box: gtk::Box,
    accent_label: gtk::Label,
    tone_row: adw::ActionRow,
}

fn refresh(st: &Shared, w: &Widgets) {
    w.desktop.queue_draw();
    w.terminal.queue_draw();
    w.swatches.queue_draw();
    let s = st.borrow();
    while let Some(child) = w.icons_box.first_child() {
        w.icons_box.remove(&child);
    }
    for path in sample_icons(&s) {
        if let Ok(tex) = gdk::Texture::from_filename(&path) {
            let pic = gtk::Picture::for_paintable(&tex);
            pic.set_size_request(72, 72);
            pic.set_content_fit(gtk::ContentFit::Contain);
            w.icons_box.append(&pic);
        }
    }
    let accent = s.palette.icon_accent(&s.cfg.icons);
    w.accent_label.set_text(&format!(
        "{}  {}  ·  {} {}",
        t("Icons", "Iconos"),
        colors::hex(accent),
        t("accent", "acento"),
        colors::hex(s.palette.primary)
    ));
    w.tone_row.set_sensitive(s.cfg.icons.accent == AccentRole::Custom);
}

fn combo(title: &str, subtitle: &str, items: &[&str], selected: u32) -> adw::ComboRow {
    let row = adw::ComboRow::builder()
        .title(title)
        .subtitle(subtitle)
        .model(&gtk::StringList::new(items))
        .build();
    row.set_selected(selected);
    row
}

fn scale_row(title: &str, subtitle: &str, min: f64, max: f64, step: f64, value: f64) -> (adw::ActionRow, gtk::Scale) {
    let row = adw::ActionRow::builder().title(title).subtitle(subtitle).build();
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, min, max, step);
    scale.set_value(value);
    scale.set_draw_value(true);
    scale.set_size_request(200, -1);
    scale.set_valign(gtk::Align::Center);
    row.add_suffix(&scale);
    (row, scale)
}

fn build_ui(app: &adw::Application) {
    let cfg = Config::load().unwrap_or_default();
    let preview_dark = apply::prefers_dark();
    let wallpaper = apply::current_wallpaper(preview_dark).ok();
    let source = wallpaper
        .as_ref()
        .and_then(|w| colors::source_from_image(w).ok())
        .unwrap_or_else(|| Argb::new(255, 0x67, 0x50, 0xa4));
    let theme = colors::build_theme(source, cfg.variant);
    let palette = Palette::build(&theme, preview_dark, cfg.tint, cfg.darken, &cfg.terminals);
    let st: Shared = Rc::new(RefCell::new(State {
        cfg,
        theme,
        source,
        wallpaper,
        preview_dark,
        palette,
    }));

    // ---- preview column
    let preview = gtk::Box::new(gtk::Orientation::Vertical, 12);
    preview.set_margin_top(18);
    preview.set_margin_bottom(18);
    preview.set_margin_start(18);
    preview.set_margin_end(6);
    preview.set_size_request(380, -1);

    let wp = gtk::Picture::builder().content_fit(gtk::ContentFit::Cover).build();
    wp.set_size_request(-1, 110);
    wp.add_css_class("card");
    wp.set_overflow(gtk::Overflow::Hidden);
    if let Some(p) = &st.borrow().wallpaper {
        wp.set_filename(Some(p));
    }
    preview.append(&wp);

    let desktop = gtk::DrawingArea::new();
    desktop.set_size_request(-1, 240);
    desktop.add_css_class("card");
    desktop.set_overflow(gtk::Overflow::Hidden);
    {
        let st = st.clone();
        desktop.set_draw_func(move |_, cr, w, h| draw_desktop(cr, w as f64, h as f64, &st.borrow()));
    }
    preview.append(&desktop);

    let swatches = gtk::DrawingArea::new();
    swatches.set_size_request(-1, 28);
    swatches.add_css_class("card");
    swatches.set_overflow(gtk::Overflow::Hidden);
    {
        let st = st.clone();
        swatches.set_draw_func(move |_, cr, w, h| draw_swatches(cr, w as f64, h as f64, &st.borrow()));
    }
    preview.append(&swatches);

    let icons_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    icons_box.set_halign(gtk::Align::Center);
    preview.append(&icons_box);
    let accent_label = gtk::Label::new(None);
    accent_label.add_css_class("dim-label");
    accent_label.add_css_class("caption");
    preview.append(&accent_label);

    let terminal = gtk::DrawingArea::new();
    terminal.set_size_request(-1, 90);
    {
        let st = st.clone();
        terminal.set_draw_func(move |_, cr, w, h| draw_terminal(cr, w as f64, h as f64, &st.borrow()));
    }
    preview.append(&terminal);

    // ---- settings column
    let page = adw::PreferencesPage::new();

    let g_scheme = adw::PreferencesGroup::builder().title(t("Colours", "Colores")).build();
    let variants = [
        Variant::FruitSalad,
        Variant::Vibrant,
        Variant::Expressive,
        Variant::TonalSpot,
        Variant::Rainbow,
        Variant::Fidelity,
        Variant::Content,
        Variant::Neutral,
        Variant::Monochrome,
    ];
    let variant_names = [
        "Fruit Salad",
        "Vibrant",
        "Expressive",
        "Tonal Spot",
        "Rainbow",
        "Fidelity",
        "Content",
        "Neutral",
        "Monochrome",
    ];
    let cur_variant = variants.iter().position(|v| *v == st.borrow().cfg.variant).unwrap_or(0) as u32;
    let variant_row = combo(
        t("Scheme", "Esquema"),
        t(
            "How the wallpaper colour becomes a palette",
            "Cómo el color del fondo se convierte en paleta",
        ),
        &variant_names,
        cur_variant,
    );
    let tints = [Tint::Soft, Tint::Normal, Tint::Strong];
    let cur_tint = tints.iter().position(|v| *v == st.borrow().cfg.tint).unwrap_or(2) as u32;
    let tint_row = combo(
        t("Tint", "Tinte"),
        t(
            "How much wallpaper hue goes into greys, headerbars and sidebars",
            "Cuánto matiz del fondo entra en grises, headerbars y sidebars",
        ),
        &[t("Soft", "Suave"), t("Normal", "Normal"), t("Strong", "Fuerte")],
        cur_tint,
    );
    let (darken_row, darken_scale) = scale_row(
        t("Darkness", "Oscuridad"),
        t(
            "Dark mode only: 0 = Material tones, 1 = near black",
            "Solo modo oscuro: 0 = tonos Material, 1 = casi negro",
        ),
        0.0,
        1.0,
        0.05,
        st.borrow().cfg.darken,
    );
    let preview_light = adw::SwitchRow::builder()
        .title(t("Preview light mode", "Previsualizar modo claro"))
        .active(!preview_dark)
        .build();
    g_scheme.add(&variant_row);
    g_scheme.add(&tint_row);
    g_scheme.add(&darken_row);
    g_scheme.add(&preview_light);
    page.add(&g_scheme);

    let g_icons = adw::PreferencesGroup::builder().title(t("Icons", "Iconos")).build();
    let families = [IconFamily::Auto, IconFamily::Tela, IconFamily::Papirus];
    let cur_family = families
        .iter()
        .position(|v| *v == st.borrow().cfg.icons.family)
        .unwrap_or(0) as u32;
    let installed: Vec<String> = icons_target::FAMILIES
        .iter()
        .map(|f| {
            if icons_target::family_installed(f) {
                f.name.to_string()
            } else {
                format!("{} ({})", f.name, t("not installed", "no instalado"))
            }
        })
        .collect();
    let family_items: Vec<&str> = std::iter::once(t("Auto", "Automático"))
        .chain(installed.iter().map(String::as_str))
        .collect();
    let family_row = combo(
        t("Icon family", "Familia de iconos"),
        t(
            "Recoloured on top of the installed theme",
            "Se recolorea encima del tema instalado",
        ),
        &family_items,
        cur_family,
    );
    let roles = [
        AccentRole::PrimaryContainer,
        AccentRole::Primary,
        AccentRole::Secondary,
        AccentRole::Tertiary,
        AccentRole::Custom,
    ];
    let cur_role = roles
        .iter()
        .position(|v| *v == st.borrow().cfg.icons.accent)
        .unwrap_or(0) as u32;
    let role_row = combo(
        t("Icon colour", "Color de los iconos"),
        t(
            "Which colour of the scheme paints folders",
            "Qué color del esquema pinta las carpetas",
        ),
        &[
            t(
                "Primary container (deep, like Material You)",
                "Primary container (profundo, como Material You)",
            ),
            t("Primary (the accent)", "Primary (el acento)"),
            t("Secondary", "Secondary"),
            t("Tertiary", "Tertiary"),
            t("Custom tone", "Tono a medida"),
        ],
        cur_role,
    );
    let (tone_row, tone_scale) = scale_row(
        t("Tone", "Tono"),
        t(
            "0 = black, 100 = white (custom only)",
            "0 = negro, 100 = blanco (solo a medida)",
        ),
        5.0,
        95.0,
        1.0,
        st.borrow().cfg.icons.tone,
    );
    let (chroma_row, chroma_scale) = scale_row(
        t("Minimum colourfulness", "Saturación mínima"),
        t("0 keeps the scheme's own", "0 deja la del esquema"),
        0.0,
        80.0,
        1.0,
        st.borrow().cfg.icons.chroma,
    );
    g_icons.add(&family_row);
    g_icons.add(&role_row);
    g_icons.add(&tone_row);
    g_icons.add(&chroma_row);
    page.add(&g_icons);

    let g_shell = adw::PreferencesGroup::builder().title("GNOME Shell").build();
    let panels = [PanelStyle::Black, PanelStyle::Colored, PanelStyle::Transparent];
    let cur_panel = panels
        .iter()
        .position(|v| *v == st.borrow().cfg.shell.panel)
        .unwrap_or(0) as u32;
    let panel_row = combo(
        t("Top bar", "Barra superior"),
        t(
            "Colour of the panel outside the overview",
            "Color del panel fuera de la vista general",
        ),
        &[
            t("Stock (black in dark mode)", "Como GNOME (negra en oscuro)"),
            t("Coloured", "De color"),
            t("Transparent", "Transparente"),
        ],
        cur_panel,
    );
    let (opacity_row, opacity_scale) = scale_row(
        t("Top bar opacity", "Opacidad de la barra"),
        t("0 = see-through, 1 = solid", "0 = se ve el fondo, 1 = sólida"),
        0.0,
        1.0,
        0.05,
        st.borrow().cfg.shell.panel_opacity,
    );
    opacity_row.set_sensitive(st.borrow().cfg.shell.panel != PanelStyle::Transparent);
    g_shell.add(&panel_row);
    g_shell.add(&opacity_row);
    page.add(&g_shell);

    let g_term = adw::PreferencesGroup::builder()
        .title(t("Terminals", "Terminales"))
        .build();
    let (term_opacity_row, term_opacity_scale) = scale_row(
        t("Terminal opacity", "Opacidad de la terminal"),
        t(
            "Ptyxis, Console and Black Box background",
            "Fondo de Ptyxis, Console y Black Box",
        ),
        0.0,
        1.0,
        0.05,
        st.borrow().cfg.terminals.opacity,
    );
    let schemes_all = TermScheme::ALL;
    let scheme_labels: Vec<&str> = schemes_all.iter().map(|s| s.label()).collect();
    let cur_scheme = schemes_all
        .iter()
        .position(|v| *v == st.borrow().cfg.terminals.scheme)
        .unwrap_or(0) as u32;
    let scheme_row = combo(
        t("Colour scheme", "Esquema de colores"),
        t(
            "Flandre is built from the wallpaper scheme; the rest are classic palettes pulled towards it",
            "Flandre sale del esquema del fondo; el resto son paletas clásicas acercadas a él",
        ),
        &scheme_labels,
        cur_scheme,
    );
    let (blend_row, blend_scale) = scale_row(
        t("Blend with wallpaper", "Fusión con el fondo"),
        t(
            "Classic schemes only: 0 = as published, 1 = fully harmonised",
            "Solo esquemas clásicos: 0 = tal cual, 1 = del todo armonizado",
        ),
        0.0,
        1.0,
        0.05,
        st.borrow().cfg.terminals.blend,
    );
    blend_row.set_sensitive(st.borrow().cfg.terminals.scheme != TermScheme::Flandre);
    g_term.add(&scheme_row);
    g_term.add(&blend_row);
    g_term.add(&term_opacity_row);
    page.add(&g_term);

    let g_targets = adw::PreferencesGroup::builder()
        .title(t("Apply to", "Aplicar a"))
        .build();
    let mk = |title: &str, on: bool| adw::SwitchRow::builder().title(title).active(on).build();
    let tg = st.borrow().cfg.targets.clone();
    let sw_gtk = mk("GTK / libadwaita", tg.gtk);
    let sw_shell = mk("GNOME Shell", tg.shell);
    let sw_icons = mk(t("Icons", "Iconos"), tg.icons);
    let sw_ptyxis = mk("Ptyxis", tg.ptyxis);
    let sw_console = mk("GNOME Console", tg.console);
    let sw_blackbox = mk("Black Box", tg.blackbox);
    let sw_terms = mk(t("Open terminals (OSC)", "Terminales abiertas (OSC)"), tg.terminals);
    let sw_notify = mk(
        t("Notification after applying", "Notificación al aplicar"),
        st.borrow().cfg.notify,
    );
    for r in [
        &sw_gtk,
        &sw_shell,
        &sw_icons,
        &sw_ptyxis,
        &sw_console,
        &sw_blackbox,
        &sw_terms,
        &sw_notify,
    ] {
        g_targets.add(r);
    }
    page.add(&g_targets);

    // ---- layout
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    content.append(&preview);
    page.set_hexpand(true);
    content.append(&page);

    let header = adw::HeaderBar::new();
    let apply_btn = gtk::Button::with_label(t("Apply", "Aplicar"));
    apply_btn.add_css_class("suggested-action");
    header.pack_end(&apply_btn);
    let tv = adw::ToolbarView::new();
    tv.add_top_bar(&header);
    tv.set_content(Some(&content));
    let overlay = adw::ToastOverlay::new();
    overlay.set_child(Some(&tv));
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Flandre")
        .default_width(1040)
        .default_height(720)
        .content(&overlay)
        .build();

    let widgets = Rc::new(Widgets {
        desktop,
        terminal,
        swatches,
        icons_box,
        accent_label,
        tone_row,
    });
    refresh(&st, &widgets);

    // ---- wiring
    macro_rules! on_change {
        ($widget:expr, $signal:ident, |$s:ident, $w:ident| $body:expr) => {{
            let st2 = st.clone();
            let wd = widgets.clone();
            $widget.$signal(move |$w| {
                {
                    let mut $s = st2.borrow_mut();
                    $body;
                    $s.rebuild();
                }
                refresh(&st2, &wd);
            });
        }};
    }
    on_change!(variant_row, connect_selected_notify, |s, w| s.cfg.variant =
        variants[w.selected() as usize]);
    on_change!(tint_row, connect_selected_notify, |s, w| s.cfg.tint =
        tints[w.selected() as usize]);
    on_change!(darken_scale, connect_value_changed, |s, w| s.cfg.darken = w.value());
    on_change!(preview_light, connect_active_notify, |s, w| s.preview_dark =
        !w.is_active());
    on_change!(family_row, connect_selected_notify, |s, w| s.cfg.icons.family =
        families[w.selected() as usize]);
    on_change!(role_row, connect_selected_notify, |s, w| s.cfg.icons.accent =
        roles[w.selected() as usize]);
    on_change!(tone_scale, connect_value_changed, |s, w| s.cfg.icons.tone = w.value());
    on_change!(chroma_scale, connect_value_changed, |s, w| s.cfg.icons.chroma =
        w.value());
    {
        let opacity_row = opacity_row.clone();
        panel_row.connect_selected_notify(move |w| opacity_row.set_sensitive(w.selected() != 2));
    }
    on_change!(panel_row, connect_selected_notify, |s, w| s.cfg.shell.panel =
        panels[w.selected() as usize]);
    on_change!(opacity_scale, connect_value_changed, |s, w| s.cfg.shell.panel_opacity =
        w.value());
    {
        let blend_row = blend_row.clone();
        scheme_row.connect_selected_notify(move |w| blend_row.set_sensitive(w.selected() != 0));
    }
    on_change!(scheme_row, connect_selected_notify, |s, w| s.cfg.terminals.scheme =
        schemes_all[w.selected() as usize]);
    on_change!(blend_scale, connect_value_changed, |s, w| s.cfg.terminals.blend =
        w.value());
    on_change!(term_opacity_scale, connect_value_changed, |s, w| s
        .cfg
        .terminals
        .opacity =
        w.value());
    on_change!(sw_gtk, connect_active_notify, |s, w| s.cfg.targets.gtk = w.is_active());
    on_change!(sw_shell, connect_active_notify, |s, w| s.cfg.targets.shell =
        w.is_active());
    on_change!(sw_icons, connect_active_notify, |s, w| s.cfg.targets.icons =
        w.is_active());
    on_change!(sw_ptyxis, connect_active_notify, |s, w| s.cfg.targets.ptyxis =
        w.is_active());
    on_change!(sw_console, connect_active_notify, |s, w| s.cfg.targets.console =
        w.is_active());
    on_change!(sw_blackbox, connect_active_notify, |s, w| s.cfg.targets.blackbox =
        w.is_active());
    on_change!(sw_terms, connect_active_notify, |s, w| s.cfg.targets.terminals =
        w.is_active());
    on_change!(sw_notify, connect_active_notify, |s, w| s.cfg.notify = w.is_active());

    {
        let st = st.clone();
        let overlay = overlay.clone();
        apply_btn.connect_clicked(move |btn| {
            if let Err(e) = st.borrow().cfg.save() {
                overlay.add_toast(adw::Toast::new(&format!(
                    "{}: {e:#}",
                    t("Could not save", "No se pudo guardar")
                )));
                return;
            }
            btn.set_sensitive(false);
            let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("flandre"));
            let btn = btn.clone();
            let overlay = overlay.clone();
            glib::spawn_future_local(async move {
                let result = gio::Subprocess::newv(
                    &[
                        exe.as_os_str(),
                        "apply".as_ref(),
                        "--force".as_ref(),
                        "--quiet".as_ref(),
                    ],
                    gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_PIPE,
                );
                let msg = match result {
                    Ok(sp) => match sp.communicate_utf8_future(None).await {
                        Ok((_out, err)) if sp.is_successful() => {
                            let warnings = err.map(|e| e.to_string()).unwrap_or_default();
                            if warnings.trim().is_empty() {
                                t("Theme applied", "Tema aplicado").to_string()
                            } else {
                                format!(
                                    "{}: {}",
                                    t("Applied with warnings", "Aplicado con avisos"),
                                    warnings.trim()
                                )
                            }
                        }
                        Ok((_out, err)) => format!(
                            "{}: {}",
                            t("Apply failed", "Falló al aplicar"),
                            err.map(|e| e.to_string()).unwrap_or_default().trim()
                        ),
                        Err(e) => format!("{}: {e}", t("Apply failed", "Falló al aplicar")),
                    },
                    Err(e) => format!("{}: {e}", t("Could not start", "No se pudo lanzar")),
                };
                let toast = adw::Toast::new(&msg);
                toast.set_timeout(6);
                overlay.add_toast(toast);
                btn.set_sensitive(true);
            });
        });
    }

    window.present();

    // Debug aid: FLANDRE_SNAPSHOT=/path.png saves a picture of the window two seconds after it opens.
    if let Ok(path) = std::env::var("FLANDRE_SNAPSHOT") {
        let paintable = gtk::WidgetPaintable::new(Some(&overlay));
        let window = window.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(2000), move || {
            if let Some(image) = paintable
                .current_image()
                .downcast_ref::<gdk::Texture>()
                .cloned()
                .or_else(|| {
                    let snap = gtk::Snapshot::new();
                    paintable.current_image().snapshot(
                        &snap,
                        paintable.intrinsic_width() as f64,
                        paintable.intrinsic_height() as f64,
                    );
                    snap.to_node().and_then(|node| {
                        let renderer = window.native().and_then(|n| n.renderer())?;
                        Some(renderer.render_texture(&node, None))
                    })
                })
            {
                let _ = image.save_to_png(&path);
            }
            window.close();
        });
    }
}

pub fn run() -> Result<()> {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run_with_args::<&str>(&[]);
    Ok(())
}
