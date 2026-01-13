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
}

#[derive(Clone, Debug, Default)]
pub struct SetupOptions {
    pub root_override: Option<PathBuf>,
    pub config_home: Option<PathBuf>,
}

pub fn ensure_setup(opts: SetupOptions) -> MdResult<Config> {
    let config_file = config_path(&opts);
    if config_file.exists() {
        let mut config = read_config(&config_file)?;
        if let Some(root) = opts.root_override {
            config.root = root;
            write_config(&config_file, &config)?;
        }
        ensure_directories(&config.root)?;
        return Ok(config);
    }

    let root_override = opts.root_override.clone();
    let root = root_override
        .or_else(|| env::var_os(ROOT_OVERRIDE_ENV).map(PathBuf::from))
        .unwrap_or_else(|| config_home(&opts).join(DEFAULT_ROOT_DIR));
    let config = Config { root, remote: None };
    write_config(&config_file, &config)?;
    ensure_directories(&config.root)?;
    initialize_git(&config.root)?;
    Ok(config)
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
    Ok(())
}

fn read_config(path: &Path) -> MdResult<Config> {
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;

    let mut root: Option<PathBuf> = None;
    let mut remote: Option<String> = None;
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "root" => root = Some(PathBuf::from(value.trim())),
                "remote" => remote = Some(value.trim().to_string()),
                _ => {}
            }
        }
    }
    let root = root.ok_or_else(|| MdError("Invalid config: missing root".into()))?;
    Ok(Config { root, remote })
}

pub fn ensure_directories(root: &Path) -> MdResult<()> {
    fs::create_dir_all(root.join("notes"))?;
    fs::create_dir_all(root.join("tasks"))?;
    fs::create_dir_all(root.join("tags"))?;
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
