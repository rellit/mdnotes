use crate::config::{ensure_setup, SetupOptions};
use crate::git::{sync_pull, sync_push};
use crate::storage::resolve_item;
use crate::tags::remove_tag_links;
use crate::MdResult;
use std::fs;

pub fn run(id: String, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let (_kind, path, _) = resolve_item(&config, &id)?;
    fs::remove_file(&path)?;
    remove_tag_links(&config, &path)?;
    sync_push(&config, &format!("mdnotes: delete {}", id))?;
    Ok(vec![format!("Deleted {}", path.display())])
}
