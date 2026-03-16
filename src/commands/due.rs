use crate::MdResult;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::sync_push;
use crate::models::{ItemKind, Status};
use crate::storage::{list_item_ids, resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::util::{shortest_unique_prefix, validate_due_inner};

pub fn run(
    id: String,
    due: Option<String>,
    setup: SetupOptions,
    verbose: bool,
) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (_path, mut item) = resolve_item(&config, &id)?;
    if let Some(due_value) = due {
        let validated = validate_due_inner(&due_value)?;
        item.due = Some(validated);
    } else {
        item.due = None;
    }
    item.kind = ItemKind::infer(&item.status, &item.due);
    if matches!(item.kind, ItemKind::Task) && item.status.is_none() {
        item.status = Some(Status::Pending);
    }
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    let display_id = if verbose {
        item.id.clone()
    } else {
        let all_ids = list_item_ids(&config)?;
        shortest_unique_prefix(&item.id, &all_ids)
    };
    let message = match &item.due {
        Some(d) => format!("Due date for {} set to {}", display_id, d),
        None => format!("Due date cleared for {}", display_id),
    };
    sync_push(&config, &format!("mdnotes: due {}", item.id))?;
    Ok(vec![message])
}
