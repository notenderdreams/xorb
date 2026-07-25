use anyhow::{Context, Result};
use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard provider")?;
    clipboard
        .set_text(text)
        .context("Failed to set text content into system clipboard")?;
    Ok(())
}
