use crate::api::{Story, Workflow};
use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
    pub workflow_state_map: HashMap<i64, String>,
    pub member_cache: HashMap<String, String>, // owner_id -> name
    pub current_user_id: Option<String>, // ID of current user
    pub should_quit: bool,
    pub selected_column: usize,
    pub selected_row: usize,
    pub stories_by_state: HashMap<i64, Vec<Story>>,
    pub workflow_states: Vec<(i64, String)>,
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
            workflow_state_map,
            member_cache: HashMap::new(),
            current_user_id: None,
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
        if self.show_state_selector {
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
                KeyCode::Esc if self.show_detail => self.show_detail = false,
                _ => {}
            }
        }
        Ok(())
    }

    pub fn get_owner_names(&self, owner_ids: &[String]) -> Vec<String> {
        owner_ids.iter()
            .map(|id| {
                let name = self.member_cache.get(id)
                    .cloned()
                    .unwrap_or_else(|| {
                        // If debug mode, log cache miss
                        if std::env::var("RUST_LOG").is_ok() {
                            eprintln!("Cache miss for owner ID: {}", id);
                        }
                        id.clone()
                    });
                name
            })
            .collect()
    }

    pub fn add_member_to_cache(&mut self, member_id: String, member_name: String) {
        self.member_cache.insert(member_id, member_name);
    }

    pub fn set_current_user_id(&mut self, user_id: String) {
        self.current_user_id = Some(user_id);
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
            
            // Get stories for this state
            let stories = app.stories_by_state.get(state_id)
                .map(|s| s.as_slice())
                .unwrap_or(&[]);
            
            // Create list items
            let items: Vec<ListItem> = stories.iter().enumerate()
                .map(|(story_idx, story)| {
                    let content = format!("[#{}] {}", story.id, story.name);
                    
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
                    ListItem::new(content).style(style)
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
        "Press [Esc] to close detail | [q] to quit"
    } else {
        "[←/h] [→/l] columns | [↑/k] [↓/j] navigate | [Enter] details | [Space] move | [o] own | [q] quit"
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

    let paragraph = Paragraph::new(text_lines)
        .block(
            Block::default()
                .title("Story Details")
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
            let content = format!(" {} ", state_name);
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