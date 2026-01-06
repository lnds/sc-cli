mod api;
mod config;
mod git;
mod story_creator;
mod story_editor;
mod ui;

use anyhow::{Context, Result};
use api::{ShortcutApi, client::ShortcutClient};
use clap::Parser;
use config::Config;
use dialoguer::Input;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{collections::HashMap, io};
use story_creator::StoryCreator;
use story_editor::StoryEditor;
use ui::App;

fn validate_story_type(s: &str) -> Result<String, String> {
    match s {
        "feature" | "bug" | "chore" => Ok(s.to_string()),
        _ => Err(format!(
            "Invalid story type '{s}'. Must be one of: feature, bug, chore"
        )),
    }
}

#[derive(Debug)]
struct ViewCommandArgs {
    workspace: Option<String>,
    username: Option<String>,
    token: Option<String>,
    limit: Option<usize>,
    story_type: Option<String>,
    search: Option<String>,
    all: bool,
    _owner: bool,
    requester: bool,
    debug: bool,
}

#[derive(Debug)]
struct ShowCommandArgs {
    workspace: Option<String>,
    username: Option<String>,
    token: Option<String>,
    limit: usize,
    story_type: Option<String>,
    search: Option<String>,
    all: bool,
    _owner: bool,
    requester: bool,
    debug: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI and TUI client for Shortcut stories", long_about = None)]
struct Args {
    /// Workspace name from config file (optional if default workspace is set)
    #[arg(short, long, global = true)]
    workspace: Option<String>,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,

    /// Show all stories (no owner/requester filter)
    #[arg(long, global = true, conflicts_with_all = ["owner", "requester"])]
    all: bool,

    /// Show stories where user is the owner (default when no filter specified)
    #[arg(long, global = true, conflicts_with_all = ["all", "requester"])]
    owner: bool,

    /// Show stories where user is the requester
    #[arg(long, global = true, conflicts_with_all = ["all", "owner"])]
    requester: bool,

    /// Maximum number of stories to display (overrides workspace config)
    #[arg(short, long, global = true)]
    limit: Option<usize>,

    /// Filter by story type (feature, bug, chore)
    #[arg(long, global = true)]
    story_type: Option<String>,

    /// Custom search query using Shortcut's search syntax
    #[arg(short, long, global = true)]
    search: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Add a new story to the backlog
    Add {
        /// Story name words (optional, will prompt if not provided)
        #[arg(trailing_var_arg = true)]
        name: Vec<String>,

        /// Shortcut API token (optional if using workspace)
        #[arg(short, long)]
        token: Option<String>,

        /// Story type (feature, bug, chore)
        #[arg(long, value_parser = validate_story_type)]
        r#type: Option<String>,
    },
    /// Mark a story as finished (Done state)
    Finish {
        /// Story ID to mark as finished (e.g., 42 or sc-42)
        story_id: String,

        /// Shortcut API token (optional if using workspace)
        #[arg(short, long)]
        token: Option<String>,
    },
    /// View stories in TUI mode (default command)
    View {
        /// Shortcut mention name to search for (optional if using workspace)
        username: Option<String>,

        /// Shortcut API token (optional if using workspace)
        #[arg(short, long)]
        token: Option<String>,

        /// Maximum number of stories to display (overrides workspace config)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Filter by story type (feature, bug, chore)
        #[arg(long)]
        story_type: Option<String>,

        /// Custom search query using Shortcut's search syntax
        #[arg(short, long)]
        search: Option<String>,

        /// Show all stories (no owner/requester filter)
        #[arg(long, conflicts_with_all = ["owner", "requester"])]
        all: bool,

        /// Show stories where user is the owner (default)
        #[arg(long, conflicts_with_all = ["all", "requester"])]
        owner: bool,

        /// Show stories where user is the requester
        #[arg(long, conflicts_with_all = ["all", "owner"])]
        requester: bool,
    },
    /// Show stories in terminal with pagination (like more command)
    Show {
        /// Shortcut mention name to search for (optional if using workspace)
        username: Option<String>,

        /// Shortcut API token (optional if using workspace)
        #[arg(short, long)]
        token: Option<String>,

        /// Number of stories to show per page (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Filter by story type (feature, bug, chore)
        #[arg(long)]
        story_type: Option<String>,

        /// Custom search query using Shortcut's search syntax
        #[arg(short, long)]
        search: Option<String>,

        /// Show all stories (no owner/requester filter)
        #[arg(long, conflicts_with_all = ["owner", "requester"])]
        all: bool,

        /// Show stories where user is the owner (default)
        #[arg(long, conflicts_with_all = ["all", "requester"])]
        owner: bool,

        /// Show stories where user is the requester
        #[arg(long, conflicts_with_all = ["all", "owner"])]
        requester: bool,
    },
    /// Edit an existing story
    Edit {
        /// Story ID to edit (e.g., 42 or sc-42)
        story_id: String,

        /// Shortcut API token (optional if using workspace)
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Add a comment to a story
    Comment {
        /// Story ID to comment on (e.g., 42 or sc-42)
        story_id: String,

        /// Comment message (will prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,

        /// Shortcut API token (optional if using workspace)
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Create a git branch for a story
    Branch {
        /// Story ID to create branch for (e.g., 42 or sc-42)
        story_id: String,

        /// Use the default branch name without prompting
        #[arg(long)]
        default: bool,

        /// Create a worktree instead of a branch (for bare repositories)
        #[arg(long)]
        worktree: bool,

        /// Shortcut API token (optional if using workspace)
        #[arg(long)]
        token: Option<String>,
    },
    /// Display the version of sc-cli
    Version,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Add {
            name,
            token,
            r#type,
        }) => handle_add_command(args.workspace, token, name, r#type, args.debug),
        Some(Command::Finish { story_id, token }) => {
            handle_finish_command(args.workspace, token, story_id, args.debug)
        }
        Some(Command::View {
            username,
            token,
            limit,
            story_type,
            search,
            all,
            owner,
            requester,
        }) => handle_view_command(ViewCommandArgs {
            workspace: args.workspace,
            username,
            token,
            limit: limit.or(args.limit),
            story_type: story_type.or(args.story_type),
            search: search.or(args.search),
            all: all || args.all,
            _owner: owner || args.owner,
            requester: requester || args.requester,
            debug: args.debug,
        }),
        Some(Command::Show {
            username,
            token,
            limit,
            story_type,
            search,
            all,
            owner,
            requester,
        }) => handle_show_command(ShowCommandArgs {
            workspace: args.workspace,
            username,
            token,
            limit,
            story_type: story_type.or(args.story_type),
            search: search.or(args.search),
            all: all || args.all,
            _owner: owner || args.owner,
            requester: requester || args.requester,
            debug: args.debug,
        }),
        Some(Command::Edit { story_id, token }) => {
            handle_edit_command(args.workspace, token, story_id, args.debug)
        }
        Some(Command::Comment {
            story_id,
            message,
            token,
        }) => handle_comment_command(args.workspace, token, story_id, message, args.debug),
        Some(Command::Branch {
            story_id,
            default,
            worktree,
            token,
        }) => handle_branch_command(args.workspace, token, story_id, default, worktree, args.debug),
        Some(Command::Version) => handle_version_command(),
        None => {
            // Default to view command when no subcommand is specified
            handle_view_command(ViewCommandArgs {
                workspace: args.workspace,
                username: None,
                token: None,
                limit: args.limit,
                story_type: args.story_type,
                search: args.search,
                all: args.all,
                _owner: args.owner,
                requester: args.requester,
                debug: args.debug,
            })
        }
    }
}

fn handle_version_command() -> Result<()> {
    println!("sc-cli {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn handle_add_command(
    workspace: Option<String>,
    token: Option<String>,
    name: Vec<String>,
    story_type: Option<String>,
    debug: bool,
) -> Result<()> {
    // Get token and user info from args or config
    // Priority: 1. Explicit workspace, 2. Default workspace (if no token), 3. Token from CLI
    let (token, _username) = if let Some(workspace_name) = workspace {
        // Use explicitly specified workspace
        let (config, _created) =
            Config::load_or_create(&workspace_name).context("Failed to load or create config")?;
        let workspace = config
            .get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        (workspace.api_key.clone(), workspace.user_id.clone())
    } else if token.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let workspace =
                        config
                            .get_workspace(&default_workspace_name)
                            .context(format!(
                                "Failed to get default workspace '{default_workspace_name}'"
                            ))?;
                    (workspace.api_key.clone(), workspace.user_id.clone())
                } else {
                    anyhow::bail!(
                        "No default workspace configured. Use --workspace to specify one or provide --token"
                    );
                }
            }
            Err(_) => {
                anyhow::bail!(
                    "No configuration file found. Use --workspace to create one or provide --token"
                );
            }
        }
    } else {
        // Use command line arguments
        let token = token
            .ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?;
        // For add command, we don't need username from CLI, we'll get it from the API
        (token, String::new())
    };

    // Initialize API client
    let client = ShortcutClient::new(token, debug).context("Failed to create Shortcut client")?;

    // Get current member info to use as requester
    let current_member = client
        .get_current_member()
        .context("Failed to get current member info")?;

    if debug {
        eprintln!(
            "Current user: {} ({}) - ID: {}",
            current_member.name, current_member.mention_name, current_member.id
        );
    }

    // Get workflows to find the appropriate initial state
    let workflows = client
        .get_workflows()
        .context("Failed to fetch workflows")?;

    // Find the first workflow and get its first state (typically "Backlog" or "To Do")
    let workflow_state_id = workflows
        .first()
        .and_then(|w| w.states.first())
        .map(|s| s.id)
        .ok_or_else(|| anyhow::anyhow!("No workflows found in the workspace"))?;

    if debug {
        eprintln!("Using workflow state ID: {workflow_state_id}");
    }

    // Convert name vector to optional string
    let name_str = if name.is_empty() {
        None
    } else {
        Some(name.join(" "))
    };

    // Use StoryCreator to gather input and create the story
    let story_creator =
        StoryCreator::from_prompts(current_member.id, workflow_state_id, name_str, story_type)?;

    if debug {
        eprintln!("Creating story:");
        eprintln!("  Name: {}", story_creator.name);
        eprintln!("  Type: {}", story_creator.story_type);
        eprintln!(
            "  Description length: {} chars",
            story_creator.description.len()
        );
        eprintln!("  Requester ID: {}", story_creator.requested_by_id);
    }

    // Create the story
    let created_story = story_creator.create(&client)?;

    println!("\n✅ Story created successfully!");
    println!("  ID: #{}", created_story.id);
    println!("  Name: {}", created_story.name);
    println!("  URL: {}", created_story.app_url);

    Ok(())
}

fn handle_finish_command(
    workspace: Option<String>,
    token: Option<String>,
    story_id: String,
    debug: bool,
) -> Result<()> {
    // Parse story ID - accept both "42" and "sc-42" formats
    let story_id = if story_id.to_lowercase().starts_with("sc-") {
        story_id[3..]
            .parse::<i64>()
            .context("Invalid story ID format. Expected 'sc-N' where N is a number")?
    } else {
        story_id
            .parse::<i64>()
            .context("Invalid story ID format. Expected a number or 'sc-N' format")?
    };
    // Get token from args or config
    // Priority: 1. Explicit workspace, 2. Default workspace (if no token), 3. Token from CLI
    let token = if let Some(workspace_name) = workspace {
        // Use explicitly specified workspace
        let (config, _created) =
            Config::load_or_create(&workspace_name).context("Failed to load or create config")?;
        let workspace = config
            .get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        workspace.api_key.clone()
    } else if token.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let workspace =
                        config
                            .get_workspace(&default_workspace_name)
                            .context(format!(
                                "Failed to get default workspace '{default_workspace_name}'"
                            ))?;
                    workspace.api_key.clone()
                } else {
                    anyhow::bail!(
                        "No default workspace configured. Use --workspace to specify one or provide --token"
                    );
                }
            }
            Err(_) => {
                anyhow::bail!(
                    "No configuration file found. Use --workspace to create one or provide --token"
                );
            }
        }
    } else {
        // Use command line arguments
        token.ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?
    };

    // Initialize API client
    let client = ShortcutClient::new(token, debug).context("Failed to create Shortcut client")?;

    // Get current member info for debug/confirmation
    let current_member = client
        .get_current_member()
        .context("Failed to get current member info")?;

    if debug {
        eprintln!(
            "Current user: {} ({}) - ID: {}",
            current_member.name, current_member.mention_name, current_member.id
        );
        eprintln!("Marking story #{story_id} as finished...");
    }

    // Update story to Done state (workflow_state_id: 500000010)
    let done_state_id = 500000010;

    match client.update_story_state(story_id, done_state_id) {
        Ok(updated_story) => {
            println!("✅ Story successfully marked as finished!");
            println!("  ID: #{}", updated_story.id);
            println!("  Name: {}", updated_story.name);
            println!("  URL: {}", updated_story.app_url);

            if debug {
                eprintln!(
                    "Story moved to workflow state ID: {}",
                    updated_story.workflow_state_id
                );
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to mark story as finished: {e}");

            if debug {
                eprintln!("Error details: {e:?}");
            }

            // Check if it's a 404 error (story not found)
            if e.to_string().contains("404") {
                eprintln!("💡 Story #{story_id} was not found. Please check the story ID.");
            } else if e.to_string().contains("422") {
                eprintln!(
                    "💡 The story might already be in the Done state or there might be a workflow restriction."
                );
            }

            anyhow::bail!("Failed to finish story");
        }
    }

    Ok(())
}

fn handle_edit_command(
    workspace: Option<String>,
    token: Option<String>,
    story_id: String,
    debug: bool,
) -> Result<()> {
    // Parse story ID - accept both "42" and "sc-42" formats
    let story_id = if story_id.to_lowercase().starts_with("sc-") {
        story_id[3..]
            .parse::<i64>()
            .context("Invalid story ID format. Expected 'sc-N' where N is a number")?
    } else {
        story_id
            .parse::<i64>()
            .context("Invalid story ID format. Expected a number or 'sc-N' format")?
    };
    // Get token from args or config
    // Priority: 1. Explicit workspace, 2. Default workspace (if no token), 3. Token from CLI
    let token = if let Some(workspace_name) = workspace {
        // Use explicitly specified workspace
        let (config, _created) =
            Config::load_or_create(&workspace_name).context("Failed to load or create config")?;
        let workspace = config
            .get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        workspace.api_key.clone()
    } else if token.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let workspace =
                        config
                            .get_workspace(&default_workspace_name)
                            .context(format!(
                                "Failed to get default workspace '{default_workspace_name}'"
                            ))?;
                    workspace.api_key.clone()
                } else {
                    anyhow::bail!(
                        "No default workspace configured. Use --workspace to specify one or provide --token"
                    );
                }
            }
            Err(_) => {
                anyhow::bail!(
                    "No configuration file found. Use --workspace to create one or provide --token"
                );
            }
        }
    } else {
        // Use command line arguments
        token.ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?
    };

    // Initialize API client
    let client = ShortcutClient::new(token, debug).context("Failed to create Shortcut client")?;

    if debug {
        eprintln!("Fetching story #{story_id} for editing...");
    }

    // Fetch the story to edit
    let story = client
        .get_story(story_id)
        .context(format!("Failed to fetch story #{story_id}"))?;

    if debug {
        eprintln!("Found story: {} - {}", story.id, story.name);
        eprintln!("Current type: {}", story.story_type);
        eprintln!("Description length: {} chars", story.description.len());
    }

    // Create a story editor with the current story
    let mut story_editor = StoryEditor::from_story(&story);

    // Show current story details
    println!("\n📖 Current Story Details:");
    println!("  ID: #{}", story.id);
    println!("  Name: {}", story.name);
    println!("  Type: {}", story.story_type);
    if story.description.is_empty() {
        println!("  Description: (no description)");
    } else {
        println!("  Description:");
        for line in story.description.lines() {
            println!("    {line}");
        }
    }
    println!("  URL: {}", story.app_url);

    // Interactive editing
    let should_save = story_editor
        .edit_with_prompts()
        .context("Failed to edit story")?;

    if !should_save {
        println!("\n❌ Edit cancelled. No changes were made.");
        return Ok(());
    }

    if debug {
        eprintln!("Updating story:");
        eprintln!("  Name: {}", story_editor.name);
        eprintln!("  Type: {}", story_editor.story_type);
        eprintln!(
            "  Description length: {} chars",
            story_editor.description.len()
        );
    }

    // Update the story
    let updated_story = story_editor
        .update(&client)
        .context("Failed to update story")?;

    println!("\n✅ Story updated successfully!");
    println!("  ID: #{}", updated_story.id);
    println!("  Name: {}", updated_story.name);
    println!("  Type: {}", updated_story.story_type);
    println!("  URL: {}", updated_story.app_url);

    Ok(())
}

fn handle_comment_command(
    workspace: Option<String>,
    token: Option<String>,
    story_id: String,
    message: Option<String>,
    debug: bool,
) -> Result<()> {
    // Parse story ID (remove "sc-" prefix if present)
    let story_id: i64 = story_id
        .strip_prefix("sc-")
        .unwrap_or(&story_id)
        .parse()
        .context(format!("Invalid story ID: {story_id}"))?;

    // Get API token from command line or config
    let token = if let Some(t) = token {
        t
    } else if let Some(ws) = workspace {
        let (config, _created) =
            Config::load_or_create(&ws).context("Failed to load or create config")?;
        config
            .get_workspace(&ws)
            .with_context(|| format!("Workspace '{}' not found in config", ws))?
            .api_key
            .clone()
    } else {
        anyhow::bail!("No API token provided. Use --token or --workspace");
    };

    // Initialize API client
    let client = ShortcutClient::new(token, debug).context("Failed to create Shortcut client")?;

    if debug {
        eprintln!("Fetching story #{story_id} to add comment...");
    }

    // Fetch the story to verify it exists
    let story = client
        .get_story(story_id)
        .context(format!("Failed to fetch story #{story_id}"))?;

    println!("\n💬 Adding comment to story:");
    println!("  #{} - {}", story.id, story.name);

    // Get the comment text
    let comment_text = if let Some(msg) = message {
        msg
    } else {
        // Prompt for comment if not provided
        println!("\nEnter your comment (press Ctrl+D or Enter twice to submit):");

        let mut lines = Vec::new();
        let stdin = std::io::stdin();
        let mut empty_line_count = 0;

        loop {
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => break, // EOF (Ctrl+D)
                Ok(_) => {
                    if line == "\n" {
                        empty_line_count += 1;
                        if empty_line_count >= 2 {
                            break; // Two consecutive empty lines
                        }
                    } else {
                        empty_line_count = 0;
                    }
                    lines.push(line);
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to read input: {}", e)),
            }
        }

        lines.join("").trim().to_string()
    };

    if comment_text.is_empty() {
        anyhow::bail!("Comment cannot be empty");
    }

    if debug {
        eprintln!("Posting comment ({} chars)...", comment_text.len());
    }

    // Add the comment via API
    client
        .add_comment(story_id, &comment_text)
        .context("Failed to add comment")?;

    println!("\n✅ Comment added successfully!");
    println!("  View story: {}", story.app_url);

    Ok(())
}

fn handle_branch_command(
    workspace: Option<String>,
    token: Option<String>,
    story_id_str: String,
    use_default: bool,
    use_worktree: bool,
    debug: bool,
) -> Result<()> {
    // Get token from args or config
    let token = if let Some(workspace_name) = workspace {
        // Use explicitly specified workspace
        let (config, _created) =
            Config::load_or_create(&workspace_name).context("Failed to load or create config")?;
        let ws = config
            .get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        ws.api_key.clone()
    } else if token.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let ws = config
                        .get_workspace(&default_workspace_name)
                        .context(format!(
                            "Failed to get default workspace '{default_workspace_name}'"
                        ))?;
                    ws.api_key.clone()
                } else {
                    anyhow::bail!(
                        "No default workspace configured. Use --workspace to specify one or provide --token"
                    );
                }
            }
            Err(_) => {
                anyhow::bail!(
                    "No configuration file found. Use --workspace to create one or provide --token"
                );
            }
        }
    } else {
        // Use command line arguments
        token.ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?
    };

    let client = ShortcutClient::new(token, debug).context("Failed to create Shortcut client")?;

    // Parse story ID (handle both "42" and "sc-42" formats)
    let story_id: i64 = story_id_str
        .trim_start_matches("sc-")
        .trim_start_matches("SC-")
        .parse()
        .context(format!("Invalid story ID: {story_id_str}"))?;

    // Fetch the story to get the suggested branch name
    let story = client
        .get_story(story_id)
        .context(format!("Failed to fetch story {story_id}"))?;

    if debug {
        eprintln!("Fetched story: {} - {}", story.id, story.name);
    }

    // Generate the suggested branch name
    let suggested_branch = story.formatted_vcs_branch_name.clone().unwrap_or_else(|| {
        format!(
            "sc-{}-{}",
            story.id,
            story
                .name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .take(5)
                .collect::<Vec<_>>()
                .join("-")
        )
    });

    // Determine the branch name to use
    let branch_name = if use_default {
        println!("Using default branch name: {}", suggested_branch);
        suggested_branch
    } else {
        Input::new()
            .with_prompt("Branch name")
            .with_initial_text(&suggested_branch)
            .interact_text()
            .context("Failed to read branch name")?
    };

    // Detect git context
    let git_context = git::GitContext::detect().context("Failed to detect git context")?;

    if !git_context.is_git_repo() {
        anyhow::bail!("Not a git repository. Please run this command from within a git repository.");
    }

    // Determine operation type
    let should_use_worktree = use_worktree || git_context.is_bare_repo();

    if should_use_worktree && !use_worktree && git_context.is_bare_repo() {
        println!("Detected bare repository, using worktree mode.");
    }

    // Build the request
    let request = git::operations::GitBranchRequest {
        branch_name: branch_name.clone(),
        worktree_path: git::generate_worktree_path(&branch_name),
        operation: if should_use_worktree {
            git::operations::GitOperation::CreateWorktree
        } else {
            git::operations::GitOperation::CreateBranch
        },
        story_id,
    };

    // Execute the git operation
    let result = git::operations::execute_git_operation(&request);

    if result.success {
        println!("\n✅ {}", result.message);

        // Move story to In Progress
        let workflows = client
            .get_workflows()
            .context("Failed to fetch workflows")?;

        if let Some(_updated_story) =
            git::operations::move_story_to_in_progress(&client, story_id, &workflows, debug)
        {
            println!("📋 Story moved to In Progress");
        }

        if let Some(worktree_path) = result.worktree_path {
            println!("\n📁 Worktree created at: {}", worktree_path);
            println!("   Run: cd {}", worktree_path);
        }
    } else {
        println!("\n❌ {}", result.message);
        return Err(anyhow::anyhow!(result.message));
    }

    println!("  View story: {}", story.app_url);

    Ok(())
}

fn handle_view_command(args: ViewCommandArgs) -> Result<()> {
    // Get token, username, and fetch_limit from args or config
    let (token, username, config_limit) = if let Some(workspace_name) = args.workspace {
        // Use explicitly specified workspace
        let (config, _created) =
            Config::load_or_create(&workspace_name).context("Failed to load or create config")?;
        let workspace = config
            .get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        (
            workspace.api_key.clone(),
            workspace.user_id.clone(),
            workspace.fetch_limit,
        )
    } else if args.token.is_none() && args.username.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let workspace =
                        config
                            .get_workspace(&default_workspace_name)
                            .context(format!(
                                "Failed to get default workspace '{default_workspace_name}'"
                            ))?;
                    (
                        workspace.api_key.clone(),
                        workspace.user_id.clone(),
                        workspace.fetch_limit,
                    )
                } else {
                    anyhow::bail!(
                        "No default workspace configured. Use --workspace to specify one or provide --token and username"
                    );
                }
            }
            Err(_) => {
                anyhow::bail!(
                    "No configuration file found. Use --workspace to create one or provide --token and username"
                );
            }
        }
    } else {
        // Use command line arguments with default limit
        let token = args
            .token
            .ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?;
        let username = args
            .username
            .ok_or_else(|| anyhow::anyhow!("Either username or --workspace must be provided"))?;
        (token, username, 50) // Default limit when not using workspace
    };

    // Use command-line limit if provided, otherwise use workspace config limit
    let limit = args.limit.unwrap_or(config_limit);

    // Initialize API client
    let client =
        ShortcutClient::new(token, args.debug).context("Failed to create Shortcut client")?;

    // Get workflows
    if args.debug {
        eprintln!("Fetching workflows...");
    }
    let workflows = client
        .get_workflows()
        .context("Failed to fetch workflows")?;

    // Get epics for filtering
    if args.debug {
        eprintln!("Fetching epics...");
    }
    let epics = client.get_epics().context("Failed to fetch epics")?;
    if args.debug {
        eprintln!("Found {} epics", epics.len());
    }

    // Build search query
    let query = if let Some(search) = args.search {
        search
    } else {
        let mut query_parts = vec![];

        // Apply filter based on flags (default to owner if none specified)
        if args.all {
            // No user filter for --all flag
        } else if args.requester {
            query_parts.push(format!("requester:{username}"));
        } else {
            // Default to owner filter (also when --owner is explicitly used)
            query_parts.push(format!("owner:{username}"));
        }

        if let Some(story_type) = args.story_type {
            query_parts.push(format!("type:{story_type}"));
        }

        query_parts.push("is:story".to_string());
        query_parts.join(" ")
    };

    // Search for stories - use initial page loading
    if args.debug {
        eprintln!("Searching for stories...");
        eprintln!("Query: {query}");
    }

    // Load first page initially, but limit to the specified limit
    let mut stories = Vec::new();
    let mut next_page_token = None;
    let mut loaded_count = 0;

    // Keep loading pages until we reach the limit
    loop {
        let search_result = client
            .search_stories_page(&query, next_page_token)
            .context("Failed to search stories")?;

        // Add stories up to the limit, avoiding duplicates
        let remaining_slots = limit.saturating_sub(loaded_count);
        let mut added_count = 0;

        for story in search_result.stories {
            // Stop if we've reached the limit
            if added_count >= remaining_slots {
                break;
            }

            // Check for duplicates by ID
            if !stories
                .iter()
                .any(|existing: &api::Story| existing.id == story.id)
            {
                stories.push(story);
                added_count += 1;
            }
        }

        loaded_count += added_count;
        next_page_token = search_result.next_page_token;

        // Stop if we've reached the limit or there are no more pages
        if loaded_count >= limit || next_page_token.is_none() {
            break;
        }

        // Safety check: if we didn't add any new stories from this page,
        // but there are still more pages, we're likely in a duplicate loop
        if added_count == 0 && next_page_token.is_some() {
            if args.debug {
                eprintln!(
                    "No new stories added from current page, stopping to prevent infinite loop"
                );
            }
            break;
        }
    }

    if stories.is_empty() {
        eprintln!("No stories found for query: {query}");
        eprintln!("Try using a different search query or check if the username is correct.");
        return Ok(());
    }

    if args.debug {
        eprintln!("Found {} stories", stories.len());
        if next_page_token.is_some() {
            eprintln!("More stories available for pagination");
        }
    }

    // Fetch members to populate cache BEFORE setting up terminal
    let mut member_cache = HashMap::new();
    if args.debug {
        eprintln!("Fetching members for cache...");
    }
    match client.get_members() {
        Ok(members) => {
            if args.debug {
                eprintln!("Fetched {} members from API", members.len());
            }
            for member in members {
                if args.debug {
                    eprintln!(
                        "Caching member: id='{}', name='{}', mention_name='{}'",
                        member.id, member.profile.name, member.profile.mention_name
                    );
                }
                // Store name with mention_name in parentheses
                let display_name =
                    format!("{} ({})", member.profile.name, member.profile.mention_name);
                member_cache.insert(member.id, display_name);
            }
            if args.debug {
                eprintln!("Cached {} members", member_cache.len());
                // Also show some story owner IDs for comparison
                if !stories.is_empty() {
                    eprintln!("Sample story owner IDs:");
                    for story in stories.iter().take(3) {
                        if !story.owner_ids.is_empty() {
                            eprintln!("  Story {}: owner_ids={:?}", story.id, story.owner_ids);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("WARNING: Failed to fetch members for cache: {e}");
            if args.debug {
                eprintln!("Full error: {e:?}");
            }
            eprintln!("Owner names will be displayed as IDs");
        }
    }

    // Setup terminal AFTER fetching members
    setup_terminal()?;

    // Create app with stories and workflows
    let mut app = App::new(stories, workflows.clone(), query.clone(), next_page_token);

    // Set epics in the app for filtering
    app.set_epics(epics.clone());

    // Populate the member cache in the app
    for (id, name) in member_cache {
        app.add_member_to_cache(id, name);
    }

    // Try to get current user ID to highlight owned stories
    if args.debug {
        eprintln!("Fetching current user for story highlighting...");
    }
    match client.get_current_member() {
        Ok(member) => {
            if args.debug {
                eprintln!(
                    "Current user: {} ({}) - ID: {}",
                    member.name, member.mention_name, member.id
                );
            }
            app.set_current_user_id(member.id);
        }
        Err(e) => {
            if args.debug {
                eprintln!("Failed to get current user for highlighting: {e}");
                eprintln!("Owned stories will not be highlighted");
            }
        }
    }

    let result = run_app(app, client, workflows, args.debug);

    // Restore terminal
    restore_terminal()?;

    result
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_app(
    mut app: App,
    client: ShortcutClient,
    workflows: Vec<api::Workflow>,
    debug: bool,
) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) if key.kind == crossterm::event::KeyEventKind::Press => {
            // Special handling for Enter in state selector
            if app.show_state_selector && key.code == crossterm::event::KeyCode::Enter {
                let story_update = app
                    .get_selected_story()
                    .map(|story| (story.id, app.get_selected_target_state()));

                if let Some((story_id, Some(target_state_id))) = story_update {
                    // Update story state via API
                    match client.update_story_state(story_id, target_state_id) {
                        Ok(updated_story) => {
                            // Update the story in our local data
                            update_story_state(&mut app, story_id, updated_story);
                        }
                        Err(e) => {
                            eprintln!("Failed to update story state: {e}");
                        }
                    }
                }
                app.show_state_selector = false;
                app.state_selector_index = 0;
            } else {
                // Handle all other events normally
                app.handle_key_event(key)?;
            }
                }
                crossterm::event::Event::Mouse(mouse) => {
                    app.handle_mouse_event(mouse)?;
                }
                _ => {}
            }
        }

        // Check if we need to handle ownership change
        if app.take_ownership_requested {
            let story_id = app.get_selected_story().map(|s| s.id);

            if let Some(story_id) = story_id {
                // Get current member info
                match client.get_current_member() {
                    Ok(member) => {
                        // Add member to cache if not already present
                        let display_name = format!("{} ({})", member.name, member.mention_name);
                        app.add_member_to_cache(member.id.clone(), display_name);

                        // Update story ownership
                        match client.update_story(story_id, vec![member.id.clone()]) {
                            Ok(updated_story) => {
                                // Update the story in our local data
                                update_story_ownership(&mut app, story_id, updated_story);
                            }
                            Err(e) => {
                                eprintln!("Failed to update story ownership: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get current member: {e}");
                    }
                }
            }
            app.take_ownership_requested = false;
        }

        // Check if we need to create a new story
        if app.create_story_requested
            && !app
                .create_popup_state
                .name_textarea
                .lines()
                .join("")
                .trim()
                .is_empty()
        {
            // Get current member info to use as requester
            match client.get_current_member() {
                Ok(current_member) => {
                    // Find the first workflow state
                    let workflow_state_id = workflows
                        .first()
                        .and_then(|w| w.states.first())
                        .map(|s| s.id)
                        .unwrap_or(500000007); // Default to "To Do" if not found

                    // Create the story using the popup data including epic
                    match client.create_story(
                        app.create_popup_state.name_textarea.lines().join(""),
                        app.create_popup_state.description_textarea.lines().join(""),
                        app.create_popup_state.story_type.clone(),
                        current_member.id,
                        workflow_state_id,
                        app.create_popup_state.epic_id,
                    ) {
                        Ok(new_story) => {
                            // Add the new story to the unfiltered list
                            app.all_stories_unfiltered.push(new_story.clone());

                            // If there's an epic filter active, check if the new story matches
                            if let Some(epic_id) = app.selected_epic_filter {
                                if new_story.epic_id == Some(epic_id) {
                                    // Story matches filter, add it
                                    app.stories_by_state
                                        .entry(new_story.workflow_state_id)
                                        .or_default()
                                        .push(new_story.clone());

                                    // Sort stories by position
                                    if let Some(stories) =
                                        app.stories_by_state.get_mut(&workflow_state_id)
                                    {
                                        stories.sort_by_key(|s| s.position);
                                    }

                                    // Update list view
                                    app.all_stories_list.push(new_story);
                                    app.all_stories_list.sort_by_key(|s| s.position);
                                }
                                // If story doesn't match filter, it won't be visible
                            } else {
                                // No filter active, add normally
                                app.stories_by_state
                                    .entry(new_story.workflow_state_id)
                                    .or_default()
                                    .push(new_story.clone());

                                // Sort stories by position
                                if let Some(stories) =
                                    app.stories_by_state.get_mut(&workflow_state_id)
                                {
                                    stories.sort_by_key(|s| s.position);
                                }

                                // Update list view
                                app.all_stories_list.push(new_story);
                                app.all_stories_list.sort_by_key(|s| s.position);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to create story: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get current member: {e}");
                }
            }

            // Reset the popup state
            app.create_popup_state = ui::CreatePopupState::default();
            app.create_story_requested = false;
        }

        // Check if we need to edit a story
        if app.edit_story_requested
            && !app
                .edit_popup_state
                .name_textarea
                .lines()
                .join("")
                .trim()
                .is_empty()
        {
            let story_id = app.edit_popup_state.story_id;
            let name = app.edit_popup_state.name_textarea.lines().join("");
            let description = app.edit_popup_state.description_textarea.lines().join("");
            let story_type = app.edit_popup_state.story_type.clone();
            let epic_id = app.edit_popup_state.epic_id;

            match client.update_story_details(story_id, name, description, story_type, epic_id) {
                Ok(updated_story) => {
                    // Update the story in our local data
                    update_story_details(&mut app, story_id, updated_story);
                    if debug {
                        eprintln!("Successfully updated story #{story_id}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to update story: {e}");
                }
            }

            // Reset the edit state
            app.edit_popup_state = ui::EditPopupState {
                name_textarea: {
                    let mut textarea = tui_textarea::TextArea::default();
                    textarea.set_cursor_line_style(ratatui::style::Style::default());
                    textarea.set_block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .title("Name"),
                    );
                    textarea
                },
                description_textarea: {
                    let mut textarea = tui_textarea::TextArea::default();
                    textarea.set_cursor_line_style(ratatui::style::Style::default());
                    textarea.set_block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .title("Description"),
                    );
                    textarea
                },
                story_type: "feature".to_string(),
                selected_field: ui::EditField::Name,
                story_type_index: 0,
                story_id: 0,
                epic_id: None,
                epic_selector_index: 0,
            };
            app.edit_story_requested = false;
        }

        // Check if we need to add a comment
        if app.add_comment_requested {
            let comment_text = app.comment_popup_state.comment_textarea.lines().join("\n");
            let story_id = app.comment_popup_state.story_id;

            if !comment_text.trim().is_empty() {
                match client.add_comment(story_id, &comment_text) {
                    Ok(_) => {
                        if debug {
                            eprintln!("✅ Comment added to story #{}", story_id);
                        }
                        // Refresh the story to show the new comment
                        if let Ok(updated_story) = client.get_story(story_id) {
                            update_story_state(&mut app, story_id, updated_story);
                        }
                    }
                    Err(e) => {
                        if debug {
                            eprintln!("⚠️ Failed to add comment: {}", e);
                        }
                    }
                }
            }

            // Reset comment state
            app.add_comment_requested = false;
            app.comment_popup_state = ui::CommentPopupState {
                comment_textarea: {
                    let mut ta = tui_textarea::TextArea::default();
                    ta.set_cursor_line_style(ratatui::style::Style::default());
                    ta.set_placeholder_text("Enter your comment here...");
                    ta
                },
                story_id: 0,
            };
        }

        // Check if we need to create a new epic
        if app.create_epic_requested
            && !app
                .create_epic_popup_state
                .name_textarea
                .lines()
                .join("")
                .trim()
                .is_empty()
        {
            let name = app.create_epic_popup_state.name_textarea.lines().join("");
            let description = app.create_epic_popup_state.description_textarea.lines().join("\n");

            match client.create_epic(name, description) {
                Ok(new_epic) => {
                    // Add the new epic to our epic list
                    app.epics.push(new_epic.clone());
                    if debug {
                        eprintln!("Successfully created epic: {}", new_epic.name);
                    }
                    // Reset the popup state
                    app.create_epic_popup_state = ui::CreateEpicPopupState {
                        name_textarea: {
                            let mut textarea = tui_textarea::TextArea::default();
                            textarea.set_cursor_line_style(ratatui::style::Style::default());
                            textarea.set_block(
                                ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .title("Epic Name"),
                            );
                            textarea
                        },
                        description_textarea: {
                            let mut textarea = tui_textarea::TextArea::default();
                            textarea.set_cursor_line_style(ratatui::style::Style::default());
                            textarea.set_block(
                                ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .title("Description"),
                            );
                            textarea
                        },
                        selected_field: ui::CreateEpicField::Name,
                    };
                }
                Err(e) => {
                    eprintln!("Failed to create epic: {e}");
                }
            }

            app.create_epic_requested = false;
        }

        // Check if we need to handle git branch creation
        if app.git_branch_requested {
            let selected_option = app.git_popup_state.selected_option.clone();

            // Handle cancel early
            if selected_option == ui::GitBranchOption::Cancel {
                app.git_branch_requested = false;
                app.git_popup_state = ui::GitBranchPopupState::default();
                continue;
            }

            // Build the git operation request
            let request = git::operations::GitBranchRequest {
                branch_name: app.git_popup_state.branch_name_textarea.lines().join(""),
                worktree_path: app.git_popup_state.worktree_path_textarea.lines().join(""),
                operation: match selected_option {
                    ui::GitBranchOption::CreateBranch => git::operations::GitOperation::CreateBranch,
                    ui::GitBranchOption::CreateWorktree => {
                        git::operations::GitOperation::CreateWorktree
                    }
                    ui::GitBranchOption::Cancel => unreachable!(),
                },
                story_id: app.git_popup_state.story_id,
            };

            // Execute the git operation
            let result = git::operations::execute_git_operation(&request);

            // Move story to In Progress if operation was successful
            if result.success
                && let Some(updated_story) = git::operations::move_story_to_in_progress(
                    &client,
                    result.story_id,
                    &app.workflows,
                    debug,
                )
            {
                update_story_state(&mut app, result.story_id, updated_story);
            }

            // Convert to UI result state
            let operation_type = match result.operation {
                git::operations::GitOperation::CreateBranch => ui::GitOperationType::CreateBranch,
                git::operations::GitOperation::CreateWorktree => ui::GitOperationType::CreateWorktree,
            };

            app.git_result_state = ui::GitResultState {
                success: result.success,
                operation_type,
                message: result.message,
                branch_name: result.branch_name,
                worktree_path: result.worktree_path.clone(),
                story_id: result.story_id,
                selected_option: if result.worktree_path.is_some() && result.success {
                    ui::GitResultOption::ExitAndChange
                } else {
                    ui::GitResultOption::Continue
                },
            };
            app.show_git_result_popup = true;

            // Reset git request state
            app.git_branch_requested = false;
            app.git_popup_state = ui::GitBranchPopupState::default();
        }

        // Check if we need to refresh all stories
        if app.refresh_requested {
            // Reset the refresh flag
            app.refresh_requested = false;

            // Reload the first page of stories
            match client.search_stories_page(&app.search_query, None) {
                Ok(search_result) => {
                    if debug {
                        eprintln!("Refreshed with {} stories", search_result.stories.len());
                    }

                    // Create a fresh app instance with the new data
                    let new_app = App::new(
                        search_result.stories,
                        workflows.clone(),
                        app.search_query.clone(),
                        search_result.next_page_token,
                    );

                    // Preserve member cache, user ID, and epics from the old app
                    let old_member_cache = app.member_cache.clone();
                    let old_user_id = app.current_user_id.clone();
                    let old_epics = app.epics.clone();

                    // Replace the app with fresh data
                    app = new_app;

                    // Restore member cache, user ID, and epics
                    app.member_cache = old_member_cache;
                    app.epics = old_epics;
                    app.current_user_id = old_user_id;

                    app.is_loading = false;
                }
                Err(e) => {
                    eprintln!("Failed to refresh stories: {e}");
                    app.is_loading = false;
                    app.refresh_requested = false;
                }
            }
        }
        // Check if we need to load more stories
        else if app.load_more_requested {
            if let Some(ref next_token) = app.next_page_token.clone() {
                match client.search_stories_page(&app.search_query, Some(next_token.clone())) {
                    Ok(search_result) => {
                        if debug {
                            eprintln!("Loaded {} more stories", search_result.stories.len());
                        }
                        // Merge the new stories
                        app.merge_stories(search_result.stories, search_result.next_page_token);
                    }
                    Err(e) => {
                        eprintln!("Failed to load more stories: {e}");
                        app.is_loading = false;
                        app.load_more_requested = false;
                    }
                }
            } else {
                app.is_loading = false;
                app.load_more_requested = false;
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Check if we need to exit and change directory for worktree
    if let Ok(worktree_path) = std::env::var("SC_CLI_EXIT_AND_CD") {
        // Remove the environment variable
        unsafe {
            std::env::remove_var("SC_CLI_EXIT_AND_CD");
        }

        if debug {
            eprintln!("Exiting and changing to worktree directory: {worktree_path}");
        }

        eprintln!("\n🚀 Exiting application.");
        eprintln!("📁 Change to the worktree directory with:");
        eprintln!("   cd {worktree_path}");
    }

    Ok(())
}

fn update_story_state(app: &mut App, story_id: i64, updated_story: api::Story) {
    // Update the story in the unfiltered list
    if let Some(pos) = app
        .all_stories_unfiltered
        .iter()
        .position(|s| s.id == story_id)
    {
        app.all_stories_unfiltered[pos] = updated_story.clone();
    }

    // Find and remove the story from its current state
    let mut old_state_id = None;
    for (&state_id, stories) in app.stories_by_state.iter_mut() {
        if let Some(pos) = stories.iter().position(|s| s.id == story_id) {
            stories.remove(pos);
            old_state_id = Some(state_id);
            break;
        }
    }

    // Add the story to its new state
    app.stories_by_state
        .entry(updated_story.workflow_state_id)
        .or_default()
        .push(updated_story.clone());

    // Update the all_stories_list for list view
    if let Some(pos) = app.all_stories_list.iter().position(|s| s.id == story_id) {
        app.all_stories_list[pos] = updated_story;
    }

    // If we removed from the current column and it's now empty, reset selected_row
    if let Some(old_id) = old_state_id
        && app
            .workflow_states
            .get(app.selected_column)
            .map(|(id, _)| *id)
            == Some(old_id)
        && let Some(stories) = app.stories_by_state.get(&old_id)
        && (stories.is_empty() || app.selected_row >= stories.len())
    {
        app.selected_row = 0;
    }
}

fn update_story_ownership(app: &mut App, story_id: i64, updated_story: api::Story) {
    // Update the story in the unfiltered list
    if let Some(pos) = app
        .all_stories_unfiltered
        .iter()
        .position(|s| s.id == story_id)
    {
        app.all_stories_unfiltered[pos] = updated_story.clone();
    }

    // Find and update the story in its current state
    let state_id = updated_story.workflow_state_id;
    if let Some(stories) = app.stories_by_state.get_mut(&state_id)
        && let Some(pos) = stories.iter().position(|s| s.id == story_id)
    {
        stories[pos] = updated_story.clone();
    }

    // Also update the story in the all_stories_list for list view
    if let Some(pos) = app.all_stories_list.iter().position(|s| s.id == story_id) {
        app.all_stories_list[pos] = updated_story;
    }
}

fn update_story_details(app: &mut App, story_id: i64, updated_story: api::Story) {
    // Update the story in the unfiltered list first
    if let Some(pos) = app
        .all_stories_unfiltered
        .iter()
        .position(|s| s.id == story_id)
    {
        app.all_stories_unfiltered[pos] = updated_story.clone();
    }

    // If there's an epic filter active, reapply it
    if app.selected_epic_filter.is_some() {
        app.apply_epic_filter();
        // Rebuild the list view after filtering
        app.all_stories_list = app.stories_by_state.values().flatten().cloned().collect();
        app.all_stories_list.sort_by_key(|s| s.position);
    } else {
        // No filter active, update normally
        // Find and update the story in its current state
        let state_id = updated_story.workflow_state_id;
        if let Some(stories) = app.stories_by_state.get_mut(&state_id)
            && let Some(pos) = stories.iter().position(|s| s.id == story_id)
        {
            stories[pos] = updated_story.clone();
        }

        // Also update the story in the all_stories_list for list view
        if let Some(pos) = app.all_stories_list.iter().position(|s| s.id == story_id) {
            app.all_stories_list[pos] = updated_story;
        }
    }
}

fn handle_show_command(args: ShowCommandArgs) -> Result<()> {
    // Get token, username, and config from args or config (similar to view command)
    let (api_token, search_username, _config_limit) = if let Some(workspace_name) = args.workspace {
        // Use explicitly specified workspace
        let (config, _created) =
            Config::load_or_create(&workspace_name).context("Failed to load or create config")?;
        let workspace_config = config
            .get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        (
            workspace_config.api_key.clone(),
            workspace_config.user_id.clone(),
            workspace_config.fetch_limit,
        )
    } else if args.token.is_none() && args.username.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let workspace_config =
                        config
                            .get_workspace(&default_workspace_name)
                            .context(format!(
                                "Failed to get default workspace '{default_workspace_name}'"
                            ))?;
                    (
                        workspace_config.api_key.clone(),
                        workspace_config.user_id.clone(),
                        workspace_config.fetch_limit,
                    )
                } else {
                    anyhow::bail!(
                        "No default workspace configured. Use --workspace to specify one or provide --token and username"
                    );
                }
            }
            Err(_) => {
                anyhow::bail!(
                    "No configuration file found. Use --workspace to create one or provide --token and username"
                );
            }
        }
    } else {
        // Use command line arguments
        let api_token = args
            .token
            .ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?;
        let search_username = args
            .username
            .ok_or_else(|| anyhow::anyhow!("Either username or --workspace must be provided"))?;
        (api_token, search_username, 50) // Default limit when not using workspace
    };

    // Initialize API client
    let client =
        ShortcutClient::new(api_token, args.debug).context("Failed to create Shortcut client")?;

    // Build search query (similar to view command)
    let query = if let Some(search_query) = args.search {
        search_query
    } else {
        let mut query_parts = vec![];

        // Apply filter based on flags (default to owner if none specified)
        if args.all {
            // No user filter for --all flag
        } else if args.requester {
            query_parts.push(format!("requester:{search_username}"));
        } else {
            // Default to owner filter (also when --owner is explicitly used)
            query_parts.push(format!("owner:{search_username}"));
        }

        if let Some(story_type) = args.story_type {
            query_parts.push(format!("type:{story_type}"));
        }

        query_parts.push("is:story".to_string());
        query_parts.join(" ")
    };

    if args.debug {
        eprintln!("Search query: {query}");
        eprintln!("Stories per page: {}", args.limit);
    }

    // Get workflows for state name resolution
    let workflows = client
        .get_workflows()
        .context("Failed to fetch workflows")?;

    // Build workflow state map
    let mut workflow_state_map = std::collections::HashMap::new();
    for workflow in &workflows {
        for state in &workflow.states {
            workflow_state_map.insert(state.id, state.name.clone());
        }
    }

    // Fetch members for owner name resolution
    let mut member_cache = std::collections::HashMap::new();
    if args.debug {
        eprintln!("Fetching members for name resolution...");
    }
    match client.get_members() {
        Ok(members) => {
            for member in members {
                let display_name =
                    format!("{} ({})", member.profile.name, member.profile.mention_name);
                member_cache.insert(member.id, display_name);
            }
            if args.debug {
                eprintln!("Cached {} members", member_cache.len());
            }
        }
        Err(e) => {
            if args.debug {
                eprintln!("WARNING: Failed to fetch members: {e}");
                eprintln!("Owner names will be displayed as IDs");
            }
        }
    }

    // Start pagination
    show_stories_paginated(
        &client,
        &query,
        args.limit,
        args.debug,
        &workflow_state_map,
        &member_cache,
    )
}

fn show_stories_paginated(
    client: &ShortcutClient,
    query: &str,
    page_size: usize,
    debug: bool,
    workflow_state_map: &std::collections::HashMap<i64, String>,
    member_cache: &std::collections::HashMap<String, String>,
) -> Result<()> {
    use std::io::{self, Write};

    let mut next_page_token: Option<String> = None;
    let mut total_shown = 0;
    let mut current_batch: Vec<api::Story> = Vec::new();
    let mut batch_index = 0;

    loop {
        // If we need more stories (either first time or current batch exhausted)
        if current_batch.is_empty() || batch_index >= current_batch.len() {
            if current_batch.is_empty() {
                // First fetch
                if debug {
                    eprintln!("Making initial API call...");
                }
                let search_result = client
                    .search_stories_page(query, None)
                    .context("Failed to search stories")?;

                if search_result.stories.is_empty() {
                    println!("\x1b[33m🔍 No stories found for query: {query}\x1b[0m");
                    println!(
                        "\x1b[37m💡 Try using a different search query or check if the username is correct.\x1b[0m"
                    );
                    break;
                }

                current_batch = search_result.stories;
                batch_index = 0;
                next_page_token = search_result.next_page_token;

                if debug {
                    eprintln!(
                        "Initial fetch: {} stories, next_token: {:?}",
                        current_batch.len(),
                        next_page_token
                    );
                }
            } else if next_page_token.is_some() {
                // Fetch next batch from API
                if debug {
                    eprintln!("Fetching next batch from API...");
                }
                let search_result = client
                    .search_stories_page(query, next_page_token.clone())
                    .context("Failed to search stories")?;

                if search_result.stories.is_empty() {
                    println!("\x1b[32m🎉 End of stories\x1b[0m");
                    break;
                }

                current_batch = search_result.stories;
                batch_index = 0;
                next_page_token = search_result.next_page_token;

                if debug {
                    eprintln!(
                        "Fetched {} stories from API, next_token: {:?}",
                        current_batch.len(),
                        next_page_token
                    );
                }
            } else {
                // No more stories available
                println!("\x1b[32m🎉 End of stories\x1b[0m");
                break;
            }
        }

        // Display page_size stories from current batch
        let end_index = std::cmp::min(batch_index + page_size, current_batch.len());
        let stories_to_show = &current_batch[batch_index..end_index];

        if debug {
            eprintln!(
                "Showing stories {} to {} from current batch",
                batch_index,
                end_index - 1
            );
        }

        for story in stories_to_show {
            // Story title with bright cyan color and lightning bolt emoji
            println!("\x1b[1;36m⚡ #{} - {}\x1b[0m", story.id, story.name);

            if !story.description.is_empty() {
                let first_line = story.description.lines().next().unwrap_or("");
                if !first_line.is_empty() {
                    // Description with light gray color and document emoji
                    println!("   \x1b[37m📄 {first_line}\x1b[0m");
                }
            }

            if !story.owner_ids.is_empty() {
                let owner_names: Vec<String> = story
                    .owner_ids
                    .iter()
                    .map(|id| member_cache.get(id).cloned().unwrap_or_else(|| id.clone()))
                    .collect();
                // Owners with yellow color and person emoji
                println!("   \x1b[33m👤 Owner(s): {}\x1b[0m", owner_names.join(", "));
            }

            let state_name = workflow_state_map
                .get(&story.workflow_state_id)
                .cloned()
                .unwrap_or_else(|| story.workflow_state_id.to_string());

            // Get emoji and color based on story type
            let (type_emoji, type_color) = match story.story_type.as_str() {
                "feature" => ("✨", "\x1b[32m"), // Green for features
                "bug" => ("🐞", "\x1b[31m"),     // Red for bugs
                "chore" => ("⚙️", "\x1b[34m"),   // Blue for chores
                _ => ("📝", "\x1b[37m"),         // Default gray
            };

            // Get emoji based on state name
            let state_emoji = match state_name.to_lowercase().as_str() {
                name if name.contains("todo") || name.contains("backlog") => "📋",
                name if name.contains("progress") || name.contains("doing") => "🔄",
                name if name.contains("review") => "👀",
                name if name.contains("done") || name.contains("complete") => "✅",
                _ => "📌",
            };

            // State, type, and URL with appropriate colors and emojis
            println!(
                "   {} \x1b[35m{}\x1b[0m | {}{} {}\x1b[0m | \x1b[36m🔗 {}\x1b[0m",
                state_emoji, state_name, type_emoji, type_color, story.story_type, story.app_url
            );
            println!(); // Empty line between stories
        }

        total_shown += stories_to_show.len();
        batch_index = end_index;

        // Check if we have more stories to show (either in current batch or from API)
        let has_more = batch_index < current_batch.len() || next_page_token.is_some();

        if !has_more {
            println!("\x1b[32m🎉 End of stories\x1b[0m");
            break;
        }

        // Show pagination prompt with colors and emojis
        print!(
            "\x1b[1;44m📖 More \x1b[0m \x1b[36m({total_shown} stories shown, press \x1b[1;33mSPACE\x1b[0m\x1b[36m to continue, \x1b[1;33mq\x1b[0m\x1b[36m to quit)\x1b[0m"
        );
        io::stdout().flush()?;

        // Wait for user input
        match wait_for_spacebar() {
            Ok(true) => {
                continue; // Continue to next page
            }
            Ok(false) => {
                println!("\n\x1b[33m👋 Goodbye!\x1b[0m");
                break;
            }
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        }
    }

    Ok(())
}

fn wait_for_spacebar() -> Result<bool> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::io::{self, Write};

    // Enable raw mode to capture single keystrokes
    enable_raw_mode()?;

    loop {
        // Wait for key event
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            disable_raw_mode()?;

            match code {
                KeyCode::Char(' ') => {
                    // Clear the prompt line
                    print!("\r{}\r", " ".repeat(50));
                    io::stdout().flush()?;
                    return Ok(true);
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    // Clear the prompt line
                    print!("\r{}\r", " ".repeat(50));
                    io::stdout().flush()?;
                    return Ok(false);
                }
                _ => {
                    // Any other key quits
                    print!("\r{}\r", " ".repeat(50));
                    io::stdout().flush()?;
                    return Ok(false);
                }
            }
        }
    }
}
