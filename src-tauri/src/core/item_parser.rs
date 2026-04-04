/// Parse a PoE item pasted from clipboard (Ctrl+C in-game).
use anyhow::{anyhow, Result};
use crate::models::build::Item;

pub fn parse_clipboard(text: &str) -> Result<Item> {
    let lines: Vec<String> = text.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return Err(anyhow!("Empty clipboard text"));
    }

    if !lines[0].starts_with("Rarity:") {
        return Err(anyhow!("Not a PoE item — expected 'Rarity:' on first line"));
    }

    let mut item = Item::default();
    crate::core::pob_parser::parse_item_text(&lines, &mut item);
    Ok(item)
}
