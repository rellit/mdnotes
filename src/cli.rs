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
    /// List items, optionally filtered by a query string
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
    /// Set or clear due date
    Due {
        id: String,
        #[arg(value_parser = validate_due)]
        due: Option<String>,
    },
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

/// Arguments for the `list` command.
///
/// The optional query string uses a stack-based postfix filter language:
///
///   `.task`           – item has a due date (is a task)  
///   `#<tag>`          – item has the given tag  
///   `prio:<value>`    – item priority is low|medium|high  
///   `due:<yyyymmdd>`  – item due date equals (8-digit compact or YYYY-MM-DD)  
///   `due:><yyyymmdd>` – item due date is after  
///   `due:<<yyyymmdd>` – item due date is before  
///   `and`             – logical AND of the two top predicates  
///   `or`              – logical OR of the two top predicates  
///   `not`             – logical NOT of the top predicate  
///
/// Multiple tokens without explicit operators are implicitly ANDed together.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Optional query string to filter items (e.g. ".task #ui and")
    #[arg(value_name = "query", required = false)]
    pub query: Option<String>,
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
    pub target: Option<FindTarget>,
}

/// Filter target for the `find` command.
#[derive(ValueEnum, Clone, Debug)]
pub enum FindTarget {
    All,
    #[value(alias = "n")]
    Notes,
    #[value(alias = "t")]
    Tasks,
}
