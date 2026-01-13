use crate::config::{ensure_setup, SetupOptions};
use crate::storage::resolve_item;
use crate::tags::remove_tag_links;
use crate::MdResult;
use std::fs;

pub fn run(id: String, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (_kind, path, _) = resolve_item(&config, &id)?;
    fs::remove_file(&path)?;
    remove_tag_links(&config, &path)?;
    Ok(vec![format!("Deleted {}", path.display())])
}
