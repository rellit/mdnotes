# mdnotes

A simple command-line/TUI application for taking notes and managing tasks, with markdown support and git-based syncing.

## Features

See [FEATURES.md](FEATURES.md) for a comprehensive list of planned features.

## Installation

### From Release

Download the latest release for your platform from the [releases page](https://github.com/rellit/mdnotes/releases).

### From Source

```bash
# Clone the repository
git clone https://github.com/rellit/mdnotes.git
cd mdnotes

# Build with cargo
cargo build --release

# The binary will be available at target/release/mdnotes
```

## Usage

```bash
mdnotes [OPTIONS] [COMMAND]
```

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

### Code Style

This project follows Rust standard formatting guidelines. Format your code with:

```bash
cargo fmt
```

Run clippy for linting:

```bash
cargo clippy
```

## Cross-Platform Support

mdnotes is designed to work on:
- Windows (x86_64)
- Linux (x86_64)
- macOS (x86_64 and ARM64)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

See [TODO.md](TODO.md) for the current implementation roadmap.

