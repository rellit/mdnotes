use crate::config::{config_path, ensure_setup, SetupOptions};
use crate::MdResult;

pub fn run(root: Option<std::path::PathBuf>, mut setup: SetupOptions) -> MdResult<Vec<String>> {
    if root.is_some() {
        setup.root_override = root;
    }
    let config = ensure_setup(setup.clone())?;
    Ok(vec![
        format!("Config: {}", config_path(&setup).display()),
        format!("Root: {}", config.root.display()),
    ])
}
