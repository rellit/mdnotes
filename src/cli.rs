use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::models::Status;
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

    /// Show full item IDs instead of shortened unique prefixes
    #[arg(long)]
    pub verbose: bool,

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
    /// Set item priority
    #[command(visible_alias = "p")]
    Priority(PriorityArgs),
    /// Sync with remote (pull then push)
    Sync,
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
    #[arg(long)]
    pub priority: Option<u32>,
    #[arg(long)]
    pub tags: Option<String>,
}

/// Arguments for the `list` command.
///
/// The optional query string uses an infix filter language with standard
/// operator precedence (`not` > `and` > `or`).  Parentheses are supported.
///
///   `.task`           – item has a due date (is a task)
///   `#<tag>`          – item has the given tag
///   `prio:<n>`        – item priority equals n
///   `prio:><n>`       – item priority is greater than n
///   `prio:<<n>`       – item priority is less than n
///   `due:<yyyymmdd>`  – item due date equals (8-digit compact or YYYY-MM-DD)
///   `due:><yyyymmdd>` – item due date is after
///   `due:<<yyyymmdd>` – item due date is before
///   `and`             – logical AND (infix)
///   `or`              – logical OR (infix)
///   `not`             – logical NOT (prefix)
///
/// Examples: `.task and #urgent`, `(.task or #note) and prio:>3`
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Optional query string to filter items (e.g. ".task and #ui")
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
    #[arg(long)]
    pub priority: Option<u32>,
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
pub struct PriorityArgs {
    /// Item id or prefix
    pub id: String,
    /// Priority value (higher = more important); omit to clear
    pub value: Option<u32>,
}
