use crate::config::Config;
use crate::models::Item;
use crate::{MdError, MdResult};
use std::fs;
use std::path::Path;

pub fn refresh_tag_links(config: &Config, item: &Item) -> MdResult<()> {
    let path = config
        .root
        .join(item.kind.dir_name())
        .join(format!("{}.md", item.id));
    remove_tag_links(config, &path)?;
    for tag in &item.tags {
        let dir = config.root.join("tags").join(tag);
        fs::create_dir_all(&dir)?;
        let link = dir.join(&item.id);
        create_symlink_atomic(&path, &link)?;
    }
    Ok(())
}

pub fn remove_tag_links(config: &Config, path: &Path) -> MdResult<()> {
    let tags_dir = config.root.join("tags");
    if !tags_dir.exists() {
        return Ok(());
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| MdError("Missing file stem".into()))?
        .to_string();
    for entry in fs::read_dir(&tags_dir)? {
        let entry = entry?;
        let tag_dir = entry.path();
        if !tag_dir.is_dir() {
            continue;
        }
        let link_path = tag_dir.join(&stem);
        if link_path.exists() {
            fs::remove_file(&link_path)?;
        }
    }
    // clean empty tag directories
    for entry in fs::read_dir(&tags_dir)? {
        let entry = entry?;
        let tag_dir = entry.path();
        if tag_dir.is_dir() && fs::read_dir(&tag_dir)?.next().is_none() {
            fs::remove_dir(&tag_dir)?;
        }
    }
    Ok(())
}

fn create_symlink_atomic(target: &Path, link: &Path) -> MdResult<()> {
    let tmp_link = link.with_extension("tmp_link");
    if tmp_link.exists() {
        fs::remove_file(&tmp_link)?;
    }
    create_symlink(target, &tmp_link)?;
    if link.exists() {
        fs::remove_file(link)?;
    }
    fs::rename(&tmp_link, link)?;
    Ok(())
}

fn create_symlink(target: &Path, link: &Path) -> MdResult<()> {
    #[cfg(target_os = "windows")]
    {
        std::os::windows::fs::symlink_file(target, link).map_err(|e| MdError(e.to_string()))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| MdError(e.to_string()))
    }
}
