use crate::models::Item;
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
    PrioEq(u32),
    /// Matches items whose priority is greater than the given value.
    PrioGt(u32),
    /// Matches items whose priority is less than the given value.
    PrioLt(u32),
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
            Predicate::PrioEq(p) => item.priority == Some(*p),
            Predicate::PrioGt(p) => item.priority.map(|v| v > *p).unwrap_or(false),
            Predicate::PrioLt(p) => item.priority.map(|v| v < *p).unwrap_or(false),
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

/// Tokenises a query string.  Parentheses are split off from adjacent tokens so
/// that `(.task and #foo)` becomes `["(", ".task", "and", "#foo", ")"]`.
fn tokenize(query: &str) -> Vec<String> {
    let spaced = query.replace('(', " ( ").replace(')', " ) ");
    spaced.split_whitespace().map(|s| s.to_string()).collect()
}

struct Parser {
    tokens: Vec<String>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<String>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|s| s.as_str())
    }

    fn consume(&mut self) -> Option<&str> {
        let tok = self.tokens.get(self.pos).map(|s| s.as_str());
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    /// or_expr := and_expr ('or' and_expr)*
    fn parse_or(&mut self) -> MdResult<Predicate> {
        let mut left = self.parse_and()?;
        while self.peek() == Some("or") {
            self.consume();
            let right = self.parse_and()?;
            left = Predicate::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// and_expr := not_expr ('and' not_expr)*
    fn parse_and(&mut self) -> MdResult<Predicate> {
        let mut left = self.parse_not()?;
        while self.peek() == Some("and") {
            self.consume();
            let right = self.parse_not()?;
            left = Predicate::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// not_expr := 'not' not_expr | primary
    fn parse_not(&mut self) -> MdResult<Predicate> {
        if self.peek() == Some("not") {
            self.consume();
            let inner = self.parse_not()?;
            return Ok(Predicate::Not(Box::new(inner)));
        }
        self.parse_primary()
    }

    /// primary := '(' or_expr ')' | leaf
    fn parse_primary(&mut self) -> MdResult<Predicate> {
        if self.peek() == Some("(") {
            self.consume(); // consume '('
            let inner = self.parse_or()?;
            if self.peek() != Some(")") {
                return Err(MdError("Expected ')' to close parenthesis".into()));
            }
            self.consume(); // consume ')'
            return Ok(inner);
        }
        self.parse_leaf()
    }

    fn parse_leaf(&mut self) -> MdResult<Predicate> {
        let token = self
            .consume()
            .ok_or_else(|| MdError("Unexpected end of query".into()))?;
        parse_leaf_token(token)
    }
}

fn parse_leaf_token(token: &str) -> MdResult<Predicate> {
    if token == ".task" {
        return Ok(Predicate::IsTask);
    }
    if let Some(tag) = token.strip_prefix('#') {
        if tag.is_empty() {
            return Err(MdError("Tag name after '#' cannot be empty".into()));
        }
        return Ok(Predicate::HasTag(tag.to_string()));
    }
    if let Some(prio_str) = token.strip_prefix("prio:") {
        if let Some(rest) = prio_str.strip_prefix('>') {
            let n = rest
                .parse::<u32>()
                .map_err(|_| MdError(format!("Invalid priority number: '{rest}'")))?;
            return Ok(Predicate::PrioGt(n));
        }
        if let Some(rest) = prio_str.strip_prefix('<') {
            let n = rest
                .parse::<u32>()
                .map_err(|_| MdError(format!("Invalid priority number: '{rest}'")))?;
            return Ok(Predicate::PrioLt(n));
        }
        let n = prio_str
            .parse::<u32>()
            .map_err(|_| MdError(format!("Invalid priority number: '{prio_str}'")))?;
        return Ok(Predicate::PrioEq(n));
    }
    if let Some(due_str) = token.strip_prefix("due:") {
        if let Some(rest) = due_str.strip_prefix('>') {
            return Ok(Predicate::DueAfter(normalize_due_date(rest)));
        }
        if let Some(rest) = due_str.strip_prefix('<') {
            return Ok(Predicate::DueBefore(normalize_due_date(rest)));
        }
        return Ok(Predicate::DueIs(normalize_due_date(due_str)));
    }
    Err(MdError(format!("Unknown filter token: '{token}'")))
}

/// Parses a query string into a `Predicate` using an infix expression parser.
///
/// Tokens are space-separated.  Operators use standard infix notation:
/// - Filter tokens: `.task`, `#<tag>`, `prio:<n>`, `prio:><n>`, `prio:<<n>`,
///   `due:<yyyymmdd>`, `due:><yyyymmdd>`, `due:<<yyyymmdd>`
/// - `and` – logical AND (lower precedence than `not`, higher than `or`)
/// - `or`  – logical OR (lowest precedence)
/// - `not` – logical NOT (prefix, highest precedence)
/// - `(` / `)` – grouping; may be attached to adjacent tokens
///
/// Example: `.task and #urgent`, `(.task or #note) and prio:>3`
///
/// An empty query returns [`Predicate::All`].
pub fn parse_query(query: &str) -> MdResult<Predicate> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Predicate::All);
    }

    let tokens = tokenize(trimmed);
    let mut parser = Parser::new(tokens);
    let pred = parser.parse_or()?;
    if parser.peek().is_some() {
        return Err(MdError(format!(
            "Unexpected token '{}' in query",
            parser.peek().unwrap()
        )));
    }
    Ok(pred)
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
