use crate::cli::AddArgs;
use crate::config::{ensure_setup, SetupOptions};
use crate::git::{sync_pull, sync_push};
use crate::models::{Item, ItemKind, Status};
use crate::storage::write_item;
use crate::tags::refresh_tag_links;
use crate::util::{parse_tags, validate_due_inner};
use crate::MdResult;
use uuid::Uuid;

pub fn run(args: AddArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    validate_supported_fields(
        args.kind.clone(),
        args.priority.is_some(),
        args.due.is_some(),
    )?;
    let mut item = Item {
        id: Uuid::new_v4().to_string(),
        title: args.title,
        kind: args.kind.clone(),
        body: args.body.unwrap_or_default(),
        tags: args
            .tags
            .as_ref()
            .map(|t| parse_tags(t))
            .unwrap_or_default(),
        status: None,
        priority: args.priority.clone(),
        due: args.due.clone(),
    };
    if let Some(due) = &item.due {
        validate_due_inner(due)?;
    }
    if matches!(item.kind, ItemKind::Task) && item.status.is_none() {
        item.status = Some(Status::Pending);
    }
    let path = write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    sync_push(&config, &format!("mdnotes: add {}", item.id))?;
    Ok(vec![format!("Created {}", path.display())])
}

fn validate_supported_fields(kind: ItemKind, has_priority: bool, has_due: bool) -> MdResult<()> {
    if matches!(kind, ItemKind::Note) && (has_priority || has_due) {
        return Err("Due dates and priority are only allowed for tasks".into());
    }
    Ok(())
}
