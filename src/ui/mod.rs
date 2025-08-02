use crate::api::{Story, Workflow};
use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

pub struct App {
    pub show_detail: bool,
    pub show_state_selector: bool,
    pub state_selector_index: usize,
    pub take_ownership_requested: bool,
    pub create_story_requested: bool,
    pub show_create_popup: bool,
    pub create_popup_state: CreatePopupState,
    pub workflow_state_map: HashMap<i64, String>,
    pub member_cache: HashMap<String, String>, // owner_id -> name
    pub current_user_id: Option<String>, // ID of current user
    pub detail_scroll_offset: usize, // Scroll offset for detail popup
    pub should_quit: bool,
    pub selected_column: usize,
    pub selected_row: usize,
    pub stories_by_state: HashMap<i64, Vec<Story>>,
    pub workflow_states: Vec<(i64, String)>,
}

#[derive(Debug, Clone)]
pub struct CreatePopupState {
    pub name: String,
    pub description: String,
    pub story_type: String,
    pub selected_field: CreateField,
    pub story_type_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateField {
    Name,
    Description,
    Type,
}

impl Default for CreatePopupState {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            story_type: "feature".to_string(),
            selected_field: CreateField::Name,
            story_type_index: 0,
        }
    }
}

impl App {
    pub fn new(stories: Vec<Story>, workflows: Vec<Workflow>) -> Self {
        // Group stories by workflow state
        let mut stories_by_state: HashMap<i64, Vec<Story>> = HashMap::new();
        for story in stories.iter() {
            stories_by_state
                .entry(story.workflow_state_id)
                .or_default()
                .push(story.clone());
        }
        
        // Sort stories within each state by position
        for stories in stories_by_state.values_mut() {
            stories.sort_by_key(|s| s.position);
        }
        
        // Create workflow state map for quick lookups
        let mut workflow_state_map = HashMap::new();
        let mut state_positions: HashMap<i64, i64> = HashMap::new();
        
        for workflow in workflows.iter() {
            for state in workflow.states.iter() {
                workflow_state_map.insert(state.id, state.name.clone());
                state_positions.insert(state.id, state.position);
            }
        }
        
        // Get ordered list of ALL workflow states, sorted by position
        let mut workflow_states: Vec<(i64, String)> = workflow_state_map
            .iter()
            .map(|(&id, name)| (id, name.clone()))
            .collect();
        
        // Sort by position attribute
        workflow_states.sort_by_key(|(id, _)| {
            state_positions.get(id).copied().unwrap_or(i64::MAX)
        });

        Self {
            show_detail: false,
            show_state_selector: false,
            state_selector_index: 0,
            take_ownership_requested: false,
            create_story_requested: false,
            show_create_popup: false,
            create_popup_state: CreatePopupState::default(),
            workflow_state_map,
            member_cache: HashMap::new(),
            current_user_id: None,
            detail_scroll_offset: 0,
            should_quit: false,
            selected_column: 0,
            selected_row: 0,
            stories_by_state,
            workflow_states,
        }
    }

    pub fn next(&mut self) {
        if self.workflow_states.is_empty() {
            return;
        }
        
        let state_id = self.workflow_states[self.selected_column].0;
        if let Some(stories) = self.stories_by_state.get(&state_id) {
            if !stories.is_empty() {
                self.selected_row = (self.selected_row + 1) % stories.len();
            }
        }
    }

    pub fn previous(&mut self) {
        if self.workflow_states.is_empty() {
            return;
        }
        
        let state_id = self.workflow_states[self.selected_column].0;
        if let Some(stories) = self.stories_by_state.get(&state_id) {
            if !stories.is_empty() {
                if self.selected_row == 0 {
                    self.selected_row = stories.len() - 1;
                } else {
                    self.selected_row -= 1;
                }
            }
        }
    }
    
    pub fn next_column(&mut self) {
        if !self.workflow_states.is_empty() {
            self.selected_column = (self.selected_column + 1) % self.workflow_states.len();
            self.selected_row = 0;
        }
    }
    
    pub fn previous_column(&mut self) {
        if !self.workflow_states.is_empty() {
            if self.selected_column == 0 {
                self.selected_column = self.workflow_states.len() - 1;
            } else {
                self.selected_column -= 1;
            }
            self.selected_row = 0;
        }
    }

    pub fn toggle_detail(&mut self) {
        if !self.workflow_states.is_empty() {
            let state_id = self.workflow_states[self.selected_column].0;
            if let Some(stories) = self.stories_by_state.get(&state_id) {
                if !stories.is_empty() {
                    self.show_detail = !self.show_detail;
                    // Reset scroll offset when opening detail view
                    if self.show_detail {
                        self.detail_scroll_offset = 0;
                    }
                }
            }
        }
    }
    
    pub fn get_selected_story(&self) -> Option<&Story> {
        if self.workflow_states.is_empty() {
            return None;
        }
        
        let state_id = self.workflow_states[self.selected_column].0;
        self.stories_by_state.get(&state_id)
            .and_then(|stories| stories.get(self.selected_row))
    }

    pub fn toggle_state_selector(&mut self) {
        if !self.workflow_states.is_empty() {
            let state_id = self.workflow_states[self.selected_column].0;
            if let Some(stories) = self.stories_by_state.get(&state_id) {
                if !stories.is_empty() {
                    self.show_state_selector = true;
                    self.state_selector_index = 0;
                }
            }
        }
    }

    pub fn next_state_selection(&mut self) {
        if let Some(story) = self.get_selected_story() {
            let available_states = self.get_available_states_for_story(story);
            if !available_states.is_empty() {
                self.state_selector_index = (self.state_selector_index + 1) % available_states.len();
            }
        }
    }

    pub fn previous_state_selection(&mut self) {
        if let Some(story) = self.get_selected_story() {
            let available_states = self.get_available_states_for_story(story);
            if !available_states.is_empty() {
                if self.state_selector_index == 0 {
                    self.state_selector_index = available_states.len() - 1;
                } else {
                    self.state_selector_index -= 1;
                }
            }
        }
    }

    pub fn get_available_states_for_story(&self, story: &Story) -> Vec<(i64, String)> {
        self.workflow_states
            .iter()
            .filter(|(state_id, _)| *state_id != story.workflow_state_id)
            .cloned()
            .collect()
    }

    pub fn get_selected_target_state(&self) -> Option<i64> {
        if let Some(story) = self.get_selected_story() {
            let available_states = self.get_available_states_for_story(story);
            available_states.get(self.state_selector_index).map(|(id, _)| *id)
        } else {
            None
        }
    }

    pub fn handle_key_event(&mut self, key: event::KeyEvent) -> anyhow::Result<()> {
        if self.show_create_popup {
            // Handle create popup input
            match key.code {
                KeyCode::Esc => {
                    self.show_create_popup = false;
                    self.create_popup_state = CreatePopupState::default();
                }
                KeyCode::Tab => {
                    // Move to next field
                    self.create_popup_state.selected_field = match self.create_popup_state.selected_field {
                        CreateField::Name => CreateField::Description,
                        CreateField::Description => CreateField::Type,
                        CreateField::Type => CreateField::Name,
                    };
                }
                KeyCode::Enter => {
                    if self.create_popup_state.selected_field == CreateField::Type {
                        // Submit the story
                        if !self.create_popup_state.name.is_empty() {
                            self.create_story_requested = true;
                            self.show_create_popup = false;
                        }
                    } else {
                        // Move to next field on Enter
                        self.create_popup_state.selected_field = match self.create_popup_state.selected_field {
                            CreateField::Name => CreateField::Description,
                            CreateField::Description => CreateField::Type,
                            CreateField::Type => CreateField::Type,
                        };
                    }
                }
                KeyCode::Backspace => {
                    match self.create_popup_state.selected_field {
                        CreateField::Name => { self.create_popup_state.name.pop(); }
                        CreateField::Description => { self.create_popup_state.description.pop(); }
                        CreateField::Type => {}
                    }
                }
                KeyCode::Char(c) => {
                    match self.create_popup_state.selected_field {
                        CreateField::Name => self.create_popup_state.name.push(c),
                        CreateField::Description => self.create_popup_state.description.push(c),
                        CreateField::Type => {}
                    }
                }
                KeyCode::Up | KeyCode::Down if self.create_popup_state.selected_field == CreateField::Type => {
                    // Cycle through story types
                    let types = ["feature", "bug", "chore"];
                    if key.code == KeyCode::Down {
                        self.create_popup_state.story_type_index = 
                            (self.create_popup_state.story_type_index + 1) % types.len();
                    } else {
                        self.create_popup_state.story_type_index = 
                            if self.create_popup_state.story_type_index == 0 { 
                                types.len() - 1 
                            } else { 
                                self.create_popup_state.story_type_index - 1 
                            };
                    }
                    self.create_popup_state.story_type = types[self.create_popup_state.story_type_index].to_string();
                }
                _ => {}
            }
        } else if self.show_state_selector {
            // Handle state selector navigation
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.next_state_selection(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_state_selection(),
                KeyCode::Esc => {
                    self.show_state_selector = false;
                    self.state_selector_index = 0;
                }
                _ => {}
            }
        } else {
            // Normal navigation
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                // Handle detail view scrolling first (more specific patterns)
                KeyCode::Char('j') | KeyCode::Down if self.show_detail => {
                    // Simple scroll down - max scroll will be calculated in draw function
                    self.detail_scroll_offset += 1;
                }
                KeyCode::Char('k') | KeyCode::Up if self.show_detail => {
                    self.scroll_detail_up();
                }
                KeyCode::Esc if self.show_detail => {
                    self.show_detail = false;
                    self.detail_scroll_offset = 0;
                }
                // Regular navigation (less specific patterns)
                KeyCode::Char('j') | KeyCode::Down => self.next(),
                KeyCode::Char('k') | KeyCode::Up => self.previous(),
                KeyCode::Char('l') | KeyCode::Right => self.next_column(),
                KeyCode::Char('h') | KeyCode::Left => self.previous_column(),
                KeyCode::Enter => self.toggle_detail(),
                KeyCode::Char(' ') => self.toggle_state_selector(),
                KeyCode::Char('o') => {
                    if self.get_selected_story().is_some() {
                        self.take_ownership_requested = true;
                    }
                }
                KeyCode::Char('a') => {
                    self.show_create_popup = true;
                    self.create_popup_state = CreatePopupState::default();
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn get_owner_names(&self, owner_ids: &[String]) -> Vec<String> {
        owner_ids.iter()
            .map(|id| {
                
                self.member_cache.get(id)
                    .cloned()
                    .unwrap_or_else(|| {
                        // If debug mode, log cache miss
                        if std::env::var("RUST_LOG").is_ok() {
                            eprintln!("Cache miss for owner ID: {id}");
                        }
                        id.clone()
                    })
            })
            .collect()
    }

    pub fn add_member_to_cache(&mut self, member_id: String, member_name: String) {
        self.member_cache.insert(member_id, member_name);
    }

    pub fn set_current_user_id(&mut self, user_id: String) {
        self.current_user_id = Some(user_id);
    }

    pub fn scroll_detail_up(&mut self) {
        if self.detail_scroll_offset > 0 {
            self.detail_scroll_offset -= 1;
        }
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new("Shortcut Stories TUI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Create columns for workflow states
    if !app.workflow_states.is_empty() {
        let num_columns = app.workflow_states.len();
        let column_constraints: Vec<Constraint> = (0..num_columns)
            .map(|_| Constraint::Percentage((100 / num_columns) as u16))
            .collect();
        
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(column_constraints)
            .split(chunks[1]);
        
        // Render each workflow state column
        for (idx, (state_id, state_name)) in app.workflow_states.iter().enumerate() {
            let is_selected_column = idx == app.selected_column;
            
            // Get the actual column width
            let column_rect = columns[idx];
            // Account for borders (2) and some padding (2)
            let available_width = column_rect.width.saturating_sub(4) as usize;
            
            // Get stories for this state
            let stories = app.stories_by_state.get(state_id)
                .map(|s| s.as_slice())
                .unwrap_or(&[]);
            
            // Create list items
            let items: Vec<ListItem> = stories.iter().enumerate()
                .map(|(story_idx, story)| {
                    // Check if story is owned by current user
                    let is_owned = app.current_user_id.as_ref()
                        .map(|uid| story.owner_ids.contains(uid))
                        .unwrap_or(false);
                    
                    let style = if is_selected_column && story_idx == app.selected_row {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(if is_owned { Color::Cyan } else { Color::White })
                            .add_modifier(Modifier::BOLD)
                    } else if is_owned {
                        // Owned stories show in cyan
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    // Get icon for story type
                    let type_icon = match story.story_type.as_str() {
                        "feature" => "✨",
                        "bug" => "🐛",
                        "chore" => "🔧",
                        _ => "📝",
                    };
                    
                    // Create prefix for first line
                    let prefix = format!("[#{}] {} ", story.id, type_icon);
                    
                    // Calculate available width for text based on actual column width
                    let first_line_width = available_width.saturating_sub(prefix.len());
                    let second_line_width = available_width;
                    
                    // Handle story name wrapping
                    let mut line1_text = prefix.clone();
                    let mut line2_text = String::new();
                    
                    if story.name.len() <= first_line_width {
                        // Fits on one line
                        line1_text.push_str(&story.name);
                    } else {
                        // Try to wrap at word boundaries
                        let words: Vec<&str> = story.name.split_whitespace().collect();
                        
                        if !words.is_empty() {
                            // Check if even the first word fits
                            if words[0].len() > first_line_width {
                                // First word is too long, put entire name on second line
                                // But truncate if it's too long for the second line too
                                if story.name.len() > second_line_width {
                                    line2_text = story.name.chars().take(second_line_width.saturating_sub(3)).collect::<String>() + "...";
                                } else {
                                    line2_text = story.name.clone();
                                }
                            } else {
                                // Normal word wrapping
                                let mut current_length = 0;
                                let mut on_second_line = false;
                                
                                for (i, word) in words.iter().enumerate() {
                                    let word_len = word.len() + if i > 0 { 1 } else { 0 }; // +1 for space
                                    
                                    if !on_second_line && current_length + word_len <= first_line_width {
                                        if i > 0 {
                                            line1_text.push(' ');
                                        }
                                        line1_text.push_str(word);
                                        current_length += word_len;
                                    } else if !on_second_line {
                                        // Moving to second line
                                        on_second_line = true;
                                        if word_len <= second_line_width {
                                            line2_text.push_str(word);
                                            current_length = word_len;
                                        } else {
                                            // Word is too long for second line, truncate
                                            line2_text = word.chars().take(second_line_width.saturating_sub(3)).collect::<String>() + "...";
                                            break;
                                        }
                                    } else {
                                        // Already on second line
                                       if current_length + word_len < second_line_width {
                                            line2_text.push(' ');
                                            line2_text.push_str(word);
                                            current_length += word_len + 1;
                                        } else {
                                            // No more room, add ellipsis
                                            if line2_text.len() + 3 <= second_line_width {
                                                line2_text.push_str("...");
                                            } else {
                                                line2_text = line2_text.chars().take(second_line_width.saturating_sub(3)).collect::<String>() + "...";
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Create lines
                    let line1 = Line::from(Span::styled(line1_text, style));
                    let line2 = if line2_text.trim().is_empty() {
                        Line::from(Span::styled("", style))
                    } else {
                        Line::from(Span::styled(line2_text, style))
                    };
                    
                    let text = Text::from(vec![line1, line2]);
                    ListItem::new(text)
                })
                .collect();
            
            // Column title style
            let title_style = if is_selected_column {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            let title = format!(" {} ({}) ", state_name, stories.len());
            
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(title_style));
            
            frame.render_widget(list, columns[idx]);
        }
    } else {
        // No stories
        let empty = Paragraph::new("No stories found")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, chunks[1]);
    }

    // Footer
    let footer_text = if app.show_state_selector {
        "[↑/k] [↓/j] select state | [Enter] confirm | [Esc] cancel"
    } else if app.show_detail {
        "[↑/k] [↓/j] scroll | [Esc] close detail | [q] quit"
    } else {
        "[←/h] [→/l] columns | [↑/k] [↓/j] navigate | [Enter] details | [Space] move | [o] own | [a] add | [q] quit"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);

    // Detail popup
    if app.show_detail {
        if let Some(story) = app.get_selected_story() {
            draw_detail_popup(frame, story, app);
        }
    }

    // State selector popup
    if app.show_state_selector {
        if let Some(story) = app.get_selected_story() {
            draw_state_selector_popup(frame, story, app);
        }
    }
    
    // Create story popup
    if app.show_create_popup {
        draw_create_popup(frame, app);
    }
}

fn draw_detail_popup(frame: &mut Frame, story: &Story, app: &App) {
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);

    let workflow_state = app.workflow_state_map
        .get(&story.workflow_state_id)
        .map(|s| s.as_str())
        .unwrap_or("Unknown");

    let mut text_lines = vec![
        Line::from(vec![
            Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", story.id)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&story.name),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&story.story_type),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("State: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(workflow_state),
        ]),
        Line::from(""),
    ];

    // Add owners information
    if !story.owner_ids.is_empty() {
        let owner_names = app.get_owner_names(&story.owner_ids);
        text_lines.push(Line::from(vec![
            Span::styled("Owners: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(owner_names.join(", ")),
        ]));
    } else {
        text_lines.push(Line::from(vec![
            Span::styled("Owners: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Unassigned"),
        ]));
    }
    
    text_lines.push(Line::from(""));
    text_lines.push(Line::from(vec![
        Span::styled("Description:", Style::default().add_modifier(Modifier::BOLD)),
    ]));

    // Add description lines
    if !story.description.is_empty() {
        for line in story.description.lines() {
            text_lines.push(Line::from(line.to_string()));
        }
    } else {
        text_lines.push(Line::from("No description available"));
    }

    text_lines.push(Line::from(""));
    text_lines.push(Line::from(vec![
        Span::styled("URL: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(&story.app_url, Style::default().fg(Color::Cyan)),
    ]));

    // Add comments section
    if !story.comments.is_empty() {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(vec![
            Span::styled("Comments:", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        text_lines.push(Line::from(""));

        for comment in &story.comments {
            // Resolve author name from member cache
            let author_name = app.member_cache.get(&comment.author_id)
                .cloned()
                .unwrap_or_else(|| comment.author_id.clone());

            // Add author and timestamp
            text_lines.push(Line::from(vec![
                Span::styled(author_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::styled(comment.created_at.clone(), Style::default().fg(Color::DarkGray)),
            ]));

            // Add comment text with proper line wrapping
            for line in comment.text.lines() {
                text_lines.push(Line::from(format!("  {}", line)));
            }
            text_lines.push(Line::from(""));
        }
    }

    // Calculate scrollable content
    let total_lines = text_lines.len();
    let content_height = area.height.saturating_sub(2) as usize; // Account for borders
    let visible_lines = if total_lines > content_height {
        content_height
    } else {
        total_lines
    };

    // Apply scroll offset
    let start_line = app.detail_scroll_offset.min(total_lines.saturating_sub(visible_lines));
    let end_line = (start_line + visible_lines).min(total_lines);
    let visible_text_lines = if start_line < total_lines {
        text_lines[start_line..end_line].to_vec()
    } else {
        text_lines
    };

    // Create title with scroll indicator
    let scroll_indicator = if total_lines > content_height {
        format!(" Story Details ({}/{}) ", start_line + 1, total_lines.saturating_sub(content_height) + 1)
    } else {
        " Story Details ".to_string()
    };

    let paragraph = Paragraph::new(visible_text_lines)
        .block(
            Block::default()
                .title(scroll_indicator)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_state_selector_popup(frame: &mut Frame, story: &Story, app: &App) {
    let area = centered_rect(50, 40, frame.area());
    frame.render_widget(Clear, area);

    let available_states = app.get_available_states_for_story(story);
    
    // Create list items for available states
    let items: Vec<ListItem> = available_states
        .iter()
        .enumerate()
        .map(|(idx, (_, state_name))| {
            let content = format!(" {state_name} ");
            let style = if idx == app.state_selector_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Move Story #{} to: ", story.id);
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Yellow)));
    
    frame.render_widget(list, area);
}

fn draw_create_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);
    
    // Create the main popup block
    let popup = Block::default()
        .title("Create New Story")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));
    frame.render_widget(popup, area);
    
    // Create inner area for form fields
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    
    // Layout for form fields
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name field
            Constraint::Length(5), // Description field
            Constraint::Length(3), // Type field
            Constraint::Min(1),    // Space
            Constraint::Length(2), // Help text
        ])
        .split(inner);
    
    // Name field
    let name_style = if app.create_popup_state.selected_field == CreateField::Name {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let name_block = Block::default()
        .title("Name")
        .borders(Borders::ALL)
        .border_style(name_style);
    let name_text = Paragraph::new(app.create_popup_state.name.as_str())
        .block(name_block);
    frame.render_widget(name_text, chunks[0]);
    
    // Description field
    let desc_style = if app.create_popup_state.selected_field == CreateField::Description {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let desc_block = Block::default()
        .title("Description")
        .borders(Borders::ALL)
        .border_style(desc_style);
    let desc_text = Paragraph::new(app.create_popup_state.description.as_str())
        .block(desc_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(desc_text, chunks[1]);
    
    // Type field
    let type_style = if app.create_popup_state.selected_field == CreateField::Type {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let type_block = Block::default()
        .title("Type")
        .borders(Borders::ALL)
        .border_style(type_style);
    
    let type_text = if app.create_popup_state.selected_field == CreateField::Type {
        format!("< {} >", app.create_popup_state.story_type)
    } else {
        app.create_popup_state.story_type.clone()
    };
    
    let type_widget = Paragraph::new(type_text)
        .block(type_block)
        .alignment(Alignment::Center);
    frame.render_widget(type_widget, chunks[2]);
    
    // Help text
    let help_text = if app.create_popup_state.selected_field == CreateField::Type {
        "[↑/↓] change type | [Tab] next field | [Enter] submit | [Esc] cancel"
    } else {
        "[Tab] next field | [Enter] next/submit | [Esc] cancel"
    };
    
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[4]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}