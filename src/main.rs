mod api;
mod config;
mod story_creator;
mod ui;

use anyhow::{Context, Result};
use api::{client::ShortcutClient, ShortcutApi};
use clap::Parser;
use config::Config;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, collections::HashMap};
use story_creator::StoryCreator;
use ui::App;

fn validate_story_type(s: &str) -> Result<String, String> {
    match s {
        "feature" | "bug" | "chore" => Ok(s.to_string()),
        _ => Err(format!("Invalid story type '{}'. Must be one of: feature, bug, chore", s)),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "TUI client for Shortcut stories", long_about = None)]
struct Args {
    /// Workspace name from config file (optional if default workspace is set)
    #[arg(short, long, global = true)]
    workspace: Option<String>,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Add { name, token, r#type }) => {
            handle_add_command(args.workspace, token, name, r#type, args.debug)
        }
        Some(Command::View { username, token, limit, story_type, search, all, owner, requester }) => {
            handle_view_command(args.workspace, username, token, limit, story_type, search, all, owner, requester, args.debug)
        }
        None => {
            // Default to view command when no subcommand is specified
            handle_view_command(args.workspace, None, None, None, None, None, false, false, false, args.debug)
        }
    }
}

fn handle_add_command(workspace: Option<String>, token: Option<String>, name: Vec<String>, story_type: Option<String>, debug: bool) -> Result<()> {
    // Get token and user info from args or config
    // Priority: 1. Explicit workspace, 2. Default workspace (if no token), 3. Token from CLI
    let (token, _username) = if let Some(workspace_name) = workspace {
        // Use explicitly specified workspace
        let (config, _created) = Config::load_or_create(&workspace_name)
            .context("Failed to load or create config")?;
        let workspace = config.get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        (workspace.api_key.clone(), workspace.user_id.clone())
    } else if token.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let workspace = config.get_workspace(&default_workspace_name)
                        .context(format!("Failed to get default workspace '{default_workspace_name}'"))?;
                    (workspace.api_key.clone(), workspace.user_id.clone())
                } else {
                    anyhow::bail!("No default workspace configured. Use --workspace to specify one or provide --token");
                }
            }
            Err(_) => {
                anyhow::bail!("No configuration file found. Use --workspace to create one or provide --token");
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
    let client = ShortcutClient::new(token, debug)
        .context("Failed to create Shortcut client")?;

    // Get current member info to use as requester
    let current_member = client.get_current_member()
        .context("Failed to get current member info")?;
    
    if debug {
        eprintln!("Current user: {} ({}) - ID: {}", current_member.name, current_member.mention_name, current_member.id);
    }

    // Get workflows to find the appropriate initial state
    let workflows = client.get_workflows()
        .context("Failed to fetch workflows")?;
    
    // Find the first workflow and get its first state (typically "Backlog" or "To Do")
    let workflow_state_id = workflows.first()
        .and_then(|w| w.states.first())
        .map(|s| s.id)
        .ok_or_else(|| anyhow::anyhow!("No workflows found in the workspace"))?;
    
    if debug {
        eprintln!("Using workflow state ID: {}", workflow_state_id);
    }

    // Convert name vector to optional string
    let name_str = if name.is_empty() {
        None
    } else {
        Some(name.join(" "))
    };

    // Use StoryCreator to gather input and create the story
    let story_creator = StoryCreator::from_prompts(current_member.id, workflow_state_id, name_str, story_type)?;
    
    if debug {
        eprintln!("Creating story:");
        eprintln!("  Name: {}", story_creator.name);
        eprintln!("  Type: {}", story_creator.story_type);
        eprintln!("  Description length: {} chars", story_creator.description.len());
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


fn handle_view_command(
    workspace: Option<String>, 
    username: Option<String>, 
    token: Option<String>, 
    limit: Option<usize>, 
    story_type: Option<String>, 
    search: Option<String>, 
    all: bool, 
    _owner: bool, 
    requester: bool,
    debug: bool
) -> Result<()> {
    // Get token, username, and fetch_limit from args or config
    let (token, username, config_limit) = if let Some(workspace_name) = workspace {
        // Use explicitly specified workspace
        let (config, _created) = Config::load_or_create(&workspace_name)
            .context("Failed to load or create config")?;
        let workspace = config.get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{workspace_name}'"))?;
        (workspace.api_key.clone(), workspace.user_id.clone(), workspace.fetch_limit)
    } else if token.is_none() && username.is_none() {
        // No args provided, try to use default workspace
        match Config::load() {
            Ok(config) => {
                if let Some(default_workspace_name) = config.get_default_workspace() {
                    let workspace = config.get_workspace(&default_workspace_name)
                        .context(format!("Failed to get default workspace '{default_workspace_name}'"))?;
                    (workspace.api_key.clone(), workspace.user_id.clone(), workspace.fetch_limit)
                } else {
                    anyhow::bail!("No default workspace configured. Use --workspace to specify one or provide --token and username");
                }
            }
            Err(_) => {
                anyhow::bail!("No configuration file found. Use --workspace to create one or provide --token and username");
            }
        }
    } else {
        // Use command line arguments with default limit
        let token = token
            .ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?;
        let username = username
            .ok_or_else(|| anyhow::anyhow!("Either username or --workspace must be provided"))?;
        (token, username, 20) // Default limit when not using workspace
    };
    
    // Use command-line limit if provided, otherwise use workspace config limit
    let limit = limit.unwrap_or(config_limit);

    // Initialize API client
    let client = ShortcutClient::new(token, debug)
        .context("Failed to create Shortcut client")?;

    // Get workflows
    if debug {
        eprintln!("Fetching workflows...");
    }
    let workflows = client
        .get_workflows()
        .context("Failed to fetch workflows")?;

    // Build search query
    let query = if let Some(search) = search {
        search
    } else {
        let mut query_parts = vec![];
        
        // Apply filter based on flags (default to owner if none specified)
        if all {
            // No user filter for --all flag
        } else if requester {
            query_parts.push(format!("requester:{username}"));
        } else {
            // Default to owner filter (also when --owner is explicitly used)
            query_parts.push(format!("owner:{username}"));
        }
        
        if let Some(story_type) = story_type {
            query_parts.push(format!("type:{story_type}"));
        }
        
        query_parts.push("is:story".to_string());
        query_parts.join(" ")
    };

    // Search for stories
    if debug {
        eprintln!("Searching for stories...");
        eprintln!("Query: {query}");
    }
    let stories = client
        .search_stories(&query, Some(limit))
        .context("Failed to search stories")?;

    if stories.is_empty() {
        eprintln!("No stories found for query: {query}");
        eprintln!("Try using a different search query or check if the username is correct.");
        return Ok(());
    }

    if debug {
        eprintln!("Found {} stories", stories.len());
    }

    // Fetch members to populate cache BEFORE setting up terminal
    let mut member_cache = HashMap::new();
    if debug {
        eprintln!("Fetching members for cache...");
    }
    match client.get_members() {
        Ok(members) => {
            if debug {
                eprintln!("Fetched {} members from API", members.len());
            }
            for member in members {
                if debug {
                    eprintln!("Caching member: id='{}', name='{}', mention_name='{}'", 
                        member.id, member.profile.name, member.profile.mention_name);
                }
                // Store name with mention_name in parentheses
                let display_name = format!("{} ({})", member.profile.name, member.profile.mention_name);
                member_cache.insert(member.id, display_name);
            }
            if debug {
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
            if debug {
                eprintln!("Full error: {e:?}");
            }
            eprintln!("Owner names will be displayed as IDs");
        }
    }
    
    // Setup terminal AFTER fetching members
    setup_terminal()?;
    
    // Create app with stories and workflows
    let mut app = App::new(stories, workflows.clone());
    
    // Populate the member cache in the app
    for (id, name) in member_cache {
        app.add_member_to_cache(id, name);
    }
    
    // Try to get current user ID to highlight owned stories
    if debug {
        eprintln!("Fetching current user for story highlighting...");
    }
    match client.get_current_member() {
        Ok(member) => {
            if debug {
                eprintln!("Current user: {} ({}) - ID: {}", member.name, member.mention_name, member.id);
            }
            app.set_current_user_id(member.id);
        }
        Err(e) => {
            if debug {
                eprintln!("Failed to get current user for highlighting: {e}");
                eprintln!("Owned stories will not be highlighted");
            }
        }
    }
    
    let result = run_app(app, client, workflows);

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

fn run_app(mut app: App, client: ShortcutClient, workflows: Vec<api::Workflow>) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    // Special handling for Enter in state selector
                    if app.show_state_selector && key.code == crossterm::event::KeyCode::Enter {
                        let story_update = app.get_selected_story().map(|story| {
                            (story.id, app.get_selected_target_state())
                        });
                        
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
        if app.create_story_requested && !app.create_popup_state.name.is_empty() {
            // Get current member info to use as requester
            match client.get_current_member() {
                Ok(current_member) => {
                    // Find the first workflow state
                    let workflow_state_id = workflows.first()
                        .and_then(|w| w.states.first())
                        .map(|s| s.id)
                        .unwrap_or(500000007); // Default to "To Do" if not found
                    
                    // Create the story using the popup data
                    let story_creator = StoryCreator::new(
                        app.create_popup_state.name.clone(),
                        app.create_popup_state.description.clone(),
                        app.create_popup_state.story_type.clone(),
                        current_member.id,
                        workflow_state_id,
                    );
                    
                    match story_creator.create(&client) {
                        Ok(new_story) => {
                            // Add the new story to the app
                            app.stories_by_state
                                .entry(new_story.workflow_state_id)
                                .or_default()
                                .push(new_story);
                            
                            // Sort stories by position
                            if let Some(stories) = app.stories_by_state.get_mut(&workflow_state_id) {
                                stories.sort_by_key(|s| s.position);
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

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn update_story_state(app: &mut App, story_id: i64, updated_story: api::Story) {
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
        .push(updated_story);

    // If we removed from the current column and it's now empty, reset selected_row
    if let Some(old_id) = old_state_id {
        if app.workflow_states.get(app.selected_column).map(|(id, _)| *id) == Some(old_id) {
            if let Some(stories) = app.stories_by_state.get(&old_id) {
                if stories.is_empty() || app.selected_row >= stories.len() {
                    app.selected_row = 0;
                }
            }
        }
    }
}

fn update_story_ownership(app: &mut App, story_id: i64, updated_story: api::Story) {
    // Find and update the story in its current state
    let state_id = updated_story.workflow_state_id;
    if let Some(stories) = app.stories_by_state.get_mut(&state_id) {
        if let Some(pos) = stories.iter().position(|s| s.id == story_id) {
            stories[pos] = updated_story;
        }
    }
}

