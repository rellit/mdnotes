use crate::MdResult;
use crate::config::{SetupOptions, ensure_setup};
use crate::storage::resolve_item;

pub fn run(id: String, setup: SetupOptions) -> MdResult<Vec<String>> {
    let config = ensure_setup(setup)?;
    let (path, item) = resolve_item(&config, &id)?;
    let output = vec![item.to_string(), String::new(), path.display().to_string()];
    Ok(output)
}
