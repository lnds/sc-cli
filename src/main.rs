mod api;
mod config;
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
use std::io;
use ui::App;

#[derive(Parser, Debug)]
#[command(author, version, about = "TUI client for Shortcut stories", long_about = None)]
struct Args {
    /// Shortcut mention name to search for (optional if using workspace)
    username: Option<String>,

    /// Shortcut API token (optional if using workspace)
    #[arg(short, long)]
    token: Option<String>,

    /// Workspace name from config file
    #[arg(short, long)]
    workspace: Option<String>,

    /// Maximum number of stories to display
    #[arg(short, long, default_value_t = 25)]
    limit: usize,

    /// Filter by story type (feature, bug, chore)
    #[arg(long)]
    story_type: Option<String>,

    /// Custom search query using Shortcut's search syntax
    #[arg(short, long)]
    search: Option<String>,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Get token and username from args or config
    let (token, username) = if let Some(workspace_name) = args.workspace {
        // Load config and get workspace settings, creating if necessary
        let (config, _created) = Config::load_or_create(&workspace_name)
            .context("Failed to load or create config")?;
        let workspace = config.get_workspace(&workspace_name)
            .context(format!("Failed to get workspace '{}'", workspace_name))?;
        (workspace.api_key.clone(), workspace.user_id.clone())
    } else {
        // Use command line arguments
        let token = args.token
            .ok_or_else(|| anyhow::anyhow!("Either --token or --workspace must be provided"))?;
        let username = args.username
            .ok_or_else(|| anyhow::anyhow!("Either username or --workspace must be provided"))?;
        (token, username)
    };

    // Initialize API client
    let client = ShortcutClient::new(token, args.debug)
        .context("Failed to create Shortcut client")?;

    // Get workflows
    if args.debug {
        eprintln!("Fetching workflows...");
    }
    let workflows = client
        .get_workflows()
        .context("Failed to fetch workflows")?;

    // Build search query
    let query = if let Some(search) = args.search {
        search
    } else {
        let mut query_parts = vec![format!("owner:{}", username)];
        
        if let Some(story_type) = args.story_type {
            query_parts.push(format!("type:{story_type}"));
        }
        
        query_parts.push("is:story".to_string());
        query_parts.join(" ")
    };

    // Search for stories
    if args.debug {
        eprintln!("Searching for stories...");
        eprintln!("Query: {query}");
    }
    let mut stories = client
        .search_stories(&query)
        .context("Failed to search stories")?;

    if stories.is_empty() {
        eprintln!("No stories found for query: {query}");
        eprintln!("Try using a different search query or check if the username is correct.");
        return Ok(());
    }

    if args.debug {
        eprintln!("Found {} stories", stories.len());
    }
    
    // Limit results
    stories.truncate(args.limit);
    if args.debug {
        eprintln!("Displaying {} stories", stories.len());
    }

    // Setup terminal
    setup_terminal()?;

    // Run app
    let app = App::new(stories, workflows);
    let result = run_app(app);

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

fn run_app(mut app: App) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        app.handle_events()?;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}