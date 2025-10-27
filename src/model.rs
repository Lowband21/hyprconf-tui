use std::{fmt, path::PathBuf};

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Category {
    Hyprland,
    Utility,
    Themes,
    Plugins,
    ConfD,
    Scripts,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Hyprland => write!(f, "hyprland"),
            Category::Utility => write!(f, "utility"),
            Category::Themes => write!(f, "themes"),
            Category::Plugins => write!(f, "plugins"),
            Category::ConfD => write!(f, "conf.d"),
            Category::Scripts => write!(f, "scripts"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub path: PathBuf,
    pub file_name: String,
    pub alias: String,
    pub description: Option<String>,
    pub category: Category,
}

impl ConfigEntry {
    pub fn sort_key(&self) -> (u8, String) {
        let cat_order = match self.category {
            Category::Hyprland => 0,
            Category::Utility => 1,
            Category::Themes => 2,
            Category::Plugins => 3,
            Category::ConfD => 4,
            Category::Scripts => 5,
        };
        let within = match self.category {
            // Keep utilities readable by alias; hyprland is single file anyway
            Category::Hyprland | Category::Utility => self.alias.to_lowercase(),
            // Sort conf.d entries by filename alphabetically (e.g., 00-..., 05-..., ...)
            Category::ConfD => self.file_name.to_lowercase(),
            // Themes, plugins and scripts by alias
            Category::Themes | Category::Plugins | Category::Scripts => self.alias.to_lowercase(),
        };
        (cat_order, within)
    }
}
