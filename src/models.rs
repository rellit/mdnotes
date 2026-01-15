use clap::ValueEnum;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
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

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
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

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum ItemKind {
    Note,
    Task,
}

impl ItemKind {
    pub fn dir_name(&self) -> &'static str {
        match self {
            ItemKind::Note => "notes",
            ItemKind::Task => "tasks",
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
