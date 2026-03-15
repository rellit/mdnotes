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
    let relative = path
        .strip_prefix(&config.root)
        .map_err(|_| MdError("Failed to derive relative path".into()))?
        .to_string_lossy()
        .to_string();
    let tags_dir = config.root.join("tags");
    fs::create_dir_all(&tags_dir)?;
    for tag in &item.tags {
        let tag_file = tags_dir.join(tag);
        let mut lines: Vec<String> = if tag_file.exists() {
            fs::read_to_string(&tag_file)?
                .lines()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };
        if !lines.iter().any(|l| l == &relative) {
            lines.push(relative.clone());
            fs::write(&tag_file, lines.join("\n") + "\n")?;
        }
    }
    Ok(())
}

pub fn remove_tag_links(config: &Config, path: &Path) -> MdResult<()> {
    let tags_dir = config.root.join("tags");
    if !tags_dir.exists() {
        return Ok(());
    }
    let relative = path
        .strip_prefix(&config.root)
        .map_err(|_| MdError("Failed to derive relative path".into()))?
        .to_string_lossy()
        .to_string();
    for entry in fs::read_dir(&tags_dir)? {
        let entry = entry?;
        let tag_file = entry.path();
        if !tag_file.is_file() {
            continue;
        }
        let mut lines: Vec<String> = fs::read_to_string(&tag_file)?
            .lines()
            .map(|s| s.to_string())
            .collect();
        let original_len = lines.len();
        lines.retain(|l| l != &relative);
        if lines.len() != original_len {
            if lines.is_empty() {
                fs::remove_file(&tag_file)?;
            } else {
                fs::write(&tag_file, lines.join("\n") + "\n")?;
            }
        }
    }
    Ok(())
}
