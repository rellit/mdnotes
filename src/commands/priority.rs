use crate::MdResult;
use crate::cli::PriorityArgs;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::{sync_pull, sync_push};
use crate::storage::{resolve_item, write_item};
use crate::tags::refresh_tag_links;

pub fn run(args: PriorityArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let (_path, mut item) = resolve_item(&config, &args.id)?;
    item.priority = args.value;
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    sync_push(&config, &format!("mdnotes: priority {}", item.id))?;
    let message = match item.priority {
        Some(p) => format!("Priority for {} set to {}", item.id, p),
        None => format!("Priority cleared for {}", item.id),
    };
    Ok(vec![message])
}
