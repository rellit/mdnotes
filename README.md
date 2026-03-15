# mdnotes

A fast, keyboard-driven command-line and TUI application for taking notes and managing tasks, with Markdown storage and git-based syncing across devices.

## Features

- **Markdown storage** — every note/task is a plain Markdown file you can open in any editor  
- **Tasks from notes** — any item with a due date automatically becomes a task  
- **Priority & tagging** — set numeric priorities and free-form `#tags` on any item  
- **Git sync** — automatic commit + push/pull on every command when a remote is configured  
- **Short CLI aliases** — `mdn a`, `mdn e <id>`, `mdn d <id>`, prefix-based UUID matching  
- **Interactive TUI** — browse notes and tasks, preview Markdown, fast editing (`mdnui`)  
- **Cross-platform** — Linux (x86_64), macOS (x86_64 & ARM64), Windows (x86_64)

See [FEATURES.md](FEATURES.md) for a detailed feature reference.

## Installation

### Linux

```sh
curl -fsSL https://raw.githubusercontent.com/rellit/mdnotes/main/install.sh | sh
```

Binaries (`mdn`, `mdnui`) are installed to `~/.local/bin`.  
If that directory is not yet in your `PATH`, the installer will tell you what to add to your shell profile.

### macOS

```sh
curl -fsSL https://raw.githubusercontent.com/rellit/mdnotes/main/install.sh | sh
```

Binaries (`mdn`, `mdnui`) are installed to `~/.local/bin`.

### Windows

Open **PowerShell** and run:

```powershell
iwr -useb https://raw.githubusercontent.com/rellit/mdnotes/main/install.ps1 | iex
```

Binaries (`mdn.exe`, `mdnui.exe`) are installed to `%LOCALAPPDATA%\mdnotes\bin`, which is added to your user `PATH` automatically.

### Manual download

Download the pre-built archive for your platform from the [releases page](https://github.com/rellit/mdnotes/releases) and extract the binaries to any directory on your `PATH`.

| Platform | Archive |
|---|---|
| Linux x86_64 | `mdnotes-linux-x86_64.tar.gz` |
| macOS x86_64 | `mdnotes-macos-x86_64.tar.gz` |
| macOS ARM64 | `mdnotes-macos-aarch64.tar.gz` |
| Windows x86_64 | `mdnotes-windows-x86_64.zip` |

### From source

```sh
git clone https://github.com/rellit/mdnotes.git
cd mdnotes
cargo build --release
# binaries: target/release/mdn  target/release/mdnui
```

## Quick start

```sh
# First run — interactive setup (config file + note root + optional git remote)
mdn config

# Add a note
mdn add "My first note"

# Add a task with a due date
mdn add "Fix the bug" --due 20260320

# List everything
mdn list

# List only tasks
mdn list .task

# Edit a note (opens $EDITOR)
mdn edit <uuid-or-prefix>

# Open the interactive TUI
mdnui
```

Run `mdn --help` for the full command reference.

## Development

### Prerequisites

- Rust 1.70 or later

### Build & test

```sh
cargo build
cargo test
cargo fmt
cargo clippy
```

## License

MIT — see [LICENSE](LICENSE).
