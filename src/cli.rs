use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use crate::model::Category;

#[derive(Parser, Debug)]
#[command(
    name = "hyprconf",
    version,
    about = "Search and edit Hypr config files"
)]
pub struct Cli {
    /// Root directory of Hypr configuration (default: ~/.config/hypr)
    #[arg(short, long, value_name = "DIR")]
    pub root: Option<PathBuf>,

    /// Pre-filter to a specific category
    #[arg(long, value_enum)]
    pub category: Option<Category>,

    /// Editor to open the file with (default: hx)
    #[arg(long, value_name = "CMD")]
    pub editor: Option<String>,

    /// Skim color scheme, e.g. "dark", "light", "none", or a custom spec
    /// like: dark,current_bg:24,matched:#00FF00
    #[arg(long = "color", value_name = "SPEC")]
    pub color_spec: Option<String>,

    /// Disable per-line segment colors (category/alias/description coloring)
    #[arg(long)]
    pub no_seg_colors: bool,
}

impl Cli {
    pub fn resolve_root(&self) -> Result<PathBuf> {
        if let Some(dir) = &self.root {
            return Ok(dir.clone());
        }
        default_root_from_xdg()
    }
}

fn default_root_from_xdg() -> Result<PathBuf> {
    // Prefer XDG_CONFIG_HOME if set
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME")
        && !xdg.trim().is_empty()
    {
        return Ok(PathBuf::from(xdg).join("hypr"));
    }
    // Fallback to ~/.config/hypr
    let home = env::var("HOME").context("HOME not set; cannot resolve default root")?;
    Ok(PathBuf::from(home).join(".config").join("hypr"))
}
