use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::models::{ItemKind, Priority, Status};
use crate::util::validate_due;

#[derive(Parser, Debug)]
#[command(name = "mdn", version, about = "mdnotes - note and task manager")]
pub struct Cli {
    /// Override config home (for testing)
    #[arg(long, hide = true)]
    pub config_home: Option<PathBuf>,

    /// Override root directory (for testing)
    #[arg(long, hide = true)]
    pub root_override: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize configuration and storage
    Setup {
        /// Optional custom root directory
        #[arg(long)]
        root: Option<PathBuf>,
        /// Optional remote git repository url
        #[arg(long)]
        remote: Option<String>,
    },
    /// Create a new note or task
    #[command(visible_alias = "a")]
    Add(AddArgs),
    /// List notes or tasks
    #[command(visible_aliases = ["ls", "l"])]
    List(ListArgs),
    /// Delete a note or task by id/prefix
    #[command(visible_aliases = ["d", "del"])]
    Delete { id: String },
    /// Edit an existing note or task
    #[command(visible_alias = "e")]
    Edit(EditArgs),
    /// Search notes by content or title
    #[command(visible_alias = "find")]
    Search { query: String },
    /// Mark task complete
    #[command(visible_alias = "c")]
    Complete { id: String },
    /// Mark task incomplete
    #[command(visible_alias = "ic")]
    Incomplete { id: String },
    /// Show full item content
    #[command(visible_aliases = ["sh", "s"])]
    Show { id: String },
}

#[derive(Args, Debug)]
pub struct AddArgs {
    #[arg(value_enum, value_name = "kind", help = "note|task")]
    pub kind: ItemKind,
    pub title: String,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long, value_parser = validate_due)]
    pub due: Option<String>,
    #[arg(long, value_enum)]
    pub priority: Option<Priority>,
    #[arg(long)]
    pub tags: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ListTarget {
    Notes,
    Tasks,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(value_enum, value_name = "target", help = "notes|tasks")]
    pub target: ListTarget,
    #[arg(long, value_enum)]
    pub status: Option<Status>,
    #[arg(long, value_enum)]
    pub priority: Option<Priority>,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    pub id: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long, value_parser = validate_due)]
    pub due: Option<String>,
    #[arg(long, value_enum)]
    pub priority: Option<Priority>,
    #[arg(long, value_enum)]
    pub status: Option<Status>,
    #[arg(long)]
    pub tags: Option<String>,
}
