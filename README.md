# Shortcut Story TUI

An interactive terminal UI for fetching and displaying stories from Shortcut (formerly Clubhouse) via their API.

## Prerequisites

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **Shortcut API Token** - Get yours from: <https://app.shortcut.com/settings/account/api-tokens>

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd sc-cli

# Build the project
cargo build --release

# The binary will be available at ./target/release/sc-tui
```

## Usage

```bash
# Run the TUI
cargo run -- <username> --token <your-api-token>

# Or use the compiled binary
./target/release/sc-tui <username> --token <your-api-token>
```

### TUI Navigation

- **↑/k** - Move up in the story list
- **↓/j** - Move down in the story list
- **Enter** - View story details
- **Esc** - Close detail view
- **q** - Quit the application

### Examples

```bash
# Basic usage
cargo run -- john.doe --token YOUR_API_TOKEN

# With options
cargo run -- john.doe --token YOUR_API_TOKEN --limit 20 --story-type feature

# Custom search (overrides default filters)
cargo run -- john.doe --token YOUR_API_TOKEN --search "state:done updated:\"last week\""

# Enable debug output for troubleshooting
cargo run -- john.doe --token YOUR_API_TOKEN --debug
```

### Command-line Options

- `username` (required) - The Shortcut username to search for
- `--token` (required) - Your Shortcut API token
- `--limit` (optional) - Maximum number of stories to display (default: 25)
- `--story-type` (optional) - Filter by story type: feature, bug, chore
- `--search` (optional) - Custom search query using Shortcut's search syntax
- `--debug` (optional) - Enable debug output for troubleshooting

### Search Syntax

The application supports Shortcut's search syntax. By default, it searches for `owner:<username>`. You can use custom queries with the `--search` option:

- `owner:<username>` - Stories owned by the user (default)
- `requester:<username>` - Stories requested by the user
- `state:started` - Stories in started state
- `state:"in progress"` - Stories in progress (use quotes for multi-word states)
- `updated:"last week"` - Recently updated stories
- `updated:2024-01-01..2024-12-31` - Stories updated in a date range
- Combine queries: `owner:john state:started type:bug`

### Display

The TUI shows:

- Story list with ID and name
- Detail popup with:
  - Story ID
  - Name
  - Type
  - Workflow State
  - Description
  - Shortcut URL

### Error Handling

The application handles:

- API authentication errors
- Network connection issues
- Invalid responses
- Empty search results

Make sure your API token has the necessary permissions to read stories and workflows.

## Development

### Project Structure

```
sc-cli/
├── .gitignore           # Git ignore patterns
├── .tool-versions       # asdf version management
├── CLAUDE.md            # AI assistant guidance
├── Cargo.toml           # Rust project manifest
├── Cargo.lock           # Rust dependency lock file
├── README.md            # This file
├── src/                 # Rust source code
│   ├── main.rs          # Application entry point
│   ├── api/             # Shortcut API client
│   │   ├── mod.rs       # API types and traits
│   │   └── client.rs    # API client implementation
│   └── ui/              # TUI components
│       └── mod.rs       # UI implementation
└── target/              # Rust build artifacts (git ignored)
```

### Building and Running

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run with cargo
cargo run -- <username> --token <token>

# Run tests
cargo test

# Check code
cargo check
cargo clippy
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo clippy` and `cargo test`
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

