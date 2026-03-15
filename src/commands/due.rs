use crate::MdResult;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::{sync_pull, sync_push};
use crate::models::{ItemKind, Status};
use crate::storage::{resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::util::validate_due_inner;

pub fn run(id: String, due: Option<String>, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
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
    let message = match &item.due {
        Some(d) => format!("Due date for {} set to {}", item.id, d),
        None => format!("Due date cleared for {}", item.id),
    };
    sync_push(&config, &format!("mdnotes: due {}", item.id))?;
    Ok(vec![message])
}
