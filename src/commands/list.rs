use crate::cli::{ListArgs, ListTarget};
use crate::config::{ensure_setup, SetupOptions};
use crate::models::ItemKind;
use crate::storage::load_items;
use crate::MdResult;

pub fn run(args: ListArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    match args.target {
        ListTarget::Notes => {
            let notes = load_items(&config, ItemKind::Note)?;
            Ok(notes
                .into_iter()
                .map(|note| format!("{} - {}{}", note.id, note.title, format_tags(&note.tags)))
                .collect())
        }
        ListTarget::Tasks => {
            let tasks = load_items(&config, ItemKind::Task)?;
            let mut out = Vec::new();
            for task in tasks {
                if let Some(filter) = &args.status {
                    if task.status.as_ref() != Some(filter) {
                        continue;
                    }
                }
                if let Some(filter) = &args.priority {
                    if task.priority.as_ref() != Some(filter) {
                        continue;
                    }
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
    }
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}
