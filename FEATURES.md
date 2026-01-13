# Features to Implement

This document tracks the features planned for mdnotes - a command-line/TUI application for taking notes and managing tasks.

## Core Features

### Setup
- We'll keep our config in a config File named 'mdnrc' in a place suitable for the current OS. For Linux use ~/.config/mdnotes. For Windows and Mac use proper dirs.
- When a command gets executed, run a CLI Setup to create the File.
- We need a root Folder to store the notes.
- We need a Repo to sync to (try it, it should be empty in first place?).
- All other questions/settings that are needed


### Note Management
- Create new notes
- Edit existing notes
- Delete notes
- List all notes
- Search notes by content or title
- Tag notes for organization

### Task Management
- Create tasks with due dates
- Mark tasks as complete/incomplete
- List tasks by status (pending, completed)
- Filter tasks by priority
- Set task priorities (low, medium, high)

### Storage
- Store notes in markdown format
  - Notes/Tasks get Stored as a single Markdown File.
  - We'll use a Header to Track Title, due, priority and other Attributes.
  - Task or Note description starts after a line '--'.
- Local file-based storage
  - Each File gets a UUID as name.
  - The Root of our Directory will contain a folder 'notes' for Notes and a folder 'tasks' for tasks.
  - Tags will be implemented in a dir 'tags' With subdirectories and symlinks to notes/tasks. NAme of Symlink is Just UUID of target.
    - If this is not possible for win, we need another solution or we'll drop win support.
- Git integration for syncing notes across devices
- When executing any command, git should sync first. So there should be no to few conflicts.
- When git remains in a conflicting State, there should be a UI in TUI for that.
- Automatic commits when notes are modified

### User Interface
- Command-line interface (CLI) for quick operations
  - Named 'mdn'.
  - Should provide a fast command experience such as:
  - mdn add note 'Note Title' -- This should also be usable as mdn a n
  - mdn add task 'Task Title' -- This should also be usable as mdn a t
  - mdn edit uuid -- mdn e -- This should open a Editor for editing. tag symlinks should get updated after closing.
  - all referneces to uuid should work, if a unique prefix is given, like in git commit hashes. If not unique, it should give us the selection.
- Text-based User Interface (TUI) for interactive browsing
  - Named mdnui.
  - Lets us browse Notes and Tasks in 2 'Tabs'.
  - Preview of selected Task/Node
  - Fast editing/tagging
- Syntax highlighting for markdown in preview

### Cross-Platform Support
- Windows support
- Linux support
- macOS support

## Future Enhancements

### Advanced Features
- CLI command completion for: zsh/bash/fish
- Note templates
- Export notes to different formats (PDF, HTML)
- Encryption for sensitive notes
- Full-text search with indexing
- Note linking and backlinks
- Daily notes / journal mode
- Attachments support

---

**Note:** This is a living document. Features will be prioritized and moved to TODO.md as they are planned for implementation.
