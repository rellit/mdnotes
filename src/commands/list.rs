use crate::cli::{ListArgs, ListTarget};
use crate::config::{ensure_setup, SetupOptions};
use crate::git::sync_pull;
use crate::models::ItemKind;
use crate::storage::load_items;
use crate::MdResult;

pub fn run(args: ListArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let target = args.target.unwrap_or_else(|| {
        if args.status.is_some() || args.priority.is_some() {
            ListTarget::Tasks
        } else {
            ListTarget::All
        }
    });
    match target {
        ListTarget::Notes => list_notes(&config),
        ListTarget::Tasks => list_tasks(&config, args.status.as_ref(), args.priority.as_ref()),
        ListTarget::All => {
            let mut out = Vec::new();
            out.push("Notes:".into());
            out.extend(list_notes(&config)?);
            out.push(String::new());
            out.push("Tasks:".into());
            out.extend(list_tasks(
                &config,
                args.status.as_ref(),
                args.priority.as_ref(),
            )?);
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

fn list_notes(config: &crate::config::Config) -> MdResult<Vec<String>> {
    let notes = load_items(config, ItemKind::Note)?;
    Ok(notes
        .into_iter()
        .map(|note| format!("{} - {}{}", note.id, note.title, format_tags(&note.tags)))
        .collect())
}

fn list_tasks(
    config: &crate::config::Config,
    status: Option<&crate::models::Status>,
    priority: Option<&crate::models::Priority>,
) -> MdResult<Vec<String>> {
    let tasks = load_items(config, ItemKind::Task)?;
    let mut out = Vec::new();
    for task in tasks {
        if let Some(filter) = status {
            if task.status.as_ref() != Some(filter) {
                continue;
            }
        }
        if let Some(filter) = priority {
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
