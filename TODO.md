# TODO

This file tracks implementation tasks for mdnotes.

---

## Core Features

### Setup
- [x] Create `mdnrc` config file in platform-appropriate location and run setup on first command.
- [ ] Establish root folder for notes and initialize/sync with remote repo (initially empty).
- [ ] Gather remaining required settings during setup.

### Note Management
- [x] Create new notes.
- [x] Edit existing notes.
- [x] Delete notes.
- [x] List all notes.
- [x] Search notes by content or title.
- [x] Tag notes for organization.

### Task Management
- [x] Create tasks with due dates.
- [x] Mark tasks as complete/incomplete.
- [x] List tasks by status (pending, completed).
- [x] Set and filter tasks by priority (low, medium, high).

### Storage
- [x] Store notes/tasks as markdown with header for metadata and body after `--`.
- [x] Save files with UUID names under `notes/` and `tasks/` directories.
- [x] Implement tag directory with symlinks (or alternative for Windows) pointing to note/task UUIDs.
- [ ] Add git sync before commands, UI for conflicts, and automatic commits on modifications.

### User Interface
- [x] Provide `mdn` CLI with short aliases and UUID-prefix support for commands (add/edit).
- [ ] Build `mdnui` TUI with notes/tasks tabs, preview pane, fast editing/tagging, and markdown syntax highlighting.

### Cross-Platform Support
- [ ] Validate support on Linux, macOS, and Windows (adjust symlink strategy as needed).
