use crate::cli::ConfigArgs;
use crate::config::{config_path, ensure_setup, SetupOptions};
use crate::MdResult;

pub fn run(args: ConfigArgs, mut setup: SetupOptions) -> MdResult<Vec<String>> {
    if args.root.is_some() {
        setup.root_override = args.root;
    }
    if args.remote.is_some() {
        setup.remote_override = args.remote;
    }
    if args.editor.is_some() {
        setup.editor_override = args.editor;
    }
    let config = ensure_setup(setup.clone())?;
    Ok(vec![
        format!("Config: {}", config_path(&setup).display()),
        format!("Root: {}", config.root.display()),
        format!("Remote: {}", config.remote.as_deref().unwrap_or("<unset>")),
        format!("Editor: {}", config.editor.as_deref().unwrap_or("<unset>")),
    ])
}
