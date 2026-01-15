use crate::config::{ensure_setup, SetupOptions};
use crate::git::sync_pull;
use crate::models::ItemKind;
use crate::storage::load_items;
use crate::MdResult;

pub fn run(query: String, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let notes = load_items(&config, ItemKind::Note)?;
    let mut out = Vec::new();
    for note in notes {
        if note
            .title
            .to_ascii_lowercase()
            .contains(&query.to_ascii_lowercase())
            || note
                .body
                .to_ascii_lowercase()
                .contains(&query.to_ascii_lowercase())
        {
            out.push(format!("{} - {}", note.id, note.title));
        }
    }
    Ok(out)
}
