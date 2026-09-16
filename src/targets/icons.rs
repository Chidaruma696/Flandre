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
fn walk(dir: &Path, out: &mut Vec<PathBuf>, links: &mut Vec<(PathBuf, PathBuf)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let ft = match e.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_symlink() {
            if p.is_dir()
                && let Ok(target) = std::fs::read_link(&p)
            {
                links.push((p, target));
            }
        } else if ft.is_dir() {
            walk(&p, out, links);
        } else if p.extension().is_some_and(|e| e == "svg") {
            out.push(p);
        }
    }
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
    let mut seen = std::collections::HashSet::new();
    let mut links: Vec<(PathBuf, PathBuf)> = Vec::new();
    for root in &roots {
        let mut files = Vec::new();
        let mut found_links = Vec::new();
        walk(root, &mut files, &mut found_links);
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
            if !seen.insert(rel.clone()) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&f) else {
                continue;
            };
            if !(text.contains(&source_accent) || text.contains(&source_accent_upper)) {
                continue;
            }
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
            let out = tmp.join(&rel);
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out, new)?;
            recolored += 1;
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
