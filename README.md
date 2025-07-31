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

### Using Command Line Arguments

```bash
# Run the TUI
cargo run -- <username> --token <your-api-token>

# Or use the compiled binary
./target/release/sc-tui <username> --token <your-api-token>
```

### Using Configuration File (Recommended)

You can either create a `config.toml` file manually based on `config.toml.example`, or let the tool create it interactively:

#### Default Workspace

If you have only one workspace configured, it will be used automatically:

```bash
# With single workspace, no need to specify --workspace
cargo run
```

For multiple workspaces, you can set a default in the config file:

```toml
default_workspace = "personal"
```

Then run without arguments:

```bash
# Uses the default workspace
cargo run
```

#### Interactive Setup (Easy Way)

Simply run with a workspace name and the tool will guide you through setup:

```bash
# First time setup - will prompt to create config
cargo run -- --workspace personal

# Short form
cargo run -- -w work
```

The tool will:
1. Ask if you want to create the configuration
2. Let you choose where to save it (default: `~/.config/sc-tui/config.toml`)
3. Prompt for your Shortcut API key
4. Prompt for your Shortcut mention name
5. Set this as the default workspace if it's the first one

#### Manual Setup

Create a `config.toml` file:

```toml
workspaces = ["personal", "work"]

# Optional: specify default workspace
default_workspace = "personal"

[personal]
api_key = "your-personal-api-key"
user_id = "your.mention.name"

[work]
api_key = "your-work-api-key"
user_id = "your.work.mention.name"
```

### TUI Navigation

- **↑/k** - Move up in the story list
- **↓/j** - Move down in the story list
- **←/h** - Move to previous column (workflow state)
- **→/l** - Move to next column (workflow state)
- **Enter** - View story details
- **Space** - Move story to a different workflow state
- **Esc** - Close detail view or cancel state selection
- **q** - Quit the application

#### Moving Stories Between States

When you press **Space** on a selected story:
1. A state selector dialog appears showing available workflow states
2. Use **↑/k** or **↓/j** to navigate through the states
3. Press **Enter** to confirm and move the story
4. Press **Esc** to cancel without making changes

The story will be immediately updated in Shortcut and moved to the appropriate column in the UI.

### Examples

```bash
# Basic usage with command line args
cargo run -- john.doe --token YOUR_API_TOKEN

# Using workspace from config
cargo run -- --workspace personal
cargo run -- -w work

# With options
cargo run -- john.doe --token YOUR_API_TOKEN --limit 20 --story-type feature

# Custom search (overrides default filters)
cargo run -- --workspace work --search "state:done updated:\"last week\""

# Enable debug output for troubleshooting
cargo run -- -w personal --debug
```

### Command-line Options

- `username` - The Shortcut mention name to search for (required if not using --workspace)
- `--token` - Your Shortcut API token (required if not using --workspace)
- `--workspace` / `-w` - Workspace name from config file (alternative to username/token)
- `--limit` (optional) - Maximum number of stories to display (default: 25)
- `--story-type` (optional) - Filter by story type: feature, bug, chore
- `--search` (optional) - Custom search query using Shortcut's search syntax
- `--debug` (optional) - Enable debug output for troubleshooting

### Search Syntax

The application supports Shortcut's search syntax. By default, it searches for stories where the user is either the owner OR the requester. You can use custom queries with the `--search` option:

- `owner:<username>` - Stories owned by the user only
- `requester:<username>` - Stories requested by the user only
- `(owner:<username> OR requester:<username>)` - Stories owned or requested by the user (default)
- `state:started` - Stories in started state
- `state:"in progress"` - Stories in progress (use quotes for multi-word states)
- `updated:"last week"` - Recently updated stories
- `updated:2024-01-01..2024-12-31` - Stories updated in a date range
- Combine queries: `owner:john state:started type:bug`

### Display

The TUI shows:

- Stories organized in columns by workflow state (To Do, In Progress, Done, etc.)
- All workflow states are displayed, even if they contain no stories
- Story count for each column in the header
- Story list with ID and name
- Detail popup with:
  - Story ID
  - Name
  - Type
  - Workflow State
  - Description
  - Shortcut URL
- State selector dialog for moving stories between workflow states

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

