use crate::config::Config;
use crate::models::Item;
use crate::{MdError, MdResult};
use std::fs;
use std::path::Path;
#[cfg(target_os = "windows")]
use std::os::windows::fs as winfs;

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
        // Windows ERROR_PRIVILEGE_NOT_HELD (1314) indicates missing SeCreateSymbolicLinkPrivilege.
        const WINDOWS_ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;
        match winfs::symlink_file(target, link) {
            Ok(()) => Ok(()),
            Err(e) if e.raw_os_error() == Some(WINDOWS_ERROR_PRIVILEGE_NOT_HELD) => {
                // Fallback when symlinks are not permitted; hard links keep content available
                // but won't mirror changes if the target is later replaced.
                fs::hard_link(target, link).map_err(|e| MdError(e.to_string()))
            }
            Err(e) => Err(MdError(e.to_string())),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| MdError(e.to_string()))
    }
}
