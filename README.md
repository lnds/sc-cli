# Shortcut CLI & TUI

A command-line interface and interactive terminal UI for managing Shortcut stories via their API.

## Prerequisites

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **Shortcut API Token** - Get yours from: <https://app.shortcut.com/settings/account/api-tokens>

## Installation

### From Source

```bash
# Clone the repository
git clone [<repository-url>](https://github.com/lnds/sc-cli.git)
cd sc-cli

# Build the project
cargo build --release

# The binary will be available at ./target/release/sc-cli
```

### Using Cargo Install

```bash
# Install directly from the repository (requires Git)
cargo install --git <repository-url>

# Or install from a local clone
git clone <repository-url>
cd sc-cli
cargo install --path .
```

This will install the `sc-cli` binary to your Cargo bin directory (usually `~/.cargo/bin`), making it available from anywhere on your system.

## Usage

### Using Command Line Arguments

```bash
# Run the TUI
sc-cli <username> --token <your-api-token>
```

### Using Configuration File (Recommended)

You can either create a `config.toml` file manually based on `config.toml.example`, or let the tool create it interactively:

#### Default Workspace

If you have only one workspace configured, it will be used automatically:

```bash
# With single workspace, no need to specify --workspace
sc-cli
```

For multiple workspaces, you can set a default in the config file:

```toml
default_workspace = "personal"
```

Then run without arguments:

```bash
# Uses the default workspace
sc-cli
```

#### Interactive Setup (Easy Way)

Simply run with a workspace name and the tool will guide you through setup:

```bash
# First time setup - will prompt to create config
sc-cli --workspace personal

# Short form
sc-cli -w work
```

The tool will:
1. Ask if you want to create the configuration
2. Let you choose where to save it (default: `~/.config/sc-cli/config.toml`)
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

- **↑/k** - Move up in the story list (or scroll up in detail view)
- **↓/j** - Move down in the story list (or scroll down in detail view)
- **←/h** - Move to previous column (workflow state)
- **→/l** - Move to next column (workflow state)
- **Enter** - View story details
- **Space** - Move story to a different workflow state
- **o** - Take ownership of the selected story
- **a** - Add a new story
- **e** - Edit the selected story
- **g** - Create git branch for the selected story (in git repositories)
- **v** - Toggle between column and list view modes
- **n** - Load more stories (fetch next page)
- **Esc** - Close detail view or cancel state selection
- **q** - Quit the application

#### Moving Stories Between States

When you press **Space** on a selected story:
1. A state selector dialog appears showing available workflow states
2. Use **↑/k** or **↓/j** to navigate through the states
3. Press **Enter** to confirm and move the story
4. Press **Esc** to cancel without making changes

The story will be immediately updated in Shortcut and moved to the appropriate column in the UI.

#### Taking Ownership of Stories

When you press **o** on a selected story:
- The story ownership will be immediately updated to assign you as the owner
- This uses the API token's associated user account
- The story display will refresh to show the updated ownership
- This is useful for quickly claiming unassigned stories or taking over stories from other team members

#### Creating Stories in TUI

When you press **a** in the TUI:
- A popup form appears for creating a new story
- Navigate between fields using **Tab**
- Fill in the following fields:
  - **Name**: The story title (required)
  - **Description**: Detailed story description
  - **Type**: Use **↑/↓** to select between feature, bug, or chore
- Press **Enter** on the Type field to submit the story
- Press **Esc** at any time to cancel
- The story is created with you as the requester
- The new story appears in the first workflow state (typically "Backlog" or "To Do")

#### Editing Stories in TUI

When you press **e** on a selected story:
- A popup form appears for editing the story
- Navigate between fields using **Tab**
- Edit the following fields:
  - **Name**: The story title
  - **Description**: Detailed story description
  - **Type**: Use **↑/↓** to select between feature, bug, or chore
- Press **Enter** on the Type field to save changes
- Press **Esc** at any time to cancel without saving
- Changes are immediately updated in Shortcut and reflected in the UI

#### Loading More Stories (Pagination)

When you press **n** in the TUI:
- The application fetches the next page of stories from Shortcut
- New stories are seamlessly merged into the existing workflow columns
- The status bar shows the current count of loaded stories
- The loading state is indicated in the footer while fetching
- This allows you to load additional stories without restarting the application
- The feature remembers your search query and continues from where it left off

### Examples

#### Viewing Stories

```bash
# Basic usage with command line args
sc-cli view john.doe --token YOUR_API_TOKEN

# Using workspace from config
sc-cli view --workspace personal
sc-cli view -w work

# With options
sc-cli view john.doe --token YOUR_API_TOKEN --limit 20 --story-type feature

# Show stories where you are the requester
sc-cli view --workspace work --requester

# Show all stories (no user filter)
sc-cli view --workspace work --all

# Custom search (overrides default filters)
sc-cli view --workspace work --search "state:done updated:\"last week\""

# Enable debug output for troubleshooting
sc-cli view -w personal --debug

# Default behavior (view command is optional)
sc-cli --workspace personal
```

#### Adding Stories

```bash
# Interactive mode - prompts for all values
sc-cli add --workspace work

# Provide story name on command line (with quotes)
sc-cli add "Fix login bug" --workspace work

# Provide story name without quotes (multiple words)
sc-cli add --type bug this is a bug fix --workspace work

# Provide both name and type
sc-cli add --type bug Fix login bug --workspace work

# Using direct token instead of workspace
sc-cli add --token YOUR_API_TOKEN

# All values provided - only prompts for description
sc-cli add "Add user profile feature" --type feature -w work
```

#### Editing Stories

```bash
# Edit a story by ID
sc-cli edit 42 --workspace work

# Edit with prefixed story ID
sc-cli edit sc-42 -w work

# Using direct token instead of workspace
sc-cli edit 42 --token YOUR_API_TOKEN
```

#### Finishing Stories

```bash
# Mark a story as finished (move to Done state)
sc-cli finish 42 --workspace work

# With prefixed story ID
sc-cli finish sc-42 -w work

# Using direct token
sc-cli finish 42 --token YOUR_API_TOKEN
```


### Command-line Options

#### Global Options
- `--workspace` / `-w` - Workspace name from config file
- `--debug` / `-d` - Enable debug output

#### View Command (default)
- `username` - The Shortcut mention name to search for (optional if using --workspace)
- `--token` / `-t` - Your Shortcut API token (optional if using --workspace)
- `--limit` (optional) - Maximum number of stories to display (default: 50)
- `--story-type` (optional) - Filter by story type: feature, bug, chore
- `--search` (optional) - Custom search query using Shortcut's search syntax
- `--all` (optional) - Show all stories (no owner/requester filter)
- `--owner` (optional) - Show stories where user is the owner (default behavior)
- `--requester` (optional) - Show stories where user is the requester

Note: The `--all`, `--owner`, and `--requester` flags are mutually exclusive. Only one can be used at a time.

#### Add Command
- `name` (optional) - Story name as positional arguments. Can be provided as:
  - A quoted string: `"Fix login bug"`
  - Multiple words: `Fix login bug` (all remaining arguments become the name)
  - Will prompt if not provided
- `--token` / `-t` - Your Shortcut API token (optional if using --workspace)
- `--type` (optional) - Story type: feature, bug, or chore (will prompt if not provided)

#### Edit Command
- `story_id` - Story ID to edit (e.g., 42 or sc-42)
- `--token` / `-t` - Your Shortcut API token (optional if using --workspace)

#### Finish Command
- `story_id` - Story ID to mark as finished (e.g., 42 or sc-42)
- `--token` / `-t` - Your Shortcut API token (optional if using --workspace)

#### Show Command
- Same options as View command but displays stories in paginated terminal output instead of TUI

### Search Syntax

The application supports Shortcut's search syntax. By default, it searches for stories where the user is the owner. You can use custom queries with the `--search` option:

- `owner:<username>` - Stories owned by the user (default behavior)
- `requester:<username>` - Stories requested by the user
- `state:started` - Stories in started state
- `state:"in progress"` - Stories in progress (use quotes for multi-word states)
- `updated:"last week"` - Recently updated stories
- `updated:2024-01-01..2024-12-31` - Stories updated in a date range
- Combine queries: `owner:john state:started type:bug`

### Display

The TUI shows:

- Stories organized in columns by workflow state (To Do, In Progress, Done, etc.)
- All workflow states are displayed, even if they contain no stories
- Automatically selects the first story in the leftmost column that contains stories
- Story count for each column in the header
- Story list with ID and name
- Stories owned by you are displayed in cyan color for easy identification
- Detail popup with:
  - Story ID
  - Name
  - Type
  - Workflow State
  - Owners (shows owner names or "Unassigned")
  - Description
  - Comments (with author names and timestamps)
  - Shortcut URL
  - Scrollable content when there are many comments or long descriptions
  - Scroll indicator showing current position
- State selector dialog for moving stories between workflow states

### Error Handling

The application handles:

- API authentication errors
- Network connection issues
- Invalid responses
- Empty search results

Make sure your API token has the necessary permissions to read stories and workflows.

## Additional Commands

The application also includes additional commands for enhanced workflow:

```bash
# Mark a story as finished
sc-cli finish 42  # or sc-42

# Edit an existing story
sc-cli edit 42

# Show stories in paginated terminal output
sc-cli show --limit 10
```

## Git Integration

When working in a git repository, the TUI provides additional functionality:

- **Press 'g' on any story** to create a git branch
- **Branch names** are suggested using Shortcut's formatted VCS branch names
- **Edit branch names** before creation using Tab or 'e' key
- **Supports both normal and bare repositories** (uses git worktree for bare repos)
- **Automatic branch naming** follows Shortcut conventions

## Development

For development information, build instructions, contributing guidelines, and technical details, see [DEVELOPMENT.md](DEVELOPMENT.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

