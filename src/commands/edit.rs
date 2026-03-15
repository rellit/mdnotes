use crate::cli::EditArgs;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::{sync_pull, sync_push};
use crate::models::{ItemKind, Status};
use crate::storage::{read_item, resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::util::{parse_tags, validate_due_inner};
use crate::{MdError, MdResult};
use std::process::Command;

pub fn run(args: EditArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let (path, mut item) = resolve_item(&config, &args.id)?;
    let original_id = item.id.clone();
    let has_field_update = args.title.is_some()
        || args.body.is_some()
        || args.tags.is_some()
        || args.due.is_some()
        || args.priority.is_some()
        || args.status.is_some();
    if !has_field_update {
        open_editor(&config, &path)?;
        item = read_item(&path)?;
        // The canonical ID is the name of the UUID directory, not the file content
        let path_id = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(&original_id)
            .to_string();
        if item.id != path_id {
            item.id = path_id;
        }
    } else {
        if let Some(title) = args.title {
            item.title = title;
        }
        if let Some(body) = args.body {
            item.body = body;
        }
        if let Some(tags) = args.tags {
            item.tags = parse_tags(&tags);
        }
        if let Some(due) = args.due {
            if due.trim().is_empty() {
                item.due = None;
            } else {
                validate_due_inner(&due)?;
                item.due = Some(due);
            }
        }
        if let Some(priority) = args.priority {
            item.priority = Some(priority);
        }
        if let Some(status) = args.status {
            item.status = Some(status);
        }
    }
    item.kind = ItemKind::infer(&item.status, &item.due);
    if matches!(item.kind, ItemKind::Task) && item.status.is_none() {
        item.status = Some(Status::Pending);
    }
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    sync_push(&config, &format!("mdnotes: edit {}", item.id))?;
    Ok(vec![format!(
        "Updated {}/{}/MAIN.md",
        config.root.display(),
        item.id
    )])
}

fn open_editor(config: &crate::config::Config, path: &std::path::Path) -> MdResult<()> {
    let editor = config
        .editor
        .clone()
        .or_else(|| std::env::var("VISUAL").ok())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "notepad".into()
            } else {
                "vi".into()
            }
        });
    let status = Command::new(editor)
        .arg(path)
        .status()
        .map_err(|e| MdError(e.to_string()))?;
    if !status.success() {
        return Err(MdError("Editor exited with an error".into()));
    }
    Ok(())
}
