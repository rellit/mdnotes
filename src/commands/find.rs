use crate::cli::{FindArgs, FindTarget};
use crate::config::{ensure_setup, SetupOptions};
use crate::git::sync_pull;
use crate::storage::load_all_items;
use crate::MdResult;

pub fn run(args: FindArgs, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    let query = args.query.to_ascii_lowercase();
    let all = load_all_items(&config)?;
    let target = args.target.unwrap_or(FindTarget::All);

    let mut matched: Vec<_> = all
        .into_iter()
        .filter(|item| {
            let title = item.title.to_ascii_lowercase();
            let body = item.body.to_ascii_lowercase();
            (title.contains(&query) || body.contains(&query))
                && match target {
                    FindTarget::Notes => !item.is_task(),
                    FindTarget::Tasks => item.is_task(),
                    FindTarget::All => true,
                }
        })
        .collect();
    matched.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    Ok(matched
        .into_iter()
        .map(|item| format!("{} - {}", item.id, item.title))
        .collect())
}
