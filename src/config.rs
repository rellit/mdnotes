use crate::{MdError, MdResult};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const CONFIG_NAME: &str = "mdnrc";
pub const DEFAULT_ROOT_DIR: &str = "repository";
pub const CONFIG_OVERRIDE_ENV: &str = "MDNOTES_CONFIG_HOME";
pub const ROOT_OVERRIDE_ENV: &str = "MDNOTES_ROOT";

#[derive(Clone, Debug)]
pub struct Config {
    pub root: PathBuf,
    pub remote: Option<String>,
    pub editor: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SetupOptions {
    pub root_override: Option<PathBuf>,
    pub config_home: Option<PathBuf>,
    pub remote_override: Option<String>,
    pub editor_override: Option<String>,
}

pub fn ensure_setup(opts: SetupOptions) -> MdResult<Config> {
    let config_file = config_path(&opts);
    if config_file.exists() {
        let mut config = read_config(&config_file)?;
        let mut changed = false;
        if let Some(root) = opts.root_override {
            config.root = root;
            changed = true;
        }
        if let Some(remote) = opts.remote_override {
            config.remote = Some(remote);
            changed = true;
        }
        if let Some(editor) = opts.editor_override {
            config.editor = Some(editor);
            changed = true;
        }
        if changed {
            write_config(&config_file, &config)?;
        }
        ensure_directories(&config.root)?;
        if let Some(remote) = &config.remote {
            ensure_remote_configured(&config.root, remote)?;
        }
        return Ok(config);
    }

    let root_override = opts.root_override.clone();
    let root = root_override
        .or_else(|| env::var_os(ROOT_OVERRIDE_ENV).map(PathBuf::from))
        .unwrap_or_else(|| config_home(&opts).join(DEFAULT_ROOT_DIR));
    let config = Config {
        root,
        remote: opts.remote_override.clone(),
        editor: opts.editor_override.clone(),
    };
    write_config(&config_file, &config)?;
    ensure_directories(&config.root)?;
    initialize_git(&config.root)?;
    if let Some(remote) = &config.remote {
        ensure_remote_configured(&config.root, remote)?;
    }
    Ok(config)
}

pub fn save_config(opts: &SetupOptions, config: &Config) -> MdResult<()> {
    let config_file = config_path(opts);
    write_config(&config_file, config)?;
    ensure_directories(&config.root)?;
    if let Some(remote) = &config.remote {
        ensure_remote_configured(&config.root, remote)?;
    }
    Ok(())
}

pub fn config_path(opts: &SetupOptions) -> PathBuf {
    config_home(opts).join(CONFIG_NAME)
}

fn config_home(opts: &SetupOptions) -> PathBuf {
    if let Some(custom) = &opts.config_home {
        return custom.clone();
    }
    if let Some(override_dir) = env::var_os(CONFIG_OVERRIDE_ENV) {
        return PathBuf::from(override_dir);
    }
    if cfg!(target_os = "windows") {
        if let Some(appdata) = env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("mdnotes");
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = dirs_home() {
            return home
                .join("Library")
                .join("Application Support")
                .join("mdnotes");
        }
    }
    dirs_home()
        .map(|home| home.join(".config").join("mdnotes"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn dirs_home() -> Option<PathBuf> {
    if let Some(home) = env::var_os("HOME") {
        return Some(PathBuf::from(home));
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(profile) = env::var_os("USERPROFILE") {
            return Some(PathBuf::from(profile));
        }
    }
    None
}

fn write_config(path: &Path, config: &Config) -> MdResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(file, "root={}", config.root.display())?;
    if let Some(remote) = &config.remote {
        writeln!(file, "remote={remote}")?;
    }
    if let Some(editor) = &config.editor {
        writeln!(file, "editor={editor}")?;
    }
    Ok(())
}

fn read_config(path: &Path) -> MdResult<Config> {
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;

    let mut root: Option<PathBuf> = None;
    let mut remote: Option<String> = None;
    let mut editor: Option<String> = None;
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "root" => root = Some(PathBuf::from(value.trim())),
                "remote" => remote = Some(value.trim().to_string()),
                "editor" => editor = Some(value.trim().to_string()),
                _ => {}
            }
        }
    }
    let root = root.ok_or_else(|| MdError("Invalid config: missing root".into()))?;
    Ok(Config {
        root,
        remote,
        editor,
    })
}

pub fn ensure_directories(root: &Path) -> MdResult<()> {
    fs::create_dir_all(root)?;
    Ok(())
}

fn initialize_git(root: &Path) -> MdResult<()> {
    if root.join(".git").exists() {
        return Ok(());
    }
    let output = Command::new("git").arg("init").arg(root).output()?;
    if !output.status.success() {
        return Err(MdError(format!(
            "git init failed with status {}",
            output.status
        )));
    }
    Ok(())
}

fn ensure_remote_configured(root: &Path, remote: &str) -> MdResult<()> {
    let has_origin = Command::new("git")
        .current_dir(root)
        .args(["remote", "get-url", "origin"])
        .output()?;
    if has_origin.status.success() {
        let out = Command::new("git")
            .current_dir(root)
            .args(["remote", "set-url", "origin", remote])
            .output()?;
        if !out.status.success() {
            return Err(MdError(format!(
                "git remote set-url failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }
    } else {
        let out = Command::new("git")
            .current_dir(root)
            .args(["remote", "add", "origin", remote])
            .output()?;
        if !out.status.success() {
            return Err(MdError(format!(
                "git remote add failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }
    }
    Ok(())
}
