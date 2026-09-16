//! Small helpers: paths, atomic writes, external commands, gsettings.

use anyhow::{Context, Result, anyhow};
use gio::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(fallback))
}

pub fn config_home() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config")
}

pub fn data_home() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share")
}

pub fn cache_home() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache")
}

pub fn flandre_config_dir() -> PathBuf {
    config_home().join("flandre")
}

pub fn flandre_cache_dir() -> PathBuf {
    cache_home().join("flandre")
}

/// Writes `content` to `path` through a temporary file so readers never see a half-written file.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&tmp, content).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("renaming into {}", path.display()))?;
    Ok(())
}

pub fn exists_in_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(bin).is_file()))
        .unwrap_or(false)
}

/// Runs a command, returning stdout on success.
pub fn run(bin: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("running {bin}"))?;
    if !out.status.success() {
        return Err(anyhow!(
            "{bin} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Runs a command, ignoring failures (used for best-effort steps).
pub fn run_quiet(bin: &str, args: &[&str]) -> bool {
    Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn flatpak_app_installed(id: &str) -> bool {
    home().join(".var/app").join(id).is_dir()
}

// ---------- gsettings through gio ----------

pub fn schema_exists(id: &str) -> bool {
    gio::SettingsSchemaSource::default()
        .and_then(|s| s.lookup(id, true))
        .is_some()
}

pub fn settings(id: &str) -> Option<gio::Settings> {
    schema_exists(id).then(|| gio::Settings::new(id))
}

pub fn settings_at(id: &str, path: &str) -> Option<gio::Settings> {
    schema_exists(id).then(|| gio::Settings::with_path(id, path))
}

pub fn get_string(id: &str, key: &str) -> Option<String> {
    settings(id).map(|s| s.string(key).to_string())
}

pub fn set_string(id: &str, key: &str, value: &str) -> Result<()> {
    let s = settings(id).ok_or_else(|| anyhow!("schema {id} not installed"))?;
    s.set_string(key, value).map_err(|e| anyhow!("{id} {key}: {e}"))?;
    gio::Settings::sync();
    Ok(())
}

/// Percent-decodes a `file://` URI into a path.
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(v) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?, 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    Some(PathBuf::from(String::from_utf8_lossy(&out).into_owned()))
}

pub fn notify(summary: &str, body: &str) {
    run_quiet(
        "notify-send",
        &["-a", "Flandre", "-i", "preferences-color-symbolic", summary, body],
    );
}
