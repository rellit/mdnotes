use crate::cli::{FindArgs, ListTarget};
use crate::config::{ensure_setup, SetupOptions};
use crate::git::sync_pull;
use crate::models::ItemKind;
use crate::storage::load_items;
use crate::MdResult;

pub fn run(args: FindArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let query = args.query.to_ascii_lowercase();
    let target = args.target.unwrap_or(ListTarget::All);
    match target {
        ListTarget::Notes => find_notes(&config, &query),
        ListTarget::Tasks => find_tasks(&config, &query),
        ListTarget::All => {
            let mut out = Vec::new();
            out.push("Notes:".into());
            out.extend(find_notes(&config, &query)?);
            out.push(String::new());
            out.push("Tasks:".into());
            out.extend(find_tasks(&config, &query)?);
            Ok(out)
        }
    }
}

fn matches_query(value: &str, query: &str) -> bool {
    value.contains(query)
}

fn find_notes(config: &crate::config::Config, query: &str) -> MdResult<Vec<String>> {
    let notes = load_items(config, ItemKind::Note)?;
    Ok(notes
        .into_iter()
        .filter(|note| {
            let title = note.title.to_ascii_lowercase();
            let body = note.body.to_ascii_lowercase();
            matches_query(&title, query) || matches_query(&body, query)
        })
        .map(|note| format!("{} - {}", note.id, note.title))
        .collect())
}

fn find_tasks(config: &crate::config::Config, query: &str) -> MdResult<Vec<String>> {
    let tasks = load_items(config, ItemKind::Task)?;
    let mut out = Vec::new();
    for task in tasks {
        let title = task.title.to_ascii_lowercase();
        let body = task.body.to_ascii_lowercase();
        if !matches_query(&title, query) && !matches_query(&body, query) {
            continue;
        }
        out.push(format!(
            "{} - {} [{}{}{}]",
            task.id,
            task.title,
            task.status
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("pending"),
            task.priority
                .as_ref()
                .map(|p| format!(", {}", p.as_str()))
                .unwrap_or_default(),
            task.due
                .as_ref()
                .map(|d| format!(", due {d}"))
                .unwrap_or_default()
        ));
    }
    Ok(out)
}
