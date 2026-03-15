use crate::config::{ensure_setup, SetupOptions};
use crate::git::{sync_pull, sync_push};
use crate::models::Status;
use crate::storage::{resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::MdResult;

pub fn run(id: String, completed: bool, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let (_path, mut item) = resolve_item(&config, &id)?;
    if !item.is_task() {
        return Err("Completion can only be toggled for tasks (items with a due date)".into());
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
    sync_push(&config, &format!("mdnotes: complete {}", item.id))?;
    Ok(vec![message])
}
