use crate::config::{ensure_setup, SetupOptions};
use crate::git::{sync_pull, sync_push};
use crate::MdResult;

pub fn run(setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    sync_pull(&config)?;
    sync_push(&config, "mdnotes: sync")?;
    Ok(vec!["Synced with remote".to_string()])
}
