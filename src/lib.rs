pub mod cli;
pub mod commands;
pub mod config;
pub mod models;
pub mod storage;
pub mod tags;
pub mod util;

use clap::Parser;
pub use cli::{Cli, Commands};
pub use models::{Item, ItemKind, Priority, Status};
pub use util::{parse_tags, validate_due};

#[derive(Debug)]
pub struct MdError(pub String);

pub type MdResult<T> = Result<T, MdError>;

impl std::fmt::Display for MdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for MdError {}

impl From<std::io::Error> for MdError {
    fn from(value: std::io::Error) -> Self {
        MdError(value.to_string())
    }
}

impl From<String> for MdError {
    fn from(value: String) -> Self {
        MdError(value)
    }
}

impl From<&str> for MdError {
    fn from(value: &str) -> Self {
        MdError(value.to_string())
    }
}

pub fn run() -> MdResult<()> {
    let cli = Cli::parse();
    let lines = dispatch(cli)?;
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

pub fn run_with_args<I, S>(args: I) -> MdResult<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    dispatch(cli)
}

fn dispatch(cli: Cli) -> MdResult<Vec<String>> {
    let setup_opts = config::SetupOptions {
        root_override: cli.root_override.clone(),
        config_home: cli.config_home.clone(),
    };
    match cli.command {
        Commands::Setup { root } => commands::setup::run(root, setup_opts),
        Commands::Add(args) => commands::add::run(args, setup_opts),
        Commands::List(args) => commands::list::run(args, setup_opts),
        Commands::Delete { id } => commands::delete::run(id, setup_opts),
        Commands::Edit(args) => commands::edit::run(args, setup_opts),
        Commands::Search { query } => commands::search::run(query, setup_opts),
        Commands::Complete { id } => commands::complete::run(id, true, setup_opts),
        Commands::Incomplete { id } => commands::complete::run(id, false, setup_opts),
        Commands::Show { id } => commands::show::run(id, setup_opts),
    }
}
