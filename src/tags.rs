use crate::config::Config;
use crate::models::Item;
use crate::MdResult;
use std::path::Path;

/// Tag links are no longer maintained as a file-system index.
/// Tags are stored directly in each item's `MAIN.md` and parsed live.
pub fn refresh_tag_links(_config: &Config, _item: &Item) -> MdResult<()> {
    Ok(())
}

/// Tag links are no longer maintained as a file-system index.
pub fn remove_tag_links(_config: &Config, _path: &Path) -> MdResult<()> {
    Ok(())
}
