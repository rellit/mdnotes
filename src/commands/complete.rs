use crate::config::{ensure_setup, SetupOptions};
use crate::models::{ItemKind, Status};
use crate::storage::{resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::MdResult;

pub fn run(id: String, completed: bool, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (kind, _path, mut item) = resolve_item(&config, &id)?;
    if !matches!(kind, ItemKind::Task) {
        return Err("Completion can only be toggled for tasks".into());
    }
    item.status = Some(if completed {
        Status::Completed
    } else {
        Status::Pending
    });
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    let message = match &item.status {
        Some(status) => format!("Task {} marked {}", item.id, status.as_str()),
        None => format!("Task {} updated", item.id),
    };
    Ok(vec![message])
}
