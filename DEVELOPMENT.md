# Development Guide

This document contains development-specific information for contributors and maintainers of the sc-cli project.

## Prerequisites

- **Rust** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For version control and branch management
- **Shortcut API Token** - For testing (get yours from: <https://app.shortcut.com/settings/account/api-tokens>)

## Project Structure

```
sc-cli/
├── .gitignore           # Git ignore patterns
├── .tool-versions       # asdf version management
├── CLAUDE.md            # AI assistant guidance
├── Cargo.toml           # Rust project manifest
├── Cargo.lock           # Rust dependency lock file
├── README.md            # User documentation
├── DEVELOPMENT.md       # This file
├── src/                 # Rust source code
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root
│   ├── config.rs        # Configuration management
│   ├── git.rs           # Git integration functionality
│   ├── story_creator.rs # Story creation logic
│   ├── story_editor.rs  # Story editing functionality
│   ├── api/             # Shortcut API client
│   │   ├── mod.rs       # API types and traits
│   │   └── client.rs    # API client implementation
│   └── ui/              # TUI components
│       └── mod.rs       # UI implementation and tests
├── tests/               # Integration tests
│   ├── cli_test.rs      # CLI argument tests
│   └── integration_test.rs # API integration tests
└── target/              # Rust build artifacts (git ignored)
```

## Architecture Overview

### Core Components

- **Main Application** (`src/main.rs`) - Entry point and command handling
- **API Layer** (`src/api/`) - Shortcut API client and data structures
- **UI Layer** (`src/ui/`) - Terminal UI implementation using ratatui
- **Configuration** (`src/config.rs`) - Multi-workspace configuration management
- **Git Integration** (`src/git.rs`) - Git repository detection and branch creation
- **Story Management** - Creation (`src/story_creator.rs`) and editing (`src/story_editor.rs`)

### Key Features

- **Multi-workspace Support** - Manage multiple Shortcut workspaces
- **Interactive TUI** - Column-based view with keyboard navigation
- **Story Management** - Create, edit, move, and assign stories
- **Git Integration** - Create branches for stories with editable names
- **Pagination** - Load stories incrementally for performance

## Building and Running

### Development Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run with cargo (for development)
cargo run -- <args>

# Run with debug output
cargo run -- --debug <args>

# Install locally for testing
cargo install --path .
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test config::tests

# Run with debug output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test
```

### Code Quality

```bash
# Check code (fast)
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt

# Check format without changes
cargo fmt -- --check
```

## Development Workflow

### Setting Up Development Environment

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd sc-cli
   ```

2. **Create test configuration**
   ```bash
   # Create config for testing
   mkdir -p ~/.config/sc-cli
   cp config.toml.example ~/.config/sc-cli/config.toml
   # Edit with your API tokens
   ```

3. **Run tests to verify setup**
   ```bash
   cargo test
   ```

### Development Best Practices

1. **Code Style**
   - Follow Rust naming conventions
   - Use `cargo fmt` to format code
   - Run `cargo clippy` and fix warnings
   - Write tests for new functionality

2. **Testing**
   - Write unit tests alongside source code
   - Add integration tests for CLI commands
   - Use mock APIs for testing (see `src/story_creator/tests.rs`)
   - Test both success and error scenarios

3. **Error Handling**
   - Use `anyhow` for error propagation
   - Use `thiserror` for custom error types
   - Provide helpful error messages to users
   - Include debug information when `--debug` is used

4. **Documentation**
   - Update `CLAUDE.md` for AI assistant guidance
   - Update `README.md` for user-facing changes
   - Update this file for development changes
   - Add inline documentation for complex code

### Adding New Features

1. **Plan the feature**
   - Consider CLI interface design
   - Think about TUI integration
   - Plan error handling
   - Consider configuration needs

2. **Implement with tests**
   - Write failing tests first (TDD)
   - Implement the feature
   - Ensure tests pass
   - Add integration tests if needed

3. **Update documentation**
   - Add to appropriate sections in README.md
   - Update help text and command descriptions
   - Update CLAUDE.md if relevant

### Testing with Real API

```bash
# Test with your Shortcut workspace
cargo run -- --workspace your-workspace --debug

# Test specific commands
cargo run -- add "Test story" --workspace your-workspace
cargo run -- finish 123 --workspace your-workspace
cargo run -- edit 123 --workspace your-workspace
```

## API Integration

### Shortcut API

- Base URL: `https://api.app.shortcut.com/api/v3`
- Authentication: Bearer token via API key
- Rate limits: Respect Shortcut's API limits
- Error handling: Handle 401, 403, 404, 422 responses appropriately


## TUI Development

### Key Libraries

- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal handling
- **tokio** - Async runtime (if needed for future features)

### UI Components

- **App** - Main application state
- **Popups** - Story creation, editing, git branch creation
- **Navigation** - Column and list view modes
- **State Management** - Stories organized by workflow state

### Adding New Popups

1. Create state struct for popup data
2. Add popup state to main App struct
3. Add key handling in `handle_key_event`
4. Add rendering in `draw_*_popup` function
5. Add popup to main draw function

## Git Integration

### Features

- **Repository Detection** - Normal vs bare repositories
- **Branch Creation** - Interactive branch naming
- **Worktree Support** - For bare repositories
- **Branch Name Editing** - Users can customize suggested names

### Adding Git Features

1. Add functions to `src/git.rs`
2. Add UI components for user interaction
3. Add error handling for git command failures
4. Test with different repository types

## Release Process

1. **Prepare Release**
   ```bash
   # Update version in Cargo.toml
   # Update CHANGELOG (if exists)
   # Run full test suite
   cargo test
   cargo clippy
   ```

2. **Create Release Build**
   ```bash
   cargo build --release
   ```

3. **Test Release Build**
   ```bash
   ./target/release/sc-cli --version
   ./target/release/sc-cli --help
   ```

4. **Tag and Release**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

## Contributing

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow the development workflow above
   - Write tests for new functionality
   - Update documentation as needed

4. **Test thoroughly**
   ```bash
   cargo test
   cargo clippy
   cargo build --release
   ```

5. **Submit a pull request**
   - Include description of changes
   - Reference any related issues
   - Ensure CI passes

## Debugging

### Common Issues

1. **API Authentication**
   ```bash
   # Test with debug output
   cargo run -- --debug --workspace your-workspace
   ```

2. **Configuration Problems**
   ```bash
   # Check config file location
   ls -la ~/.config/sc-cli/config.toml
   ```

3. **Git Integration**
   ```bash
   # Test in git repository
   cd /path/to/git/repo
   cargo run -- --workspace your-workspace
   # Press 'g' on a story to test git integration
   ```

### Debug Output

Use `--debug` flag to see:
- API requests and responses
- Configuration loading details
- Git command execution
- Internal state changes

### Logging

The application uses `eprintln!` for debug output and `println!` for user output. This allows piping user output while seeing debug information.

## Performance Considerations

- **Story Loading** - Use pagination to avoid loading too many stories
- **API Calls** - Cache member information to avoid repeated lookups
- **UI Rendering** - Efficient list rendering with scrolling
- **Memory Usage** - Consider story count limits for large workspaces

## Security

- **API Tokens** - Store securely in config files (not in environment)
- **File Permissions** - Config files should be user-readable only
- **Error Messages** - Don't expose API tokens in error output
- **Input Validation** - Sanitize user input for API calls