use std::io::Cursor;

use anyhow::Result;
use skim::prelude::*;

use crate::model::{Category, ConfigEntry};

pub struct Picker {
    category: Option<Category>,
    color_spec: Option<String>,
    seg_colors: bool,
}

impl Picker {
    pub fn new(category: Option<Category>, color_spec: Option<String>, seg_colors: bool) -> Self {
        Self { category, color_spec, seg_colors }
    }

    pub fn pick(&self, entries: &mut [ConfigEntry]) -> Result<Option<ConfigEntry>> {
        // Pre-filter by category if requested
        let filtered: Vec<(usize, &ConfigEntry)> = entries
            .iter()
            .enumerate()
            .filter(|(_, e)| match self.category {
                Some(c) => e.category == c,
                None => true,
            })
            .collect();

        // Build display lines. Keep the last fields as raw path/index for retrieval.
        // Layout: "[category] alias — shortdesc | file (path)\t<path>\t<idx>"
        let mut buf = String::new();
        for (pos, (_idx_orig, e)) in filtered.iter().enumerate() {
            let desc = e.description.as_deref().unwrap_or("");
            let cat = e.category.to_string();
            let sep = if desc.trim().is_empty() { "" } else { " — " };
            let display = format!(
                "[{cat}] {alias}{sep}{desc} | {file} ({path})",
                cat = cat,
                alias = e.alias,
                sep = sep,
                desc = desc,
                file = e.file_name,
                path = e.path.display(),
            );
            let line = format!("{display}\t{path}\t{pos}", display = display, path = e.path.display(), pos = pos);
            buf.push_str(&line);
            buf.push('\n');
        }

        let mut options = SkimOptionsBuilder::default()
            .height("60%".to_string())
            .multi(false)
            .reverse(true)
            .prompt(String::new())
            .bind(vec![
                "alt-k:kill-line".to_string(),
                "alt-w:toggle-preview".to_string(),
            ]) // small QoL
            .build()
            .unwrap();

        // Apply skim color scheme: prefer CLI value, else default to dark (unless NO_COLOR is set)
        if let Some(spec) = &self.color_spec {
            options.color = Some(spec.clone());
        } else if std::env::var("NO_COLOR").is_err() {
            options.color = Some("dark".to_string());
        }

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(buf));
        let out = Skim::run_with(&options, Some(items));

        if let Some(out) = out {
            if out.is_abort {
                return Ok(None);
            }
            if let Some(selected) = out.selected_items.first() {
                let output = selected.output();
                // Split from the end to get idx (stable) or path
                // Format: ... \t <path> \t <idx>
                let parts: Vec<&str> = output.rsplitn(2, '\t').collect();
                let idx_opt = parts.first().and_then(|s| s.parse::<usize>().ok());

                if let Some(pos) = idx_opt
                    && let Some((_, e)) = filtered.get(pos)
                {
                    return Ok(Some((*e).clone()));
                }
                // Fallback: match by path
                if let Some(path_field) = parts.get(1) {
                    let path_field = path_field.trim();
                    if let Some((_, e)) = filtered
                        .iter()
                        .find(|(_, e)| e.path.to_string_lossy() == path_field)
                    {
                        return Ok(Some((*e).clone()));
                    }
                }
            }
        }
        Ok(None)
    }
}
