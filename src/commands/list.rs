use crate::MdResult;
use crate::cli::ListArgs;
use crate::config::{SetupOptions, ensure_setup};
use crate::filter::parse_query;
use crate::storage::load_all_items;
use crate::util::shortest_unique_prefix;

pub fn run(args: ListArgs, setup: SetupOptions, verbose: bool) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let predicate = parse_query(args.query.as_deref().unwrap_or(""))?;
    let all = load_all_items(&config)?;
    let all_ids: Vec<String> = all.iter().map(|i| i.id.clone()).collect();
    let mut items: Vec<_> = all.into_iter().filter(|i| predicate.matches(i)).collect();
    items.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    let out = items
        .into_iter()
        .map(|item| format_item(item, &all_ids, verbose))
        .collect();
    Ok(out)
}

fn format_item(item: crate::models::Item, all_ids: &[String], verbose: bool) -> String {
    let display_id = if verbose {
        item.id.clone()
    } else {
        shortest_unique_prefix(&item.id, all_ids)
    };
    let mut meta = Vec::new();
    if let Some(status) = &item.status {
        meta.push(status.as_str().to_string());
    }
    if let Some(priority) = &item.priority {
        meta.push(format!("prio {priority}"));
    }
    if let Some(due) = &item.due {
        meta.push(format!("due {due}"));
    }
    if !item.tags.is_empty() {
        meta.push(format!("tags: {}", item.tags.join(", ")));
    }
    if meta.is_empty() {
        format!("{} - {}", display_id, item.title)
    } else {
        format!("{} - {} [{}]", display_id, item.title, meta.join(", "))
    }
}
