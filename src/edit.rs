use std::{env, path::Path, process::Command};

use anyhow::{bail, Context, Result};

pub fn open_in_editor(editor: Option<&str>, path: &Path, _root: &Path) -> Result<()> {
    let editor_cmd = editor
        .map(|s| s.to_string())
        .or_else(|| env::var("EDITOR").ok())
        .unwrap_or_else(|| "hx".to_string());

    let status = Command::new(editor_cmd)
        .arg(path)
        .status()
        .with_context(|| format!("failed to spawn editor for {}", path.display()))?;

    if !status.success() {
        bail!("editor exited with status {:?}", status.code());
    }
    Ok(())
}

