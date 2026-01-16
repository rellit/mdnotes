use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::models::{Priority, Status};
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
    /// Show or update configuration
    Config(ConfigArgs),
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
    /// Search notes and tasks by content or title
    #[command(visible_aliases = ["f", "search"])]
    Find(FindArgs),
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
    pub title: String,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long, value_parser = validate_due)]
    pub due: Option<String>,
    #[arg(long, value_enum)]
    pub status: Option<Status>,
    #[arg(long, value_enum)]
    pub priority: Option<Priority>,
    #[arg(long)]
    pub tags: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ListTarget {
    All,
    #[value(alias = "n")]
    Notes,
    #[value(alias = "t")]
    Tasks,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(
        value_enum,
        value_name = "target",
        help = "notes|tasks",
        required = false
    )]
    pub target: Option<ListTarget>,
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

#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// Optional custom root directory
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// Optional remote git repository url
    #[arg(long)]
    pub remote: Option<String>,
    /// Optional editor to use when editing notes
    #[arg(long)]
    pub editor: Option<String>,
}

#[derive(Args, Debug)]
pub struct FindArgs {
    pub query: String,
    #[arg(
        value_enum,
        value_name = "target",
        help = "notes|tasks",
        required = false
    )]
    pub target: Option<ListTarget>,
}
