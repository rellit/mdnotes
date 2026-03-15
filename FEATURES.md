# mdnotes — Feature Reference

This document describes the features that are currently implemented in mdnotes.

---

## Configuration

- Platform-appropriate config file (`mdnrc`) stored at:
  - **Linux**: `~/.config/mdnotes/mdnrc`
  - **macOS / Windows**: OS-standard config directory
- Interactive first-run setup via `mdn config`
  - Configures the note root directory
  - Configures an optional remote git repository for syncing
  - Configures the editor to use when opening items
- Override paths via `--config-home` / `--root-override` (useful for testing)

---

## Note & Task Management

- **Create** notes and tasks (`mdn add` / `mdn a`)
  - Optional `--due`, `--status`, `--priority`, `--tags`, `--body` flags
- **Edit** an existing item (`mdn edit` / `mdn e <id>`)
  - Opens `$EDITOR` when called without extra flags
  - Supports `--title`, `--due`, `--status`, `--priority`, `--tags`, `--body` flags
- **Delete** an item (`mdn delete` / `mdn d <id>`)
- **Show** full item content (`mdn show` / `mdn s <id>`)
- **List** all items (`mdn list` / `mdn ls`)
  - Optional filter query (see [Filtering](#filtering) below)
  - Results are grouped into Notes and Tasks
- **Mark complete / incomplete** (`mdn complete` / `mdn c <id>`, `mdn incomplete` / `mdn ic <id>`)
- **Set due date** (`mdn due <id> [date]`)
- **Set priority** (`mdn priority` / `mdn p <id> [value]`)

### Tasks vs Notes

Any item that has a due date is automatically treated as a **task**. Notes without a due date remain plain notes. Priority can be set on both.

---

## Filtering

The `mdn list` command accepts an optional infix query string with standard operator precedence (`not` > `and` > `or`). Parentheses are supported.

| Token | Meaning |
|---|---|
| `.task` | Item has a due date (is a task) |
| `#<tag>` | Item carries the given tag |
| `title:<substring>` | Title contains the given substring |
| `tagged` | Item has at least one tag |
| `prio:<n>` | Priority equals n |
| `prio:><n>` | Priority greater than n |
| `prio:<<n>` | Priority less than n |
| `prio:>=<n>` | Priority greater than or equal to n |
| `prio:<=<n>` | Priority less than or equal to n |
| `due:<yyyymmdd>` | Due date equals (compact YYYYMMDD or YYYY-MM-DD) |
| `due:><date>` | Due date is after date |
| `due:<<date>` | Due date is before date |
| `due:>=<date>` | Due date is on or after date |
| `due:<=<date>` | Due date is on or before date |
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT (prefix) |

**Examples:**

```
mdn list .task
mdn list ".task and #urgent"
mdn list "(.task or #note) and prio:>3"
mdn list "not .task and tagged"
mdn list "due:<=20260401"
```

---

## Storage

- Each item is stored as a Markdown file at `<root>/<uuid>/MAIN.md`
- Metadata (title, due, priority, status, tags) is stored in a YAML-like header; body follows after a `--` separator
- UUIDs can be abbreviated to any unique prefix in all commands (similar to git commit hashes)

---

## Git Sync

- On every mutating command (edit, delete, complete, due, priority) mdnotes pulls from the remote before making changes and commits + pushes afterwards
- `mdn sync` — manual pull + push

---

## User Interface

### CLI (`mdn`)

All commands have short aliases:

| Command | Aliases |
|---|---|
| `add` | `a` |
| `list` | `ls`, `l` |
| `delete` | `d`, `del` |
| `edit` | `e` |
| `complete` | `c` |
| `incomplete` | `ic` |
| `show` | `sh`, `s` |
| `priority` | `p` |

### TUI (`mdnui`)

- Browse all notes and tasks in an interactive terminal UI
- Preview selected item in Markdown
- Keyboard-driven navigation

---

## Cross-Platform Support

| Platform | Binaries |
|---|---|
| Linux x86_64 | `mdn`, `mdnui` |
| macOS x86_64 | `mdn`, `mdnui` |
| macOS ARM64 | `mdn`, `mdnui` |
| Windows x86_64 | `mdn.exe`, `mdnui.exe` |
