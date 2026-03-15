use clap::ValueEnum;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Status {
    Pending,
    Completed,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ItemKind {
    Note,
    Task,
}

impl ItemKind {
    /// Display label used in the TUI and list headers.
    pub fn dir_name(&self) -> &'static str {
        match self {
            ItemKind::Note => "notes",
            ItemKind::Task => "tasks",
        }
    }

    /// Infers the kind of an item from its metadata.
    /// An item is a task if and only if it has a due date.
    pub fn infer(_status: &Option<Status>, due: &Option<String>) -> ItemKind {
        if due.is_some() {
            ItemKind::Task
        } else {
            ItemKind::Note
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub kind: ItemKind,
    pub body: String,
    pub tags: Vec<String>,
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub due: Option<String>,
}

impl Item {
    /// Returns `true` when this item has a due date, which makes it a task.
    pub fn is_task(&self) -> bool {
        self.due.is_some()
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# {}", self.title)?;
        if let Some(status) = &self.status {
            writeln!(f, "status: {}", status.as_str())?;
        }
        if let Some(priority) = &self.priority {
            writeln!(f, "priority: {}", priority.as_str())?;
        }
        if let Some(due) = &self.due {
            writeln!(f, "due: {due}")?;
        }
        if !self.tags.is_empty() {
            writeln!(f, "tags: {}", self.tags.join(", "))?;
        }
        writeln!(f, "--")?;
        write!(f, "{}", self.body)?;
        Ok(())
    }
}
