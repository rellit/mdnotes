use crate::config::{ensure_setup, SetupOptions};
use crate::storage::resolve_item;
use crate::MdResult;

pub fn run(id: String, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (path, item) = resolve_item(&config, &id)?;
    let output = vec![item.to_string(), String::new(), path.display().to_string()];
    Ok(output)
}
