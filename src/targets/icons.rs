//! Icon theme: builds `Flandre-<hex>` on top of an installed Tela family, recolouring every SVG
//! that carries the family's accent (folders, places, devices, the coloured actions...) and
//! inheriting the rest. Same idea as Nyarch's Tela trick, without the 118 MB clone.

use crate::colors::{Palette, hex};
use crate::config::Icons;
use crate::util;
use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};

const ICON_SCHEMA: &str = "org.gnome.desktop.interface";

fn icon_dirs() -> Vec<PathBuf> {
    let mut v = vec![util::data_home().join("icons"), util::home().join(".icons")];
    let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    for d in data_dirs.split(':').filter(|d| !d.is_empty()) {
        v.push(PathBuf::from(d).join("icons"));
    }
    v
}

fn find_theme(name: &str) -> Option<PathBuf> {
    icon_dirs()
        .into_iter()
        .map(|d| d.join(name))
        .find(|p| p.join("index.theme").is_file())
}

/// Collects SVG files and symlinked directories (Tela uses `16@2x -> 16`, and the dark/light
/// variants point `scalable` at the base family). Symlinks are not followed: the base family is
/// walked on its own, and in-theme links are recreated in the output.
fn walk(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    links: &mut Vec<(PathBuf, PathBuf)>,
    file_links: &mut Vec<(PathBuf, PathBuf)>,
) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let ft = match e.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_symlink() {
            if let Ok(target) = std::fs::read_link(&p) {
                if p.is_dir() {
                    links.push((p, target));
                } else if p.extension().is_some_and(|e| e == "svg") {
                    // Tela names most icons through symlinks (folder.svg -> default-folder.svg).
                    file_links.push((p, target));
                }
            }
        } else if ft.is_dir() {
            walk(&p, out, links, file_links);
        } else if p.extension().is_some_and(|e| e == "svg") {
            out.push(p);
        }
    }
}

fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

pub struct IconResult {
    pub name: String,
    pub dir: PathBuf,
    pub recolored: usize,
}

pub fn apply(p: &Palette, cfg: &Icons) -> Result<IconResult> {
    let variant_name = format!("{}-{}", cfg.base, if p.dark { "dark" } else { "light" });
    let variant = find_theme(&variant_name);
    let base = find_theme(&cfg.base)
        .ok_or_else(|| anyhow!("icon theme {} not installed (looked in XDG icon dirs)", cfg.base))?;
    let accent = p.icon_accent();
    let accent_hex = hex(accent);
    let name = format!("Flandre-{}", &accent_hex[1..]);
    let dest = util::data_home().join("icons").join(&name);
    if dest.join("index.theme").is_file() {
        // Already built for this exact accent; just make sure it is selected.
        select(&name)?;
        return Ok(IconResult {
            name,
            dir: dest,
            recolored: 0,
        });
    }
    let tmp = dest.with_extension("building");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;

    let source_accent = cfg.source_accent.to_ascii_lowercase();
    let source_accent_upper = source_accent.to_ascii_uppercase();
    let mut recolored = 0usize;
    // Variant first (dark/light specific files win), then the base family.
    let mut roots: Vec<&Path> = Vec::new();
    if let Some(v) = &variant {
        roots.push(v);
    }
    roots.push(&base);
    // Gather every SVG (regular files and symlinked names) from the variant and the base family,
    // deduplicated by path inside the theme (variant wins).
    let mut seen = std::collections::HashSet::new();
    let mut links: Vec<(PathBuf, PathBuf)> = Vec::new();
    // (source path, path inside the theme, link target if it is a symlink)
    let mut entries: Vec<(PathBuf, PathBuf, Option<PathBuf>)> = Vec::new();
    for root in &roots {
        let mut files = Vec::new();
        let mut found_links = Vec::new();
        let mut found_file_links = Vec::new();
        walk(root, &mut files, &mut found_links, &mut found_file_links);
        for (link, target) in found_links {
            let rel = link.strip_prefix(root).unwrap_or(&link).to_path_buf();
            // Only links that stay inside the theme (e.g. 16@2x -> 16); cross-theme ones are
            // covered by walking the base family itself.
            if target.is_relative()
                && !target
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                links.push((rel, target));
            }
        }
        for f in files {
            let rel = f.strip_prefix(root).unwrap_or(&f).to_path_buf();
            if seen.insert(rel.clone()) {
                entries.push((f, rel, None));
            }
        }
        for (link, target) in found_file_links {
            let rel = link.strip_prefix(root).unwrap_or(&link).to_path_buf();
            if seen.insert(rel.clone()) {
                entries.push((link, rel, Some(target)));
            }
        }
    }

    let has_accent = |text: &str| text.contains(&source_accent) || text.contains(&source_accent_upper);
    let recolor = |text: &str| {
        let mut new = text
            .replace(&source_accent, &accent_hex)
            .replace(&source_accent_upper, &accent_hex);
        // Tela's KDE colour-scheme hooks: paint the highlight class too.
        if new.contains("ColorScheme-Highlight") {
            new = new.replace(
                "ColorScheme-Highlight\" style=\"color:currentColor",
                &format!("ColorScheme-Highlight\" style=\"color:{accent_hex}"),
            );
        }
        new
    };
    // Icon name (category/file) -> whether some size of it carries the accent. GTK prefers the
    // theme's own directories over inherited ones even at other sizes, so once one size of a name
    // is in the theme every size must be, or a 24px icon ends up scaled to 64px.
    let key = |rel: &Path| -> Option<(String, String)> {
        let name = rel.file_name()?.to_string_lossy().into_owned();
        let category = rel.parent()?.file_name()?.to_string_lossy().into_owned();
        Some((category, name))
    };
    let mut accented_names = std::collections::HashSet::new();
    let mut contents: Vec<Option<String>> = Vec::with_capacity(entries.len());
    for (src, rel, _) in &entries {
        let text = std::fs::read_to_string(src).ok();
        if text.as_deref().is_some_and(has_accent)
            && let Some(k) = key(rel)
        {
            accented_names.insert(k);
        }
        contents.push(text);
    }
    let mut copied = std::collections::HashSet::new();
    let mut pending_links: Vec<(PathBuf, PathBuf, Option<String>)> = Vec::new();
    for ((src, rel, target), text) in entries.iter().zip(contents) {
        let Some(text) = text else { continue };
        let wanted = key(rel).is_some_and(|k| accented_names.contains(&k));
        if !wanted {
            continue;
        }
        if let Some(target) = target {
            pending_links.push((rel.clone(), target.clone(), Some(text)));
            continue;
        }
        let out = tmp.join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if has_accent(&text) {
            std::fs::write(&out, recolor(&text))?;
            recolored += 1;
        } else {
            std::fs::copy(src, &out)?;
        }
        copied.insert(rel.clone());
    }
    // Symlinked names: link again when the target is in the theme, otherwise materialise the
    // resolved file (recoloured if it carries the accent) under the link's own name.
    for (rel, target, text) in pending_links {
        let target_rel = normalize(&rel.parent().unwrap_or(Path::new("")).join(&target));
        let out = tmp.join(&rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if target.is_relative() && copied.contains(&target_rel) {
            let _ = std::os::unix::fs::symlink(&target, &out);
            continue;
        }
        let Some(text) = text else { continue };
        if has_accent(&text) {
            std::fs::write(&out, recolor(&text))?;
            recolored += 1;
        } else {
            std::fs::write(&out, text)?;
        }
    }

    for (rel, target) in links {
        let link = tmp.join(&rel);
        if link.exists() || link.symlink_metadata().is_ok() {
            continue;
        }
        if let Some(parent) = link.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if tmp.join(rel.parent().unwrap_or(Path::new(""))).join(&target).is_dir() {
            let _ = std::os::unix::fs::symlink(&target, &link);
        }
    }

    // index.theme: take the variant's (or base's) directory list, rename, and inherit the family.
    let index_src = variant.as_ref().unwrap_or(&base).join("index.theme");
    let index = std::fs::read_to_string(&index_src).with_context(|| format!("reading {}", index_src.display()))?;
    let mut inherits = Vec::new();
    if variant.is_some() {
        inherits.push(variant_name.clone());
    }
    inherits.push(cfg.base.clone());
    let mut out = String::new();
    for line in index.lines() {
        if line.starts_with("Name=") {
            out.push_str(&format!("Name={name}\n"));
        } else if line.starts_with("Comment=") {
            out.push_str("Comment=Tela recoloured by Flandre from the wallpaper\n");
        } else if line.starts_with("Inherits=") {
            let old = line.trim_start_matches("Inherits=");
            let mut all = inherits.clone();
            for o in old.split(',').map(str::trim).filter(|o| !o.is_empty()) {
                if !all.iter().any(|a| a == o) {
                    all.push(o.to_string());
                }
            }
            out.push_str(&format!("Inherits={}\n", all.join(",")));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !out.contains("Inherits=") {
        out = out.replacen(
            "[Icon Theme]\n",
            &format!("[Icon Theme]\nInherits={}\n", inherits.join(",")),
            1,
        );
    }
    std::fs::write(tmp.join("index.theme"), out)?;

    let _ = std::fs::remove_dir_all(&dest);
    std::fs::rename(&tmp, &dest)?;
    if util::exists_in_path("gtk-update-icon-cache") {
        util::run_quiet(
            "gtk-update-icon-cache",
            &["-f", "-q", dest.to_str().unwrap_or_default()],
        );
    }
    select(&name)?;
    cleanup_old(&name);
    Ok(IconResult {
        name,
        dir: dest,
        recolored,
    })
}

fn select(name: &str) -> Result<()> {
    util::set_string(ICON_SCHEMA, "icon-theme", name)
}

/// Removes previous Flandre-* builds (and the old Tela-MaterialYou-* ones from the Nyarch script).
fn cleanup_old(keep: &str) {
    let dir = util::data_home().join("icons");
    let Ok(rd) = std::fs::read_dir(&dir) else { return };
    for e in rd.flatten() {
        let n = e.file_name().to_string_lossy().into_owned();
        if (n.starts_with("Flandre-") || n.starts_with("Tela-MaterialYou-")) && n != keep && !n.ends_with(".building") {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}
