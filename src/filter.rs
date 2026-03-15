use crate::models::{Item, Priority};
use crate::{MdError, MdResult};

/// A composable predicate for filtering items.
#[derive(Debug)]
pub enum Predicate {
    /// Matches all items (empty query).
    All,
    /// Matches items that are tasks (have a due date).
    IsTask,
    /// Matches items that have the given tag.
    HasTag(String),
    /// Matches items whose priority equals the given value.
    PrioIs(Priority),
    /// Matches items whose due date equals the given value (stored as YYYY-MM-DD).
    DueIs(String),
    /// Matches items whose due date is after the given value.
    DueAfter(String),
    /// Matches items whose due date is before the given value.
    DueBefore(String),
    /// Logical AND of two predicates.
    And(Box<Predicate>, Box<Predicate>),
    /// Logical OR of two predicates.
    Or(Box<Predicate>, Box<Predicate>),
    /// Logical NOT of a predicate.
    Not(Box<Predicate>),
}

impl Predicate {
    /// Returns `true` if the item matches this predicate.
    pub fn matches(&self, item: &Item) -> bool {
        match self {
            Predicate::All => true,
            Predicate::IsTask => item.is_task(),
            Predicate::HasTag(tag) => item.tags.iter().any(|t| t == tag),
            Predicate::PrioIs(p) => item.priority.as_ref() == Some(p),
            Predicate::DueIs(d) => item.due.as_deref() == Some(d.as_str()),
            Predicate::DueAfter(d) => item
                .due
                .as_ref()
                .map(|id| id.as_str() > d.as_str())
                .unwrap_or(false),
            Predicate::DueBefore(d) => item
                .due
                .as_ref()
                .map(|id| id.as_str() < d.as_str())
                .unwrap_or(false),
            Predicate::And(a, b) => a.matches(item) && b.matches(item),
            Predicate::Or(a, b) => a.matches(item) || b.matches(item),
            Predicate::Not(p) => !p.matches(item),
        }
    }
}

/// Parses a query string into a `Predicate` using a stack-based (postfix) evaluator.
///
/// Tokens are space-separated and evaluated left-to-right:
/// - Filter tokens (`.task`, `#tag`, `prio:value`, `due:value`, `due:>value`, `due:<value`)
///   push a new predicate onto the stack.
/// - `and` pops two predicates and pushes their conjunction.
/// - `or` pops two predicates and pushes their disjunction.
/// - `not` pops one predicate and pushes its negation.
///
/// If multiple predicates remain on the stack after processing, they are implicitly
/// combined with AND.
///
/// An empty query returns [`Predicate::All`].
pub fn parse_query(query: &str) -> MdResult<Predicate> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Predicate::All);
    }

    let mut stack: Vec<Predicate> = Vec::new();

    for token in trimmed.split_whitespace() {
        match token {
            "and" => {
                let b = stack
                    .pop()
                    .ok_or_else(|| MdError("'and' requires two operands".into()))?;
                let a = stack
                    .pop()
                    .ok_or_else(|| MdError("'and' requires two operands".into()))?;
                stack.push(Predicate::And(Box::new(a), Box::new(b)));
            }
            "or" => {
                let b = stack
                    .pop()
                    .ok_or_else(|| MdError("'or' requires two operands".into()))?;
                let a = stack
                    .pop()
                    .ok_or_else(|| MdError("'or' requires two operands".into()))?;
                stack.push(Predicate::Or(Box::new(a), Box::new(b)));
            }
            "not" => {
                let a = stack
                    .pop()
                    .ok_or_else(|| MdError("'not' requires one operand".into()))?;
                stack.push(Predicate::Not(Box::new(a)));
            }
            ".task" => stack.push(Predicate::IsTask),
            t if t.starts_with('#') => {
                let tag = t[1..].to_string();
                if tag.is_empty() {
                    return Err(MdError("Tag name after '#' cannot be empty".into()));
                }
                stack.push(Predicate::HasTag(tag));
            }
            t if t.starts_with("prio:") => {
                let prio_str = &t[5..];
                let prio = match prio_str {
                    "low" => Priority::Low,
                    "medium" => Priority::Medium,
                    "high" => Priority::High,
                    other => {
                        return Err(MdError(format!("Unknown priority value: '{other}'")));
                    }
                };
                stack.push(Predicate::PrioIs(prio));
            }
            t if t.starts_with("due:") => {
                let due_str = &t[4..];
                if let Some(rest) = due_str.strip_prefix('>') {
                    stack.push(Predicate::DueAfter(normalize_due_date(rest)));
                } else if let Some(rest) = due_str.strip_prefix('<') {
                    stack.push(Predicate::DueBefore(normalize_due_date(rest)));
                } else {
                    stack.push(Predicate::DueIs(normalize_due_date(due_str)));
                }
            }
            other => {
                return Err(MdError(format!("Unknown filter token: '{other}'")));
            }
        }
    }

    if stack.is_empty() {
        return Ok(Predicate::All);
    }

    // Combine remaining stack items with implicit AND
    let mut result = stack.remove(0);
    for pred in stack {
        result = Predicate::And(Box::new(result), Box::new(pred));
    }
    Ok(result)
}

/// Converts an 8-digit `yyyymmdd` string to the `YYYY-MM-DD` format used in stored items.
/// If the input already contains hyphens (e.g. `YYYY-MM-DD`) or is not exactly 8 ASCII digits,
/// it is returned unchanged.
fn normalize_due_date(s: &str) -> String {
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8])
    } else {
        s.to_string()
    }
}
