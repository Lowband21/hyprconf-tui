use std::borrow::Cow;
use anyhow::Result;
use skim::prelude::*;
use skim_tuikit::prelude::{Attr, Color, Effect};

use crate::model::{Category, ConfigEntry};

pub struct Picker {
    pub category: Option<Category>,
    pub color_spec: Option<String>,
    pub seg_colors: bool,
}

struct ColoredItem {
    text: String,
    display: AnsiString<'static>,
    id_path: String,
    index: usize,
}

impl SkimItem for ColoredItem {
    fn text(&self) -> Cow<str> { Cow::Borrowed(&self.text) }
    fn output(&self) -> Cow<str> { Cow::Borrowed(&self.id_path) }
    fn get_index(&self) -> usize { self.index }
    fn set_index(&mut self, index: usize) { self.index = index; }

    fn display<'a>(&'a self, context: DisplayContext<'a>) -> AnsiString<'a> {
        // Start with our colored segments, then overlay highlight for matches
        let mut ret = self.display.clone();
        let new_fragments: Vec<(Attr, (u32, u32))> = match context.matches {
            Matches::CharIndices(indices) => indices
                .iter()
                .map(|&idx| (context.highlight_attr, (idx as u32, idx as u32 + 1)))
                .collect(),
            Matches::CharRange(start, end) => vec![(context.highlight_attr, (start as u32, end as u32))],
            Matches::ByteRange(start, end) => {
                let ch_start = context.text[..start].chars().count();
                let ch_end = ch_start + context.text[start..end].chars().count();
                vec![(context.highlight_attr, (ch_start as u32, ch_end as u32))]
            }
            Matches::None => vec![],
        };
        ret.override_attrs(new_fragments);
        ret
    }
}

impl Picker {
    pub fn new(category: Option<Category>, color_spec: Option<String>, seg_colors: bool) -> Self {
        Self { category, color_spec, seg_colors }
    }

    pub fn pick(&self, entries: &mut [ConfigEntry]) -> Result<Option<ConfigEntry>> {
        // Pre-filter by category if requested
        let filtered: Vec<&ConfigEntry> = entries
            .iter()
            .filter(|e| match self.category { Some(c) => e.category == c, None => true })
            .collect();

        // Build custom items channel
        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let enable_seg_colors = self.seg_colors && std::env::var("NO_COLOR").is_err();
        for (i, e) in filtered.iter().enumerate() {
            let (text, display) = build_colored_line(e, enable_seg_colors);
            let item = ColoredItem {
                id_path: e.path.to_string_lossy().into_owned(),
                text,
                display,
                index: i,
            };
            let _ = tx.send(Arc::new(item));
        }
        drop(tx);

        // Build skim options
        let mut builder = SkimOptionsBuilder::default();
        builder
            .height("60%".to_string())
            .multi(false)
            .reverse(true)
            .prompt(String::new());
        let mut options = builder.build().unwrap();

        // Apply skim color scheme: prefer CLI value, else default to dark (unless NO_COLOR is set)
        if let Some(spec) = &self.color_spec {
            options.color = Some(spec.clone());
        } else if std::env::var("NO_COLOR").is_err() {
            options.color = Some("dark".to_string());
        }

        let out = Skim::run_with(&options, Some(rx));
        if let Some(out) = out {
            if out.is_abort { return Ok(None); }
            if let Some(selected) = out.selected_items.first() {
                // We output the path; lookup entry by path
                let path_out = selected.output();
                if let Some(entry) = filtered.iter().find(|e| e.path.to_string_lossy() == *path_out) {
                    return Ok(Some((*entry).clone()));
                }
            }
        }
        Ok(None)
    }
}

fn build_colored_line(e: &ConfigEntry, seg_colors: bool) -> (String, AnsiString<'static>) {
    // Build the base (stripped) string and fragment ranges per segment
    let desc = e.description.as_deref().unwrap_or("");
    let sep = if desc.trim().is_empty() { "" } else { " — " };
    let base = format!(
        "[{cat}] {alias}{sep}{desc} | {file} ({path})",
        cat = e.category,
        alias = e.alias,
        sep = sep,
        desc = desc,
        file = e.file_name,
        path = e.path.display(),
    );

    if !seg_colors {
        return (base.clone(), base.clone().into());
    }

    // Compute char indices while constructing segments
    // We will color: category label (without brackets), alias, desc (if any), trailing file+path
    let mut fragments: Vec<(Attr, (u32, u32))> = Vec::new();

    let mut idx: usize = 0; // char index

    // "["
    idx += "[".chars().count();
    // category text start
    let cat_text = e.category.to_string();
    let cat_len = cat_text.chars().count();
    let cat_attr = Attr { fg: Color::AnsiValue(3), bg: Color::Default, effect: Effect::empty() }; // yellow
    fragments.push((cat_attr, (idx as u32, (idx + cat_len) as u32)));
    idx += cat_len;
    // "] "
    idx += "] ".chars().count();

    // alias
    let alias_len = e.alias.chars().count();
    let alias_attr = Attr { fg: Color::Rgb(0xDA, 0x68, 0xEC), bg: Color::Default, effect: Effect::BOLD };
    fragments.push((alias_attr, (idx as u32, (idx + alias_len) as u32)));
    idx += alias_len;

    // sep + desc
    if !sep.is_empty() {
        idx += sep.chars().count();
        let desc_len = desc.chars().count();
        let desc_attr = Attr { fg: Color::Rgb(0xFF, 0x6A, 0x3D), bg: Color::Default, effect: Effect::empty() };
        fragments.push((desc_attr, (idx as u32, (idx + desc_len) as u32)));
        idx += desc_len;
    }

    // space before pipe
    idx += " ".chars().count();
    // trailing file+path start at current idx including "| " and rest
    let file_trail = format!("| {} ({})", e.file_name, e.path.display());
    let file_len = file_trail.chars().count();
    let file_attr = Attr { fg: Color::AnsiValue(15), bg: Color::Default, effect: Effect::empty() };
    fragments.push((file_attr, (idx as u32, (idx + file_len) as u32)));

    let ansi = AnsiString::new_string(base.clone(), fragments);
    (base, ansi)
}
