use crate::MdResult;
use crate::cli::ListArgs;
use crate::config::{SetupOptions, ensure_setup};
use crate::filter::parse_query;
use crate::storage::load_all_items;

pub fn run(args: ListArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let predicate = parse_query(args.query.as_deref().unwrap_or(""))?;
    let all = load_all_items(&config)?;
    let mut items: Vec<_> = all.into_iter().filter(|i| predicate.matches(i)).collect();
    items.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    let out = items.into_iter().map(format_item).collect();
    Ok(out)
}

fn format_item(item: crate::models::Item) -> String {
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
        format!("{} - {}", item.id, item.title)
    } else {
        format!("{} - {} [{}]", item.id, item.title, meta.join(", "))
    }
}
