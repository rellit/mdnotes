# TODO

This file tracks implementation tasks for mdnotes.

---

## Core Features

### Setup
- [ ] Create `mdnrc` config file in platform-appropriate location and run setup on first command.
- [ ] Establish root folder for notes and initialize/sync with remote repo (initially empty).
- [ ] Gather remaining required settings during setup.

### Note Management
- [ ] Create new notes.
- [ ] Edit existing notes.
- [ ] Delete notes.
- [ ] List all notes.
- [ ] Search notes by content or title.
- [ ] Tag notes for organization.

### Task Management
- [ ] Create tasks with due dates.
- [ ] Mark tasks as complete/incomplete.
- [ ] List tasks by status (pending, completed).
- [ ] Set and filter tasks by priority (low, medium, high).

### Storage
- [ ] Store notes/tasks as markdown with header for metadata and body after `--`.
- [ ] Save files with UUID names under `notes/` and `tasks/` directories.
- [ ] Implement tag directory with symlinks (or alternative for Windows) pointing to note/task UUIDs.
- [ ] Add git sync before commands, UI for conflicts, and automatic commits on modifications.

### User Interface
- [ ] Provide `mdn` CLI with short aliases and UUID-prefix support for commands (add/edit).
- [ ] Build `mdnui` TUI with notes/tasks tabs, preview pane, fast editing/tagging, and markdown syntax highlighting.

### Cross-Platform Support
- [ ] Validate support on Linux, macOS, and Windows (adjust symlink strategy as needed).
