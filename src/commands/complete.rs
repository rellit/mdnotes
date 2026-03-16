use crate::MdResult;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::sync_push;
use crate::models::Status;
use crate::storage::{list_item_ids, resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::util::shortest_unique_prefix;

pub fn run(
    id: String,
    completed: bool,
    setup: SetupOptions,
    verbose: bool,
) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (_path, mut item) = resolve_item(&config, &id)?;
    if !item.is_task() {
        return Err(
            "Cannot toggle completion: item must have a due date to be considered a task. \
             Use the `due` command to set a due date first."
                .into(),
        );
    }
    item.status = Some(if completed {
        Status::Completed
    } else {
        Status::Pending
    });
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    let display_id = if verbose {
        item.id.clone()
    } else {
        let all_ids = list_item_ids(&config)?;
        shortest_unique_prefix(&item.id, &all_ids)
    };
    let message = match &item.status {
        Some(status) => format!("Task {} marked {}", display_id, status.as_str()),
        None => format!("Task {} updated", display_id),
    };
    sync_push(&config, &format!("mdnotes: complete {}", item.id))?;
    Ok(vec![message])
}
