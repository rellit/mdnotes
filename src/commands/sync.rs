use crate::MdResult;
use crate::config::{SetupOptions, ensure_setup};
use crate::git::{sync_pull, sync_push};

pub fn run(setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    sync_push(&config, "mdnotes: sync")?;
    Ok(vec!["Synced with remote".to_string()])
}
