use crate::MdResult;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::sync_push;
use crate::storage::resolve_item;
use std::fs;

pub fn run(id: String, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (path, item) = resolve_item(&config, &id)?;
    // Remove the entire UUID directory (parent of MAIN.md)
    let item_dir = path
        .parent()
        .ok_or("MAIN.md file has no parent directory; cannot delete item")?;
    fs::remove_dir_all(item_dir)?;
    sync_push(&config, &format!("mdnotes: delete {}", item.id))?;
    Ok(vec![format!("Deleted {}", item_dir.display())])
}
