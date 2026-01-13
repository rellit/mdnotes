use crate::cli::EditArgs;
use crate::config::{ensure_setup, SetupOptions};
use crate::models::{ItemKind, Priority};
use crate::storage::{resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::util::{parse_tags, validate_due_inner};
use crate::{MdError, MdResult};

pub fn run(args: EditArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (kind, path, mut item) = resolve_item(&config, &args.id)?;
    if matches!(kind, ItemKind::Note)
        && (args.priority.is_some() || args.due.is_some() || args.status.is_some())
    {
        return Err("Status, due date, and priority can only be set on tasks".into());
    }
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
        validate_due_inner(&due)?;
        item.due = Some(due);
    }
    if let Some(priority) = args.priority {
        ensure_priority_valid_for_kind(&kind, &priority)?;
        item.priority = Some(priority);
    }
    if let Some(status) = args.status {
        if !matches!(kind, ItemKind::Task) {
            return Err(MdError("Status is only valid for tasks".into()));
        }
        item.status = Some(status);
    }
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    Ok(vec![format!("Updated {}", path.display())])
}

fn ensure_priority_valid_for_kind(kind: &ItemKind, _priority: &Priority) -> MdResult<()> {
    if matches!(kind, ItemKind::Note) {
        return Err("Priority is only allowed for tasks".into());
    }
    Ok(())
}
