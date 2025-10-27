mod cli;
mod edit;
mod model;
mod scan;
mod ui;

use anyhow::Result;
use cli::Cli;
use clap::Parser;
use scan::scan_configs;
use ui::Picker;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve root directory
    let root = cli.resolve_root()?;

    // Collect entries
    let mut entries = scan_configs(&root)?;

    // Build and run the picker
    let picker = Picker::new(cli.category, cli.color_spec.clone(), !cli.no_seg_colors);
    if let Some(selected) = picker.pick(&mut entries)? {
        // Launch editor
        edit::open_in_editor(cli.editor.as_deref(), &selected.path, &root)?;
    }
    Ok(())
}
