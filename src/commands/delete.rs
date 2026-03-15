use crate::config::{ensure_setup, SetupOptions};
use crate::git::{sync_pull, sync_push};
use crate::storage::resolve_item;
use crate::MdResult;
use std::fs;

pub fn run(id: String, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let (path, item) = resolve_item(&config, &id)?;
    // Remove the entire UUID directory (parent of MAIN.md)
    let item_dir = path
        .parent()
        .ok_or("MAIN.md file has no parent directory; cannot delete item")?;
    fs::remove_dir_all(item_dir)?;
    sync_push(&config, &format!("mdnotes: delete {}", item.id))?;
    Ok(vec![format!("Deleted {}", item_dir.display())])
}
