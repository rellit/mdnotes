use crate::config::{ensure_directories, Config};
use crate::models::{Item, ItemKind, Priority, Status};
use crate::util::parse_tags;
use crate::{MdError, MdResult};
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn write_item(config: &Config, item: &Item) -> MdResult<PathBuf> {
    ensure_directories(&config.root)?;
    let dir = config.root.join(item.kind.dir_name());
    let path = dir.join(format!("{}.md", item.id));
    let tmp_path = dir.join(format!("{}.md.tmp", item.id));
    {
        let mut file = File::create(&tmp_path)?;
        write_header(&mut file, item)?;
        file.sync_all()?;
    }
    fs::rename(&tmp_path, &path)?;
    Ok(path)
}

fn write_header(file: &mut File, item: &Item) -> MdResult<()> {
    writeln!(file, "id: {}", item.id)?;
    writeln!(file, "title: {}", item.title)?;
    writeln!(
        file,
        "type: {}",
        match item.kind {
            ItemKind::Note => "note",
            ItemKind::Task => "task",
        }
    )?;
    if let Some(status) = &item.status {
        writeln!(file, "status: {}", status.as_str())?;
    }
    if let Some(priority) = &item.priority {
        writeln!(file, "priority: {}", priority.as_str())?;
    }
    if let Some(due) = &item.due {
        writeln!(file, "due: {}", due)?;
    }
    if !item.tags.is_empty() {
        writeln!(file, "tags: {}", item.tags.join(", "))?;
    }
    writeln!(file, "--")?;
    write!(file, "{}", item.body)?;
    Ok(())
}

pub fn load_items(config: &Config, kind: ItemKind) -> MdResult<Vec<Item>> {
    let dir = config.root.join(kind.dir_name());
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match read_item(&path, kind.clone()) {
                Ok(item) => out.push(item),
                Err(err) => {
                    eprintln!(
                        "Warning: failed to load item from '{}': {}",
                        path.display(),
                        err
                    );
                }
            }
        }
    }
    Ok(out)
}

pub fn read_item(path: &Path, fallback_kind: ItemKind) -> MdResult<Item> {
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut status: Option<Status> = None;
    let mut priority: Option<Priority> = None;
    let mut due: Option<String> = None;
    let mut kind = fallback_kind.clone();
    let mut body = String::new();
    let mut in_body = false;
    let mut explicit_type = false;
    for line in content.lines() {
        if in_body {
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(line);
            continue;
        }
        if line.trim() == "--" {
            in_body = true;
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let v = value.trim();
            match key.trim() {
                "id" => id = Some(v.to_string()),
                "title" => title = Some(v.to_string()),
                "tags" => tags = parse_tags(v),
                "status" => {
                    status = match v {
                        "pending" => Some(Status::Pending),
                        "completed" | "complete" => Some(Status::Completed),
                        _ => None,
                    }
                }
                "priority" => {
                    priority = match v {
                        "low" => Some(Priority::Low),
                        "medium" => Some(Priority::Medium),
                        "high" => Some(Priority::High),
                        _ => None,
                    }
                }
                "due" => due = Some(v.to_string()),
                "type" => {
                    kind = match v {
                        "note" => ItemKind::Note,
                        "task" => ItemKind::Task,
                        other => {
                            return Err(MdError(format!(
                                "Unrecognized item type '{}' in file {}",
                                other,
                                path.display()
                            )))
                        }
                    };
                    explicit_type = true;
                }
                _ => {}
            }
        }
    }
    let id = id
        .or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| MdError("Missing id".into()))?;
    let title = title.ok_or_else(|| MdError("Missing title".into()))?;
    let kind = if status.is_some() || due.is_some() {
        ItemKind::Task
    } else if explicit_type {
        kind
    } else {
        fallback_kind
    };
    Ok(Item {
        id,
        title,
        kind,
        body,
        tags,
        status,
        priority,
        due,
    })
}

pub fn resolve_item(config: &Config, prefix: &str) -> MdResult<(ItemKind, PathBuf, Item)> {
    let mut matches: Vec<(ItemKind, PathBuf)> = Vec::new();
    for kind in [ItemKind::Note, ItemKind::Task] {
        let dir = config.root.join(kind.dir_name());
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.starts_with(prefix) {
                    matches.push((kind.clone(), path));
                }
            }
        }
    }
    if matches.is_empty() {
        return Err(MdError(format!("No item matches prefix '{prefix}'")));
    }
    if matches.len() > 1 {
        let mut msg = String::from("Multiple matches:\n");
        for (k, p) in matches {
            writeln!(
                &mut msg,
                "- {} ({})",
                p.file_name().and_then(|s| s.to_str()).unwrap_or_default(),
                k.dir_name()
            )
            .map_err(|e| MdError(e.to_string()))?;
        }
        return Err(MdError(msg));
    }
    let (kind, path) = matches.into_iter().next().unwrap();
    let item = read_item(&path, kind.clone())?;
    Ok((kind, path, item))
}
