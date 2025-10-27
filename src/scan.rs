use std::{fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};

use crate::model::{Category, ConfigEntry};

const COMMENT_PREFIXES: &[&str] = &["#", "//", ";"]; // common comment styles

fn first_comment_line(path: &Path, max_lines: usize) -> Result<Option<String>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };
    let reader = BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        if i >= max_lines { break; }
        let Ok(line) = line else { continue };
        let trimmed = line.trim();
        // Skip shebangs like #!/bin/bash
        if trimmed.starts_with("#!") { continue; }
        if let Some(prefix) = COMMENT_PREFIXES.iter().find(|p| trimmed.starts_with(**p)) {
            let content = trimmed.trim_start_matches(prefix).trim().to_string();
            if !content.is_empty() {
                return Ok(Some(content));
            }
        }
    }
    Ok(None)
}

fn alias_from_conf_d(file_stem: &str) -> String {
    // Examples: "70-binds" -> "binds", "00-env" -> "env"
    if let Some((_num, rest)) = file_stem.split_once('-') {
        rest.to_string()
    } else {
        file_stem.to_string()
    }
}

fn entry_for_path(path: PathBuf, category: Category) -> Result<ConfigEntry> {
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let (alias, mut desc) = match category {
        Category::Hyprland => {
            let alias = "hyprland".to_string();
            let desc = first_comment_line(&path, 10)?;
            (alias, desc)
        }
        Category::Utility => {
            let alias = stem.clone();
            let desc = first_comment_line(&path, 20)?;
            (alias, desc)
        }
        Category::ConfD => {
            let alias = alias_from_conf_d(&stem);
            let desc = first_comment_line(&path, 10)?;
            (alias, desc)
        }
        Category::Themes => {
            let alias = stem.clone();
            let desc = first_comment_line(&path, 10)?;
            (alias, desc)
        }
        Category::Plugins => {
            let alias = stem.clone();
            let desc = first_comment_line(&path, 10)?;
            (alias, desc)
        }
        Category::Scripts => {
            // Use file stem if available, else the full file name
            let alias = stem.clone();
            let desc = first_comment_line(&path, 10)?;
            (alias, desc)
        }
    };

    // If the description starts with the alias (e.g., "binds - ..."), strip it to avoid duplication
    if let Some(d) = &desc {
        let stripped = strip_alias_prefix(&alias, d);
        desc = Some(stripped);
    }

    Ok(ConfigEntry {
        path,
        file_name,
        alias,
        description: desc,
        category,
    })
}

fn strip_alias_prefix(alias: &str, desc: &str) -> String {
    let trimmed = desc.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let alias_lc = alias.trim().to_lowercase();
    let mut s = trimmed;
    // Case-insensitive match of leading alias
    if s.len() >= alias.len() && s[..alias.len()].to_lowercase() == alias_lc {
        s = &s[alias.len()..];
        // Drop common separators and spaces: " - ", " — ", ": ", etc.
        s = s.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '—' | '–' | ':' | '|' ));
        return s.trim_start().to_string();
    }
    trimmed.to_string()
}

pub fn scan_configs(root: &Path) -> Result<Vec<ConfigEntry>> {
    let mut out: Vec<ConfigEntry> = Vec::new();

    // 1) hyprland.conf
    let hyprland = root.join("hyprland.conf");
    if hyprland.exists() {
        out.push(entry_for_path(hyprland, Category::Hyprland)?);
    }

    // 2) Utilities: hyprpaper.conf, hyprlock.conf, hypridle.conf
    for util in ["hyprpaper.conf", "hyprlock.conf", "hypridle.conf"] {
        let p = root.join(util);
        if p.exists() {
            out.push(entry_for_path(p, Category::Utility)?);
        }
    }

    // 3) conf.d/*.conf
    let confd = root.join("conf.d");
    if confd.is_dir() {
        for entry in std::fs::read_dir(&confd).with_context(|| format!("reading {}", confd.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "conf").unwrap_or(false) {
                out.push(entry_for_path(path, Category::ConfD)?);
            }
        }
    }

    // 4) themes/*.conf
    let themes = root.join("themes");
    if themes.is_dir() {
        for entry in std::fs::read_dir(&themes).with_context(|| format!("reading {}", themes.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "conf").unwrap_or(false) {
                // alias = file stem
                let file_name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                let desc = first_comment_line(&path, 10)?;
                out.push(ConfigEntry { path, file_name, alias: stem, description: desc, category: Category::Themes });
            }
        }
    }

    // 5) plugins/*.conf
    let plugins = root.join("plugins");
    if plugins.is_dir() {
        for entry in std::fs::read_dir(&plugins).with_context(|| format!("reading {}", plugins.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "conf").unwrap_or(false) {
                let file_name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                let desc = first_comment_line(&path, 10)?;
                out.push(ConfigEntry { path, file_name, alias: stem, description: desc, category: Category::Plugins });
            }
        }
    }

    // 6) scripts/* (executable files)
    let scripts = root.join("scripts");
    if scripts.is_dir() {
        for entry in std::fs::read_dir(&scripts).with_context(|| format!("reading {}", scripts.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let meta = std::fs::metadata(&path)?;
                #[cfg(unix)]
                let is_exec = (meta.permissions().mode() & 0o111) != 0;
                #[cfg(not(unix))]
                let is_exec = true; // on non-unix, include files
                if is_exec {
                    let file_name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or(file_name.clone()).to_string();
                    let desc = first_comment_line(&path, 10)?;
                    out.push(ConfigEntry { path, file_name, alias: stem, description: desc, category: Category::Scripts });
                }
            }
        }
    }

    // Exclude everything else by design

    // Stable ordering: category order, then alias
    out.sort_by_key(|e| e.sort_key());
    Ok(out)
}
