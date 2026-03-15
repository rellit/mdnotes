use crate::config::Config;
use crate::models::{Item, ItemKind, Status};
use crate::util::parse_tags;
use crate::{MdError, MdResult};
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const HEADER_EXAMPLES: &str = "\
# status: pending|completed
# priority: 5
# due: 2099-12-31
# tags: tag-one, tag-two
";

/// Returns the path to the item's markdown file inside `item_dir`.
/// Looks for any file whose lowercased name equals "main.md"; returns the
/// canonical `main.md` path (lowercase) as the canonical target.
fn find_main_md(item_dir: &Path) -> Option<PathBuf> {
    if let Ok(rd) = fs::read_dir(item_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().to_lowercase() == "main.md" {
                return Some(entry.path());
            }
        }
    }
    None
}

/// Writes an item to `<root>/<id>/main.md`.
pub fn write_item(config: &Config, item: &Item) -> MdResult<PathBuf> {
    write_item_inner(config, item, false)
}

/// Writes an item with example headers as comments.
pub fn write_item_with_examples(config: &Config, item: &Item) -> MdResult<PathBuf> {
    write_item_inner(config, item, true)
}

fn write_item_inner(config: &Config, item: &Item, include_examples: bool) -> MdResult<PathBuf> {
    let item_dir = config.root.join(&item.id);
    fs::create_dir_all(&item_dir)?;
    // Always write to lowercase "main.md"; if an old mixed-case file exists, remove it first.
    let path = item_dir.join("main.md");
    if let Some(existing) = find_main_md(&item_dir)
        && existing != path
    {
        fs::remove_file(&existing)?;
    }
    let tmp_path = item_dir.join("main.md.tmp");
    {
        let mut file = File::create(&tmp_path)?;
        write_header(&mut file, item, include_examples)?;
        file.sync_all()?;
    }
    fs::rename(&tmp_path, &path)?;
    Ok(path)
}

fn write_header(file: &mut File, item: &Item, include_examples: bool) -> MdResult<()> {
    writeln!(file, "title: {}", item.title)?;
    if let Some(status) = &item.status {
        writeln!(file, "status: {}", status.as_str())?;
    }
    if let Some(priority) = &item.priority {
        writeln!(file, "priority: {priority}")?;
    }
    if let Some(due) = &item.due {
        writeln!(file, "due: {}", due)?;
    }
    if !item.tags.is_empty() {
        writeln!(file, "tags: {}", item.tags.join(", "))?;
    }
    if include_examples {
        for line in HEADER_EXAMPLES.lines() {
            writeln!(file, "{line}")?;
        }
    }
    writeln!(file, "--")?;
    write!(file, "{}", item.body)?;
    Ok(())
}

/// Loads all items from `<root>/<uuid>/MAIN.md` directories.
pub fn load_all_items(config: &Config) -> MdResult<Vec<Item>> {
    let root = &config.root;
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }
        // Skip hidden directories such as .git
        if dir_path
            .file_name()
            .map(|n| n.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }
        let main_path = find_main_md(&dir_path);
        if let Some(main_path) = main_path {
            match read_item(&main_path) {
                Ok(item) => out.push(item),
                Err(err) => {
                    eprintln!(
                        "Warning: failed to load item from '{}': {}",
                        main_path.display(),
                        err
                    );
                }
            }
        }
    }
    Ok(out)
}

/// Loads items of a specific kind (notes or tasks) by filtering all items.
pub fn load_items(config: &Config, kind: ItemKind) -> MdResult<Vec<Item>> {
    let all = load_all_items(config)?;
    Ok(all.into_iter().filter(|item| item.kind == kind).collect())
}

/// Parses a single `MAIN.md` file into an [`Item`].
pub fn read_item(path: &Path) -> MdResult<Item> {
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut status: Option<Status> = None;
    let mut priority: Option<u32> = None;
    let mut due: Option<String> = None;
    let mut body = String::new();
    let mut in_body = false;
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
                    priority = v.parse::<u32>().ok();
                }
                "due" => due = Some(v.to_string()),
                // "type" field is no longer used; kind is derived from due date
                _ => {}
            }
        }
    }
    // Fall back to directory name as ID if not present in file
    let id = id
        .or_else(|| {
            path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| MdError("Missing id".into()))?;
    let title = title.ok_or_else(|| MdError("Missing title".into()))?;
    let kind = ItemKind::infer(&status, &due);
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

/// Returns all item IDs (directory names) present in the root, without
/// parsing the contents of each item file.  This is cheaper than
/// [`load_all_items`] and is used to compute shortest unique prefixes.
pub fn list_item_ids(config: &Config) -> MdResult<Vec<String>> {
    let root = &config.root;
    let mut ids = Vec::new();
    if !root.exists() {
        return Ok(ids);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }
        let name = dir_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !name.starts_with('.') && find_main_md(&dir_path).is_some() {
            ids.push(name);
        }
    }
    Ok(ids)
}

/// Returns `(path_to_main.md, item)`.
pub fn resolve_item(config: &Config, prefix: &str) -> MdResult<(PathBuf, Item)> {
    let mut matches: Vec<PathBuf> = Vec::new();
    if config.root.exists() {
        for entry in fs::read_dir(&config.root)? {
            let entry = entry?;
            let dir_path = entry.path();
            if !dir_path.is_dir() {
                continue;
            }
            if let Some(dir_name) = dir_path.file_name().and_then(|n| n.to_str())
                && dir_name.starts_with(prefix)
                && !dir_name.starts_with('.')
                && let Some(main_path) = find_main_md(&dir_path)
            {
                matches.push(main_path);
            }
        }
    }
    if matches.is_empty() {
        return Err(MdError(format!("No item matches prefix '{prefix}'")));
    }
    if matches.len() > 1 {
        let mut msg = String::from("Multiple matches:\n");
        for p in &matches {
            writeln!(&mut msg, "- {}", p.display()).map_err(|e| MdError(e.to_string()))?;
        }
        return Err(MdError(msg));
    }
    let path = matches.into_iter().next().unwrap();
    let item = read_item(&path)?;
    Ok((path, item))
}
