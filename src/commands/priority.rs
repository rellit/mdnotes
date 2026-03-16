use crate::MdResult;
use crate::cli::PriorityArgs;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::sync_push;
use crate::storage::{list_item_ids, resolve_item, write_item};
use crate::tags::refresh_tag_links;
use crate::util::shortest_unique_prefix;

pub fn run(args: PriorityArgs, setup: SetupOptions, verbose: bool) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (_path, mut item) = resolve_item(&config, &args.id)?;
    item.priority = args.value;
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    sync_push(&config, &format!("mdnotes: priority {}", item.id))?;
    let display_id = if verbose {
        item.id.clone()
    } else {
        let all_ids = list_item_ids(&config)?;
        shortest_unique_prefix(&item.id, &all_ids)
    };
    let message = match item.priority {
        Some(p) => format!("Priority for {} set to {}", display_id, p),
        None => format!("Priority cleared for {}", display_id),
    };
    Ok(vec![message])
}
