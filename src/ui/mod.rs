use crate::api::{Epic, Story, Workflow};
use crate::git::GitContext;
use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use crossterm::event::{self, KeyCode, MouseEventKind, MouseButton};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::collections::HashMap;
use tui_textarea::TextArea;

fn convert_key_to_ratatui(key: crossterm::event::KeyEvent) -> ratatui::crossterm::event::KeyEvent {
    let ratatui_code = match key.code {
        KeyCode::Backspace => ratatui::crossterm::event::KeyCode::Backspace,
        KeyCode::Enter => ratatui::crossterm::event::KeyCode::Enter,
        KeyCode::Left => ratatui::crossterm::event::KeyCode::Left,
        KeyCode::Right => ratatui::crossterm::event::KeyCode::Right,
        KeyCode::Up => ratatui::crossterm::event::KeyCode::Up,
        KeyCode::Down => ratatui::crossterm::event::KeyCode::Down,
        KeyCode::Home => ratatui::crossterm::event::KeyCode::Home,
        KeyCode::End => ratatui::crossterm::event::KeyCode::End,
        KeyCode::PageUp => ratatui::crossterm::event::KeyCode::PageUp,
        KeyCode::PageDown => ratatui::crossterm::event::KeyCode::PageDown,
        KeyCode::Tab => ratatui::crossterm::event::KeyCode::Tab,
        KeyCode::BackTab => ratatui::crossterm::event::KeyCode::BackTab,
        KeyCode::Delete => ratatui::crossterm::event::KeyCode::Delete,
        KeyCode::Insert => ratatui::crossterm::event::KeyCode::Insert,
        KeyCode::Esc => ratatui::crossterm::event::KeyCode::Esc,
        KeyCode::Char(c) => ratatui::crossterm::event::KeyCode::Char(c),
        KeyCode::F(n) => ratatui::crossterm::event::KeyCode::F(n),
        _ => ratatui::crossterm::event::KeyCode::Null,
    };

    ratatui::crossterm::event::KeyEvent::from(ratatui_code)
}

#[cfg(test)]
mod tests;

/// Helper function to determine if a date string is from the current week
fn is_current_week(date_str: &str) -> bool {
    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        let now = Utc::now();
        let date_utc = date.with_timezone(&Utc);

        // Get the start of the current week (Monday)
        let days_since_monday = match now.weekday() {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        };

        let week_start = now - Duration::days(days_since_monday);
        let week_start = week_start
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        // Get the end of the current week (Sunday)
        let week_end = week_start + Duration::days(7);

        date_utc >= week_start && date_utc < week_end
    } else {
        false
    }
}

/// Helper function to check if a workflow state is a "done" state
fn is_done_state(state_id: i64, workflows: &[Workflow]) -> bool {
    for workflow in workflows {
        for state in &workflow.states {
            if state.id == state_id {
                return state.state_type == "done";
            }
        }
    }
    false
}

pub struct App {
    pub show_detail: bool,
    pub show_state_selector: bool,
    pub state_selector_index: usize,
    pub take_ownership_requested: bool,
    pub create_story_requested: bool,
    pub show_create_popup: bool,
    pub create_popup_state: CreatePopupState,
    pub edit_story_requested: bool,
    pub show_edit_popup: bool,
    pub edit_popup_state: EditPopupState,
    pub workflow_state_map: HashMap<i64, String>,
    pub member_cache: HashMap<String, String>, // owner_id -> name
    pub current_user_id: Option<String>,       // ID of current user
    pub detail_scroll_offset: usize,           // Scroll offset for detail popup
    pub should_quit: bool,
    pub selected_column: usize,
    pub selected_row: usize,
    pub stories_by_state: HashMap<i64, Vec<Story>>,
    pub workflow_states: Vec<(i64, String)>,
    pub workflows: Vec<Workflow>, // Store workflows for filtering
    // List view mode
    pub list_view_mode: bool, // Toggle between column view and list view
    pub all_stories_list: Vec<Story>, // Flattened list of all stories for list view
    pub list_selected_index: usize, // Selected story index in list view
    pub list_scroll_offset: usize, // Scroll offset for list view
    // Pagination state
    pub search_query: String,            // Store the current search query
    pub next_page_token: Option<String>, // Token for the next page
    pub load_more_requested: bool,       // Flag to request loading more stories
    pub is_loading: bool,                // Flag to show loading state
    pub total_loaded_stories: usize,     // Count of total stories loaded
    // Git integration state
    pub git_context: GitContext,              // Git repository context
    pub show_git_popup: bool,                 // Flag to show git branch creation popup
    pub git_popup_state: GitBranchPopupState, // Git popup state
    pub git_branch_requested: bool,           // Flag to request git branch creation
    pub show_git_result_popup: bool,          // Flag to show git operation result popup
    pub git_result_state: GitResultState,     // Git result popup state
    // Refresh state
    pub refresh_requested: bool, // Flag to request refreshing all stories
    // Epic filtering state
    pub epics: Vec<Epic>,                   // List of available epics
    pub selected_epic_filter: Option<i64>,  // Selected epic ID to filter by
    pub show_epic_selector: bool,           // Flag to show epic selector popup
    pub epic_selector_index: usize,         // Selected index in epic selector
    pub all_stories_unfiltered: Vec<Story>, // Keep unfiltered stories for toggling
    // Help popup state
    pub show_help_popup: bool,      // Flag to show help popup
    pub help_selected_index: usize, // Selected command index in help popup
    // Create epic popup state
    pub show_create_epic_popup: bool,
    pub create_epic_popup_state: CreateEpicPopupState,
    pub create_epic_requested: bool,
    // URL tracking for clickable links
    pub clickable_urls: Vec<ClickableUrl>,   // URLs and their positions in the detail view
    pub detail_area: Option<Rect>,           // The area of the detail popup for coordinate calculation
}

#[derive(Clone)]
pub struct CreatePopupState {
    pub name_textarea: TextArea<'static>,
    pub description_textarea: TextArea<'static>,
    pub story_type: String,
    pub selected_field: CreateField,
    pub story_type_index: usize,
    pub epic_id: Option<i64>,
    pub epic_selector_index: usize, // 0 = None, 1+ = epic index
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateField {
    Name,
    Description,
    Type,
    Epic,
}

#[derive(Clone)]
pub struct EditPopupState {
    pub name_textarea: TextArea<'static>,
    pub description_textarea: TextArea<'static>,
    pub story_type: String,
    pub selected_field: EditField,
    pub story_type_index: usize,
    pub story_id: i64,
    pub epic_id: Option<i64>,
    pub epic_selector_index: usize, // 0 = None, 1+ = epic index
}

#[derive(Debug, Clone)]
pub struct ClickableUrl {
    pub url: String,
    pub row: u16,     // Row in the detail popup (0-based)
    pub start_col: u16, // Starting column of the URL
    pub end_col: u16,   // Ending column of the URL
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditField {
    Name,
    Description,
    Type,
    Epic,
}

#[derive(Clone)]
pub struct CreateEpicPopupState {
    pub name_textarea: TextArea<'static>,
    pub description_textarea: TextArea<'static>,
    pub selected_field: CreateEpicField,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateEpicField {
    Name,
    Description,
}

#[derive(Clone)]
pub struct GitBranchPopupState {
    pub branch_name_textarea: TextArea<'static>,
    pub worktree_path_textarea: TextArea<'static>,
    pub selected_option: GitBranchOption,
    pub story_id: i64,
    pub editing_branch_name: bool,
    pub editing_worktree_path: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitBranchOption {
    CreateBranch,
    CreateWorktree,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct GitResultState {
    pub success: bool,
    #[allow(dead_code)]
    pub operation_type: GitOperationType,
    pub message: String,
    #[allow(dead_code)]
    pub branch_name: String,
    pub worktree_path: Option<String>,
    #[allow(dead_code)]
    pub story_id: i64,
    pub selected_option: GitResultOption,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitOperationType {
    CreateBranch,
    CreateWorktree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitResultOption {
    Continue,
    ExitAndChange, // Only for successful worktree creation
}

impl Default for CreatePopupState {
    fn default() -> Self {
        let mut name_textarea = TextArea::default();
        name_textarea.set_cursor_line_style(Style::default());
        name_textarea.set_block(Block::default().borders(Borders::ALL).title("Name"));

        let mut description_textarea = TextArea::default();
        description_textarea.set_cursor_line_style(Style::default());
        description_textarea.set_block(Block::default().borders(Borders::ALL).title("Description"));

        Self {
            name_textarea,
            description_textarea,
            story_type: "feature".to_string(),
            selected_field: CreateField::Name,
            story_type_index: 0,
            epic_id: None,
            epic_selector_index: 0,
        }
    }
}

impl EditPopupState {
    pub fn from_story(story: &Story) -> Self {
        let story_type_index = match story.story_type.as_str() {
            "feature" => 0,
            "bug" => 1,
            "chore" => 2,
            _ => 0,
        };

        let mut name_textarea = TextArea::default();
        name_textarea.set_cursor_line_style(Style::default());
        name_textarea.set_block(Block::default().borders(Borders::ALL).title("Name"));
        name_textarea.insert_str(&story.name);

        let mut description_textarea = TextArea::default();
        description_textarea.set_cursor_line_style(Style::default());
        description_textarea.set_block(Block::default().borders(Borders::ALL).title("Description"));
        description_textarea.insert_str(&story.description);

        Self {
            name_textarea,
            description_textarea,
            story_type: story.story_type.clone(),
            selected_field: EditField::Name,
            story_type_index,
            story_id: story.id,
            epic_id: story.epic_id,
            epic_selector_index: 0, // Will be set when popup is opened
        }
    }
}

impl App {
    pub fn new(
        stories: Vec<Story>,
        workflows: Vec<Workflow>,
        search_query: String,
        next_page_token: Option<String>,
    ) -> Self {
        // Filter stories before grouping by state
        let filtered_stories = stories
            .into_iter()
            .filter(|story| {
                if is_done_state(story.workflow_state_id, &workflows) {
                    // For Done states, only keep stories completed in the current week
                    if let Some(completed_at) = &story.completed_at {
                        return is_current_week(completed_at);
                    } else if let Some(moved_at) = &story.moved_at {
                        // Fall back to moved_at if completed_at is not available
                        return is_current_week(moved_at);
                    } else {
                        // If no completion date available, use updated_at as fallback
                        return is_current_week(&story.updated_at);
                    }
                }
                // Keep all non-Done stories
                true
            })
            .collect::<Vec<_>>();

        // Group stories by workflow state
        let mut stories_by_state: HashMap<i64, Vec<Story>> = HashMap::new();
        for story in filtered_stories.iter() {
            stories_by_state
                .entry(story.workflow_state_id)
                .or_default()
                .push(story.clone());
        }

        // Sort stories within each state by position
        for stories in stories_by_state.values_mut() {
            stories.sort_by_key(|s| s.position);
        }

        // Limit Done states to 10 stories maximum
        for (&state_id, stories) in stories_by_state.iter_mut() {
            if is_done_state(state_id, &workflows) && stories.len() > 10 {
                stories.truncate(10);
            }
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
        workflow_states.sort_by_key(|(id, _)| state_positions.get(id).copied().unwrap_or(i64::MAX));

        // Find the first column (workflow state) that contains stories
        let mut selected_column = 0;
        for (index, (state_id, _)) in workflow_states.iter().enumerate() {
            if let Some(stories) = stories_by_state.get(state_id)
                && !stories.is_empty()
            {
                selected_column = index;
                break;
            }
        }

        let total_stories = filtered_stories.len();

        // Create a flattened list of all stories for list view, sorted by position
        let mut all_stories_list = filtered_stories.clone();
        all_stories_list.sort_by_key(|s| s.position);

        // Keep unfiltered stories for epic filtering
        let all_stories_unfiltered = filtered_stories.clone();

        let git_context = GitContext::detect().unwrap_or(GitContext {
            repo_type: crate::git::GitRepoType::NotARepo,
            current_branch: None,
        });

        Self {
            show_detail: false,
            show_state_selector: false,
            state_selector_index: 0,
            take_ownership_requested: false,
            create_story_requested: false,
            show_create_popup: false,
            create_popup_state: CreatePopupState::default(),
            edit_story_requested: false,
            show_edit_popup: false,
            edit_popup_state: EditPopupState {
                name_textarea: {
                    let mut textarea = TextArea::default();
                    textarea.set_cursor_line_style(Style::default());
                    textarea.set_block(Block::default().borders(Borders::ALL).title("Name"));
                    textarea
                },
                description_textarea: {
                    let mut textarea = TextArea::default();
                    textarea.set_cursor_line_style(Style::default());
                    textarea.set_block(Block::default().borders(Borders::ALL).title("Description"));
                    textarea
                },
                story_type: "feature".to_string(),
                selected_field: EditField::Name,
                story_type_index: 0,
                story_id: 0,
                epic_id: None,
                epic_selector_index: 0,
            },
            workflow_state_map,
            member_cache: HashMap::new(),
            current_user_id: None,
            detail_scroll_offset: 0,
            should_quit: false,
            selected_column,
            selected_row: 0,
            stories_by_state,
            workflow_states,
            workflows,
            list_view_mode: false,
            all_stories_list,
            list_selected_index: 0,
            list_scroll_offset: 0,
            search_query,
            next_page_token,
            load_more_requested: false,
            is_loading: false,
            total_loaded_stories: total_stories,
            git_context,
            show_git_popup: false,
            git_popup_state: GitBranchPopupState {
                branch_name_textarea: {
                    let mut textarea = TextArea::default();
                    textarea.set_cursor_line_style(Style::default());
                    textarea.set_block(Block::default().borders(Borders::ALL).title("Branch Name"));
                    textarea
                },
                worktree_path_textarea: {
                    let mut textarea = TextArea::default();
                    textarea.set_cursor_line_style(Style::default());
                    textarea.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Worktree Path"),
                    );
                    textarea
                },
                selected_option: GitBranchOption::CreateBranch,
                story_id: 0,
                editing_branch_name: false,
                editing_worktree_path: false,
            },
            git_branch_requested: false,
            show_git_result_popup: false,
            git_result_state: GitResultState {
                success: false,
                operation_type: GitOperationType::CreateBranch,
                message: String::new(),
                branch_name: String::new(),
                worktree_path: None,
                story_id: 0,
                selected_option: GitResultOption::Continue,
            },
            refresh_requested: false,
            epics: Vec::new(),
            selected_epic_filter: None,
            show_epic_selector: false,
            epic_selector_index: 0,
            all_stories_unfiltered,
            show_help_popup: false,
            help_selected_index: 0,
            show_create_epic_popup: false,
            create_epic_popup_state: CreateEpicPopupState {
                name_textarea: {
                    let mut textarea = TextArea::default();
                    textarea.set_cursor_line_style(Style::default());
                    textarea.set_block(Block::default().borders(Borders::ALL).title("Epic Name"));
                    textarea
                },
                description_textarea: {
                    let mut textarea = TextArea::default();
                    textarea.set_cursor_line_style(Style::default());
                    textarea.set_block(Block::default().borders(Borders::ALL).title("Description"));
                    textarea
                },
                selected_field: CreateEpicField::Name,
            },
            create_epic_requested: false,
            clickable_urls: Vec::new(),
            detail_area: None,
        }
    }

    pub fn toggle_view_mode(&mut self) {
        self.list_view_mode = !self.list_view_mode;
        // Reset selections when switching modes
        if self.list_view_mode {
            self.list_selected_index = 0;
            self.list_scroll_offset = 0;
        } else {
            self.selected_column = 0;
            self.selected_row = 0;
        }
    }

    pub fn update_list_scroll(&mut self, visible_height: usize) {
        if !self.list_view_mode || self.all_stories_list.is_empty() {
            return;
        }

        // Each story takes 2 lines (title + optional wrapped name)
        let visible_stories = visible_height / 2;
        if visible_stories == 0 {
            return;
        }

        // Ensure selected item is visible
        if self.list_selected_index < self.list_scroll_offset {
            // Selected item is above visible area, scroll up
            self.list_scroll_offset = self.list_selected_index;
        } else if self.list_selected_index >= self.list_scroll_offset + visible_stories {
            // Selected item is below visible area, scroll down
            self.list_scroll_offset = self.list_selected_index.saturating_sub(visible_stories - 1);
        }

        // Ensure we don't scroll past the end
        let max_scroll = self.all_stories_list.len().saturating_sub(visible_stories);
        if self.list_scroll_offset > max_scroll {
            self.list_scroll_offset = max_scroll;
        }
    }

    pub fn next(&mut self) {
        if self.list_view_mode {
            // List view navigation
            if !self.all_stories_list.is_empty() {
                self.list_selected_index =
                    (self.list_selected_index + 1) % self.all_stories_list.len();
                // Scroll will be updated in the draw function based on visible area
            }
        } else {
            // Column view navigation
            if self.workflow_states.is_empty() {
                return;
            }

            let state_id = self.workflow_states[self.selected_column].0;
            if let Some(stories) = self.stories_by_state.get(&state_id)
                && !stories.is_empty()
            {
                self.selected_row = (self.selected_row + 1) % stories.len();
            }
        }
    }

    pub fn previous(&mut self) {
        if self.list_view_mode {
            // List view navigation
            if !self.all_stories_list.is_empty() {
                if self.list_selected_index == 0 {
                    self.list_selected_index = self.all_stories_list.len() - 1;
                } else {
                    self.list_selected_index -= 1;
                }
                // Scroll will be updated in the draw function based on visible area
            }
        } else {
            // Column view navigation
            if self.workflow_states.is_empty() {
                return;
            }

            let state_id = self.workflow_states[self.selected_column].0;
            if let Some(stories) = self.stories_by_state.get(&state_id)
                && !stories.is_empty()
            {
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
            if let Some(stories) = self.stories_by_state.get(&state_id)
                && !stories.is_empty()
            {
                self.show_detail = !self.show_detail;
                // Reset scroll offset when opening detail view
                if self.show_detail {
                    self.detail_scroll_offset = 0;
                }
            }
        }
    }

    pub fn get_selected_story(&self) -> Option<&Story> {
        if self.list_view_mode {
            // List view mode
            self.all_stories_list.get(self.list_selected_index)
        } else {
            // Column view mode
            if self.workflow_states.is_empty() {
                return None;
            }

            let state_id = self.workflow_states[self.selected_column].0;
            self.stories_by_state
                .get(&state_id)
                .and_then(|stories| stories.get(self.selected_row))
        }
    }

    pub fn toggle_state_selector(&mut self) {
        if !self.workflow_states.is_empty() {
            let state_id = self.workflow_states[self.selected_column].0;
            if let Some(stories) = self.stories_by_state.get(&state_id)
                && !stories.is_empty()
            {
                self.show_state_selector = true;
                self.state_selector_index = 0;
            }
        }
    }

    pub fn next_state_selection(&mut self) {
        if let Some(story) = self.get_selected_story() {
            let available_states = self.get_available_states_for_story(story);
            if !available_states.is_empty() {
                self.state_selector_index =
                    (self.state_selector_index + 1) % available_states.len();
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
            available_states
                .get(self.state_selector_index)
                .map(|(id, _)| *id)
        } else {
            None
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) -> anyhow::Result<()> {
        // Only handle clicks in the detail popup
        if !self.show_detail || self.detail_area.is_none() {
            return Ok(());
        }

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let area = self.detail_area.unwrap();

            // Check if click is within the detail popup area
            if mouse.column >= area.x && mouse.column < area.x + area.width
                && mouse.row >= area.y && mouse.row < area.y + area.height {

                // Calculate relative position within the popup
                let relative_row = mouse.row - area.y;

                // Check if we clicked on any URL
                for clickable_url in &self.clickable_urls {
                    if clickable_url.row == relative_row
                        && mouse.column >= area.x + clickable_url.start_col
                        && mouse.column <= area.x + clickable_url.end_col {

                        // Open the URL in the default browser
                        if let Err(e) = open::that(&clickable_url.url) {
                            eprintln!("Failed to open URL: {}", e);
                        }
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn handle_key_event(&mut self, key: event::KeyEvent) -> anyhow::Result<()> {
        if self.show_help_popup {
            // Handle help popup input
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.show_help_popup = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.help_selected_index > 0 {
                        self.help_selected_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Total commands: Navigation(4) + View(5) + Story Actions(6) + Application(2) = 17
                    let total_commands = 17;
                    if self.help_selected_index < total_commands - 1 {
                        self.help_selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    // Execute the selected command
                    self.show_help_popup = false;

                    // Map index to command
                    // Navigation: 0-3, View: 4-8, Story Actions: 9-14, Application: 15-16
                    match self.help_selected_index {
                        // Navigation
                        0 => {} // Up - no action, just informational
                        1 => {} // Down - no action, just informational
                        2 => {} // Left - no action, just informational
                        3 => {} // Right - no action, just informational
                        // View
                        4 => {
                            // Enter - Show story details
                            if !self.show_detail && self.get_selected_story().is_some() {
                                self.toggle_detail();
                            }
                        }
                        5 => self.toggle_view_mode(), // v - Toggle view
                        6 => self.toggle_epic_selector(), // f - Filter by epic
                        7 => self.refresh_stories(),  // r - Refresh
                        8 => {
                            // n - Load more stories
                            if self.has_more_stories() {
                                self.request_load_more();
                            }
                        }
                        // Story Actions
                        9 => {
                            // Space - Move story
                            if self.get_selected_story().is_some() {
                                self.toggle_state_selector();
                            }
                        }
                        10 => self.take_ownership_requested = true, // o - Take ownership
                        11 => {
                            // e - Edit story
                            if let Some(story) = self.get_selected_story().cloned() {
                                self.show_edit_popup = true;
                                self.edit_popup_state = EditPopupState::from_story(&story);
                            }
                        }
                        12 => {
                            // a - Add story
                            self.show_create_popup = true;
                            self.create_popup_state = CreatePopupState::default();
                        }
                        13 => {
                            // E - Create epic
                            self.show_create_epic_popup = true;
                            self.create_epic_popup_state.name_textarea.delete_line_by_head();
                            self.create_epic_popup_state.name_textarea.delete_line_by_end();
                            self.create_epic_popup_state.description_textarea.delete_line_by_head();
                            self.create_epic_popup_state.description_textarea.delete_line_by_end();
                            self.create_epic_popup_state.selected_field = CreateEpicField::Name;
                        }
                        14 => {
                            // g - Create git branch
                            if self.git_context.is_git_repo()
                                && let Some(story) = self.get_selected_story().cloned()
                            {
                                let suggested_branch =
                                    story.formatted_vcs_branch_name.unwrap_or_else(|| {
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
                                self.show_git_popup = true;
                                self.git_popup_state = GitBranchPopupState {
                                    branch_name_textarea: {
                                        let mut textarea = TextArea::default();
                                        textarea.set_cursor_line_style(Style::default());
                                        textarea.set_block(
                                            Block::default()
                                                .borders(Borders::ALL)
                                                .title("Branch Name"),
                                        );
                                        textarea.insert_str(&suggested_branch);
                                        textarea
                                    },
                                    worktree_path_textarea: {
                                        let mut textarea = TextArea::default();
                                        textarea.set_cursor_line_style(Style::default());
                                        textarea.set_block(
                                            Block::default()
                                                .borders(Borders::ALL)
                                                .title("Worktree Path"),
                                        );
                                        textarea.insert_str(crate::git::generate_worktree_path(
                                            &suggested_branch,
                                        ));
                                        textarea
                                    },
                                    selected_option: if self.git_context.is_bare_repo() {
                                        GitBranchOption::CreateWorktree
                                    } else {
                                        GitBranchOption::CreateBranch
                                    },
                                    story_id: story.id,
                                    editing_branch_name: false,
                                    editing_worktree_path: false,
                                };
                            }
                        }
                        // Application
                        15 => {}                       // ? - Help (already closed)
                        16 => self.should_quit = true, // q - Quit
                        _ => {}
                    }
                }
                _ => {}
            }
        } else if self.show_git_result_popup {
            // Handle git result popup input
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    if self.git_result_state.selected_option == GitResultOption::Continue
                        || key.code == KeyCode::Esc
                    {
                        // Just close the popup
                        self.show_git_result_popup = false;
                    } else if self.git_result_state.selected_option
                        == GitResultOption::ExitAndChange
                    {
                        // Exit and change to worktree directory
                        if let Some(ref worktree_path) = self.git_result_state.worktree_path {
                            // Set flag to exit the application and change directory
                            unsafe {
                                std::env::set_var("SC_CLI_EXIT_AND_CD", worktree_path);
                            }
                        }
                        self.should_quit = true;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.git_result_state.worktree_path.is_some()
                        && self.git_result_state.success
                    {
                        // Toggle between Continue and ExitAndChange
                        self.git_result_state.selected_option =
                            match self.git_result_state.selected_option {
                                GitResultOption::Continue => GitResultOption::ExitAndChange,
                                GitResultOption::ExitAndChange => GitResultOption::Continue,
                            };
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.git_result_state.worktree_path.is_some()
                        && self.git_result_state.success
                    {
                        // Toggle between Continue and ExitAndChange
                        self.git_result_state.selected_option =
                            match self.git_result_state.selected_option {
                                GitResultOption::Continue => GitResultOption::ExitAndChange,
                                GitResultOption::ExitAndChange => GitResultOption::Continue,
                            };
                    }
                }
                _ => {}
            }
        } else if self.show_git_popup {
            // Handle git popup input
            if self.git_popup_state.editing_branch_name {
                // Handle branch name editing with TextArea
                match key.code {
                    KeyCode::Esc => {
                        self.git_popup_state.editing_branch_name = false;
                    }
                    KeyCode::Enter => {
                        self.git_popup_state.editing_branch_name = false;
                        // Update worktree path when branch name changes
                        let branch_name =
                            self.git_popup_state.branch_name_textarea.lines().join("");
                        let worktree_path = crate::git::generate_worktree_path(&branch_name);
                        self.git_popup_state
                            .worktree_path_textarea
                            .delete_line_by_head();
                        self.git_popup_state
                            .worktree_path_textarea
                            .insert_str(&worktree_path);
                    }
                    _ => {
                        self.git_popup_state
                            .branch_name_textarea
                            .input(convert_key_to_ratatui(key));
                    }
                }
            } else if self.git_popup_state.editing_worktree_path {
                // Handle worktree path editing with TextArea
                match key.code {
                    KeyCode::Esc => {
                        self.git_popup_state.editing_worktree_path = false;
                    }
                    KeyCode::Enter => {
                        self.git_popup_state.editing_worktree_path = false;
                    }
                    _ => {
                        self.git_popup_state
                            .worktree_path_textarea
                            .input(convert_key_to_ratatui(key));
                    }
                }
            } else {
                // Handle normal git popup navigation
                match key.code {
                    KeyCode::Esc => {
                        self.show_git_popup = false;
                        self.git_popup_state = GitBranchPopupState {
                            branch_name_textarea: {
                                let mut textarea = TextArea::default();
                                textarea.set_cursor_line_style(Style::default());
                                textarea.set_block(
                                    Block::default().borders(Borders::ALL).title("Branch Name"),
                                );
                                textarea
                            },
                            worktree_path_textarea: {
                                let mut textarea = TextArea::default();
                                textarea.set_cursor_line_style(Style::default());
                                textarea.set_block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title("Worktree Path"),
                                );
                                textarea
                            },
                            selected_option: GitBranchOption::CreateBranch,
                            story_id: 0,
                            editing_branch_name: false,
                            editing_worktree_path: false,
                        };
                    }
                    KeyCode::Enter => match self.git_popup_state.selected_option {
                        GitBranchOption::CreateBranch | GitBranchOption::CreateWorktree => {
                            self.git_branch_requested = true;
                            self.show_git_popup = false;
                        }
                        GitBranchOption::Cancel => {
                            self.show_git_popup = false;
                        }
                    },
                    KeyCode::Tab | KeyCode::Char('e') => {
                        // Enter branch name editing mode
                        self.git_popup_state.editing_branch_name = true;
                    }
                    KeyCode::Char('w') => {
                        // Enter worktree path editing mode (only for bare repos)
                        if self.git_context.is_bare_repo() {
                            self.git_popup_state.editing_worktree_path = true;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        match self.git_popup_state.selected_option {
                            GitBranchOption::CreateBranch => {
                                self.git_popup_state.selected_option = GitBranchOption::Cancel;
                            }
                            GitBranchOption::CreateWorktree => {
                                // CreateWorktree is only available in bare repos, so always go to Cancel
                                self.git_popup_state.selected_option = GitBranchOption::Cancel;
                            }
                            GitBranchOption::Cancel => {
                                if self.git_context.is_bare_repo() {
                                    self.git_popup_state.selected_option =
                                        GitBranchOption::CreateWorktree;
                                } else {
                                    self.git_popup_state.selected_option =
                                        GitBranchOption::CreateBranch;
                                }
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        match self.git_popup_state.selected_option {
                            GitBranchOption::CreateBranch => {
                                if self.git_context.is_bare_repo() {
                                    self.git_popup_state.selected_option =
                                        GitBranchOption::CreateWorktree;
                                } else {
                                    self.git_popup_state.selected_option = GitBranchOption::Cancel;
                                }
                            }
                            GitBranchOption::CreateWorktree => {
                                self.git_popup_state.selected_option = GitBranchOption::Cancel;
                            }
                            GitBranchOption::Cancel => {
                                if self.git_context.is_bare_repo() {
                                    self.git_popup_state.selected_option =
                                        GitBranchOption::CreateWorktree;
                                } else {
                                    self.git_popup_state.selected_option =
                                        GitBranchOption::CreateBranch;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else if self.show_epic_selector {
            // Handle epic selector navigation
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.next_epic_selection(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_epic_selection(),
                KeyCode::Enter => self.apply_selected_epic_filter(),
                KeyCode::Esc => {
                    self.show_epic_selector = false;
                    self.epic_selector_index = 0;
                }
                _ => {}
            }
        } else if self.show_create_epic_popup {
            // Handle create epic popup input
            match key.code {
                KeyCode::Esc => {
                    self.show_create_epic_popup = false;
                    self.create_epic_popup_state = CreateEpicPopupState {
                        name_textarea: {
                            let mut textarea = TextArea::default();
                            textarea.set_cursor_line_style(Style::default());
                            textarea.set_block(Block::default().borders(Borders::ALL).title("Epic Name"));
                            textarea
                        },
                        description_textarea: {
                            let mut textarea = TextArea::default();
                            textarea.set_cursor_line_style(Style::default());
                            textarea.set_block(Block::default().borders(Borders::ALL).title("Description"));
                            textarea
                        },
                        selected_field: CreateEpicField::Name,
                    };
                }
                KeyCode::Tab => {
                    // Toggle between name and description fields
                    self.create_epic_popup_state.selected_field = match self.create_epic_popup_state.selected_field {
                        CreateEpicField::Name => CreateEpicField::Description,
                        CreateEpicField::Description => CreateEpicField::Name,
                    };
                }
                KeyCode::Enter => {
                    if self.create_epic_popup_state.selected_field == CreateEpicField::Description {
                        // Submit the epic when Enter is pressed on Description field
                        if !self.create_epic_popup_state.name_textarea.lines().join("").trim().is_empty() {
                            self.create_epic_requested = true;
                            self.show_create_epic_popup = false;
                        }
                    } else {
                        // Move to next field when Enter is pressed on Name field
                        self.create_epic_popup_state.selected_field = CreateEpicField::Description;
                    }
                }
                _ => {
                    // Handle text input
                    match self.create_epic_popup_state.selected_field {
                        CreateEpicField::Name => {
                            self.create_epic_popup_state.name_textarea.input(convert_key_to_ratatui(key));
                        }
                        CreateEpicField::Description => {
                            self.create_epic_popup_state.description_textarea.input(convert_key_to_ratatui(key));
                        }
                    }
                }
            }
        } else if self.show_edit_popup {
            // Handle edit popup input
            match key.code {
                KeyCode::Esc => {
                    self.show_edit_popup = false;
                    self.edit_popup_state = EditPopupState {
                        name_textarea: {
                            let mut textarea = TextArea::default();
                            textarea.set_cursor_line_style(Style::default());
                            textarea
                                .set_block(Block::default().borders(Borders::ALL).title("Name"));
                            textarea
                        },
                        description_textarea: {
                            let mut textarea = TextArea::default();
                            textarea.set_cursor_line_style(Style::default());
                            textarea.set_block(
                                Block::default().borders(Borders::ALL).title("Description"),
                            );
                            textarea
                        },
                        story_type: "feature".to_string(),
                        selected_field: EditField::Name,
                        story_type_index: 0,
                        story_id: 0,
                        epic_id: None,
                        epic_selector_index: 0,
                    };
                }
                KeyCode::Tab => {
                    // Move to next field
                    self.edit_popup_state.selected_field =
                        match self.edit_popup_state.selected_field {
                            EditField::Name => EditField::Description,
                            EditField::Description => EditField::Type,
                            EditField::Type => EditField::Epic,
                            EditField::Epic => EditField::Name,
                        };
                }
                KeyCode::Enter => {
                    if self.edit_popup_state.selected_field == EditField::Epic {
                        // Submit the story edit
                        if !self
                            .edit_popup_state
                            .name_textarea
                            .lines()
                            .join("")
                            .trim()
                            .is_empty()
                        {
                            self.edit_story_requested = true;
                            self.show_edit_popup = false;
                        }
                    } else {
                        // Move to next field on Enter
                        self.edit_popup_state.selected_field =
                            match self.edit_popup_state.selected_field {
                                EditField::Name => EditField::Description,
                                EditField::Description => EditField::Type,
                                EditField::Type => EditField::Epic,
                                EditField::Epic => EditField::Epic,
                            };
                    }
                }
                KeyCode::Up | KeyCode::Down
                    if self.edit_popup_state.selected_field == EditField::Type =>
                {
                    // Cycle through story types
                    let types = ["feature", "bug", "chore"];
                    if key.code == KeyCode::Down {
                        self.edit_popup_state.story_type_index =
                            (self.edit_popup_state.story_type_index + 1) % types.len();
                    } else {
                        self.edit_popup_state.story_type_index =
                            if self.edit_popup_state.story_type_index == 0 {
                                types.len() - 1
                            } else {
                                self.edit_popup_state.story_type_index - 1
                            };
                    }
                    self.edit_popup_state.story_type =
                        types[self.edit_popup_state.story_type_index].to_string();
                }
                KeyCode::Up | KeyCode::Down
                    if self.edit_popup_state.selected_field == EditField::Epic =>
                {
                    // Cycle through epics (including None option)
                    let epic_count = self.epics.len() + 1; // +1 for None option
                    if key.code == KeyCode::Down {
                        self.edit_popup_state.epic_selector_index =
                            (self.edit_popup_state.epic_selector_index + 1) % epic_count;
                    } else {
                        self.edit_popup_state.epic_selector_index =
                            if self.edit_popup_state.epic_selector_index == 0 {
                                epic_count - 1
                            } else {
                                self.edit_popup_state.epic_selector_index - 1
                            };
                    }
                    // Update epic_id based on selection
                    self.edit_popup_state.epic_id =
                        if self.edit_popup_state.epic_selector_index == 0 {
                            None
                        } else if self.edit_popup_state.epic_selector_index <= self.epics.len() {
                            Some(self.epics[self.edit_popup_state.epic_selector_index - 1].id)
                        } else {
                            None
                        };
                }
                _ => {
                    // Handle text input for TextArea widgets
                    match self.edit_popup_state.selected_field {
                        EditField::Name => {
                            self.edit_popup_state
                                .name_textarea
                                .input(convert_key_to_ratatui(key));
                        }
                        EditField::Description => {
                            self.edit_popup_state
                                .description_textarea
                                .input(convert_key_to_ratatui(key));
                        }
                        EditField::Type => {}
                        EditField::Epic => {}
                    }
                }
            }
        } else if self.show_create_popup {
            // Handle create popup input
            match key.code {
                KeyCode::Esc => {
                    self.show_create_popup = false;
                    self.create_popup_state = CreatePopupState::default();
                }
                KeyCode::Tab => {
                    // Move to next field
                    self.create_popup_state.selected_field =
                        match self.create_popup_state.selected_field {
                            CreateField::Name => CreateField::Description,
                            CreateField::Description => CreateField::Type,
                            CreateField::Type => CreateField::Epic,
                            CreateField::Epic => CreateField::Name,
                        };
                }
                KeyCode::Enter => {
                    if self.create_popup_state.selected_field == CreateField::Epic {
                        // Submit the story
                        if !self
                            .create_popup_state
                            .name_textarea
                            .lines()
                            .join("")
                            .trim()
                            .is_empty()
                        {
                            self.create_story_requested = true;
                            self.show_create_popup = false;
                        }
                    } else {
                        // Move to next field on Enter
                        self.create_popup_state.selected_field =
                            match self.create_popup_state.selected_field {
                                CreateField::Name => CreateField::Description,
                                CreateField::Description => CreateField::Type,
                                CreateField::Type => CreateField::Epic,
                                CreateField::Epic => CreateField::Epic,
                            };
                    }
                }
                KeyCode::Up | KeyCode::Down
                    if self.create_popup_state.selected_field == CreateField::Type =>
                {
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
                    self.create_popup_state.story_type =
                        types[self.create_popup_state.story_type_index].to_string();
                }
                KeyCode::Up | KeyCode::Down
                    if self.create_popup_state.selected_field == CreateField::Epic =>
                {
                    // Cycle through epics (including None option)
                    let epic_count = self.epics.len() + 1; // +1 for None option
                    if key.code == KeyCode::Down {
                        self.create_popup_state.epic_selector_index =
                            (self.create_popup_state.epic_selector_index + 1) % epic_count;
                    } else {
                        self.create_popup_state.epic_selector_index =
                            if self.create_popup_state.epic_selector_index == 0 {
                                epic_count - 1
                            } else {
                                self.create_popup_state.epic_selector_index - 1
                            };
                    }
                    // Update epic_id based on selection
                    self.create_popup_state.epic_id =
                        if self.create_popup_state.epic_selector_index == 0 {
                            None
                        } else if self.create_popup_state.epic_selector_index <= self.epics.len() {
                            Some(self.epics[self.create_popup_state.epic_selector_index - 1].id)
                        } else {
                            None
                        };
                }
                _ => {
                    // Handle text input for TextArea widgets
                    match self.create_popup_state.selected_field {
                        CreateField::Name => {
                            self.create_popup_state
                                .name_textarea
                                .input(convert_key_to_ratatui(key));
                        }
                        CreateField::Description => {
                            self.create_popup_state
                                .description_textarea
                                .input(convert_key_to_ratatui(key));
                        }
                        CreateField::Type => {}
                        CreateField::Epic => {}
                    }
                }
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
                KeyCode::Char('l') | KeyCode::Right => {
                    if !self.list_view_mode {
                        self.next_column();
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if !self.list_view_mode {
                        self.previous_column();
                    }
                }
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
                KeyCode::Char('E') => {
                    // Shift+E to create epic
                    self.show_create_epic_popup = true;
                    self.create_epic_popup_state.name_textarea.delete_line_by_head();
                    self.create_epic_popup_state.name_textarea.delete_line_by_end();
                    self.create_epic_popup_state.description_textarea.delete_line_by_head();
                    self.create_epic_popup_state.description_textarea.delete_line_by_end();
                    self.create_epic_popup_state.selected_field = CreateEpicField::Name;
                }
                KeyCode::Char('e') => {
                    // Clone the story first to avoid borrowing issues
                    if let Some(story) = self.get_selected_story().cloned() {
                        self.show_edit_popup = true;
                        self.edit_popup_state = EditPopupState::from_story(&story);
                        // Set the epic selector index based on current epic
                        self.edit_popup_state.epic_selector_index =
                            if let Some(epic_id) = story.epic_id {
                                self.epics
                                    .iter()
                                    .position(|e| e.id == epic_id)
                                    .map(|i| i + 1)
                                    .unwrap_or(0)
                            } else {
                                0 // None selected
                            };
                    }
                }
                KeyCode::Char('n') => {
                    // Load more stories (next page)
                    self.request_load_more();
                }
                KeyCode::Char('v') => {
                    // Toggle view mode between columns and list
                    self.toggle_view_mode();
                }
                KeyCode::Char('r') => {
                    // Refresh stories - trigger a reload from the beginning
                    self.refresh_stories();
                }
                KeyCode::Char('f') => {
                    // Toggle epic filter selector
                    self.toggle_epic_selector();
                }
                KeyCode::Char('?') => {
                    // Show help popup
                    self.show_help_popup = true;
                    self.help_selected_index = 0;
                }
                KeyCode::Char('g') => {
                    // Create git branch for selected story
                    if self.git_context.is_git_repo()
                        && let Some(story) = self.get_selected_story().cloned()
                    {
                        // Use the formatted VCS branch name from Shortcut if available, otherwise generate one
                        let suggested_branch =
                            story.formatted_vcs_branch_name.unwrap_or_else(|| {
                                format!(
                                    "sc-{}-{}",
                                    story.id,
                                    story.name.replace([' ', '/'], "-").to_lowercase()
                                )
                            });
                        self.show_git_popup = true;
                        self.git_popup_state = GitBranchPopupState {
                            branch_name_textarea: {
                                let mut textarea = TextArea::default();
                                textarea.set_cursor_line_style(Style::default());
                                textarea.set_block(
                                    Block::default().borders(Borders::ALL).title("Branch Name"),
                                );
                                textarea.insert_str(&suggested_branch);
                                textarea
                            },
                            worktree_path_textarea: {
                                let mut textarea = TextArea::default();
                                textarea.set_cursor_line_style(Style::default());
                                textarea.set_block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title("Worktree Path"),
                                );
                                textarea.insert_str(crate::git::generate_worktree_path(
                                    &suggested_branch,
                                ));
                                textarea
                            },
                            selected_option: if self.git_context.is_bare_repo() {
                                GitBranchOption::CreateWorktree
                            } else {
                                GitBranchOption::CreateBranch
                            },
                            story_id: story.id,
                            editing_branch_name: false,
                            editing_worktree_path: false,
                        };
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn get_owner_names(&self, owner_ids: &[String]) -> Vec<String> {
        owner_ids
            .iter()
            .map(|id| {
                self.member_cache.get(id).cloned().unwrap_or_else(|| {
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

    pub fn merge_stories(&mut self, new_stories: Vec<Story>, next_page_token: Option<String>) {
        // Filter new stories using the same logic as App::new
        let filtered_stories: Vec<Story> = new_stories
            .into_iter()
            .filter(|story| {
                if is_done_state(story.workflow_state_id, &self.workflows) {
                    // For Done states, only keep stories completed in the current week
                    if let Some(completed_at) = &story.completed_at {
                        return is_current_week(completed_at);
                    } else if let Some(moved_at) = &story.moved_at {
                        // Fall back to moved_at if completed_at is not available
                        return is_current_week(moved_at);
                    } else {
                        // If no completion date available, use updated_at as fallback
                        return is_current_week(&story.updated_at);
                    }
                }
                // Keep all non-Done stories
                true
            })
            .collect();

        // Add filtered stories to unfiltered list, avoiding duplicates
        for story in filtered_stories.iter() {
            if !self
                .all_stories_unfiltered
                .iter()
                .any(|existing| existing.id == story.id)
            {
                self.all_stories_unfiltered.push(story.clone());
            }
        }

        // Re-apply epic filter to update the display
        self.apply_epic_filter();

        // Update pagination state
        self.next_page_token = next_page_token;
        self.is_loading = false;
        self.load_more_requested = false;
    }

    pub fn request_load_more(&mut self) {
        if self.next_page_token.is_some() && !self.is_loading {
            self.load_more_requested = true;
            self.is_loading = true;
        }
    }

    pub fn has_more_stories(&self) -> bool {
        self.next_page_token.is_some()
    }

    pub fn refresh_stories(&mut self) {
        // Set flag to request a refresh
        self.refresh_requested = true;
        self.is_loading = true;

        // Clear existing stories to prepare for fresh data
        self.stories_by_state.clear();
        self.all_stories_list.clear();
        self.all_stories_unfiltered.clear();
        self.total_loaded_stories = 0;
        self.next_page_token = None;

        // Reset selection to avoid out-of-bounds issues
        self.selected_column = 0;
        self.selected_row = 0;
        self.list_selected_index = 0;
        self.list_scroll_offset = 0;
    }

    pub fn set_epics(&mut self, epics: Vec<Epic>) {
        self.epics = epics;
    }

    pub fn apply_epic_filter(&mut self) {
        // Start with all unfiltered stories
        let filtered_stories = if let Some(epic_id) = self.selected_epic_filter {
            // Filter stories by selected epic
            self.all_stories_unfiltered
                .iter()
                .filter(|story| story.epic_id == Some(epic_id))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            // No filter, use all stories
            self.all_stories_unfiltered.clone()
        };

        // Clear current grouped stories
        self.stories_by_state.clear();

        // Re-group filtered stories by workflow state
        for story in filtered_stories.iter() {
            self.stories_by_state
                .entry(story.workflow_state_id)
                .or_default()
                .push(story.clone());
        }

        // Sort stories within each state by position
        for stories in self.stories_by_state.values_mut() {
            stories.sort_by_key(|s| s.position);
        }

        // Apply limit of 10 stories for Done states
        for (&state_id, stories) in self.stories_by_state.iter_mut() {
            if is_done_state(state_id, &self.workflows) && stories.len() > 10 {
                stories.truncate(10);
            }
        }

        // Rebuild the flattened list for list view
        self.all_stories_list.clear();
        for stories in self.stories_by_state.values() {
            self.all_stories_list.extend(stories.iter().cloned());
        }
        self.all_stories_list.sort_by_key(|s| s.position);

        // Update total count
        self.total_loaded_stories = self.all_stories_list.len();

        // Reset selections to avoid out-of-bounds
        self.selected_column = 0;
        self.selected_row = 0;
        self.list_selected_index = 0;
        self.list_scroll_offset = 0;
    }

    pub fn toggle_epic_selector(&mut self) {
        self.show_epic_selector = !self.show_epic_selector;
        if self.show_epic_selector {
            self.epic_selector_index = 0;
        }
    }

    pub fn next_epic_selection(&mut self) {
        // +1 for the "All Stories" option
        let total_options = self.epics.len() + 1;
        if total_options > 0 {
            self.epic_selector_index = (self.epic_selector_index + 1) % total_options;
        }
    }

    pub fn previous_epic_selection(&mut self) {
        // +1 for the "All Stories" option
        let total_options = self.epics.len() + 1;
        if total_options > 0 {
            if self.epic_selector_index == 0 {
                self.epic_selector_index = total_options - 1;
            } else {
                self.epic_selector_index -= 1;
            }
        }
    }

    pub fn apply_selected_epic_filter(&mut self) {
        if self.epic_selector_index == 0 {
            // "All Stories" selected
            self.selected_epic_filter = None;
        } else if self.epic_selector_index > 0 && self.epic_selector_index <= self.epics.len() {
            // Epic selected
            self.selected_epic_filter = Some(self.epics[self.epic_selector_index - 1].id);
        }
        self.show_epic_selector = false;
        self.apply_epic_filter();
    }
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with epic filter status
    let (header_text, header_style) = if let Some(epic_id) = app.selected_epic_filter {
        if let Some(epic) = app.epics.iter().find(|e| e.id == epic_id) {
            (
                format!("Shortcut Stories TUI | 🔍 Epic: {}", epic.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                "Shortcut Stories TUI".to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        }
    } else {
        (
            "Shortcut Stories TUI | All Stories".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    };

    let header = Paragraph::new(header_text)
        .style(header_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    if app.list_view_mode {
        // List view mode - single column with all stories
        draw_list_view(frame, app, chunks[1]);
    } else {
        // Column view mode - columns for workflow states
        draw_column_view(frame, app, chunks[1]);
    }

    // Footer
    let footer_text = if app.show_state_selector {
        "[↑/k] [↓/j] select state | [Enter] confirm | [Esc] cancel".to_string()
    } else if app.show_detail {
        "[↑/k] [↓/j] scroll | [Esc] close detail | [q] quit".to_string()
    } else if app.is_loading {
        if app.refresh_requested {
            "Refreshing all stories... Please wait...".to_string()
        } else {
            format!(
                "Loading more stories... | {} stories loaded",
                app.total_loaded_stories
            )
        }
    } else if app.show_epic_selector {
        "[↑/k] [↓/j] select epic | [Enter] apply filter | [Esc] cancel".to_string()
    } else if app.list_view_mode {
        // List view mode footer - simplified
        let story_count_text = if app.selected_epic_filter.is_some() {
            format!("{} filtered", app.all_stories_list.len())
        } else {
            format!("{} stories", app.total_loaded_stories)
        };
        format!(
            "[↑↓] navigate | [Enter] details | [?] help | [q] quit | {}",
            story_count_text
        )
    } else {
        // Column view mode footer - simplified
        let story_count_text = if app.selected_epic_filter.is_some() {
            format!("{} filtered", app.all_stories_list.len())
        } else {
            format!("{} stories", app.total_loaded_stories)
        };
        format!(
            "[←→] columns | [↑↓] rows | [Enter] details | [?] help | [q] quit | {}",
            story_count_text
        )
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);

    // Detail popup
    if app.show_detail
        && let Some(story) = app.get_selected_story().cloned()
    {
        draw_detail_popup(frame, &story, app);
    }

    // State selector popup
    if app.show_state_selector
        && let Some(story) = app.get_selected_story()
    {
        draw_state_selector_popup(frame, story, app);
    }

    // Create story popup
    if app.show_create_popup {
        draw_create_popup(frame, app);
    }

    // Create epic popup
    if app.show_create_epic_popup {
        draw_create_epic_popup(frame, app);
    }

    // Edit story popup
    if app.show_edit_popup {
        draw_edit_popup(frame, app);
    }

    // Git branch popup
    if app.show_git_popup {
        draw_git_popup(frame, app);
    }

    if app.show_git_result_popup {
        draw_git_result_popup(frame, app);
    }

    // Help popup
    if app.show_help_popup {
        draw_help_popup(frame, app);
    }

    // Epic selector popup
    if app.show_epic_selector {
        draw_epic_selector_popup(frame, app);
    }
}

fn draw_detail_popup(frame: &mut Frame, story: &Story, app: &mut App) {
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);

    // Store the detail area and clear clickable URLs for this render
    app.detail_area = Some(area);
    app.clickable_urls.clear();

    let workflow_state = app
        .workflow_state_map
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

    // Add epic information if present
    if let Some(epic_id) = story.epic_id
        && let Some(epic) = app.epics.iter().find(|e| e.id == epic_id) {
            text_lines.push(Line::from(vec![
                Span::styled("Epic: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&epic.name, Style::default().fg(Color::Magenta)),
            ]));
            text_lines.push(Line::from(""));
        }

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
    text_lines.push(Line::from(vec![Span::styled(
        "Description:",
        Style::default().add_modifier(Modifier::BOLD),
    )]));

    // Add description lines
    if !story.description.is_empty() {
        for line in story.description.lines() {
            text_lines.push(Line::from(line.to_string()));
        }
    } else {
        text_lines.push(Line::from("No description available"));
    }

    text_lines.push(Line::from(""));
    // Track main story URL
    let url_line_index = text_lines.len();
    text_lines.push(Line::from(vec![
        Span::styled("URL: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(&story.app_url, Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
    ]));
    // Store URL position (will be adjusted for scroll later)
    app.clickable_urls.push(ClickableUrl {
        url: story.app_url.clone(),
        row: url_line_index as u16,
        start_col: 5, // "URL: " is 5 chars
        end_col: 5 + story.app_url.len() as u16,
    });

    // Add git branches section
    if !story.branches.is_empty() {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(vec![Span::styled(
            "Git Branches:",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        for branch in &story.branches {
            let branch_line_index = text_lines.len();
            let url_start = 2 + branch.name.len() + 3; // "  " + name + " - "
            text_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&branch.name, Style::default().fg(Color::Green)),
                Span::raw(" - "),
                Span::styled(&branch.url, Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
            ]));
            app.clickable_urls.push(ClickableUrl {
                url: branch.url.clone(),
                row: branch_line_index as u16,
                start_col: url_start as u16,
                end_col: (url_start + branch.url.len()) as u16,
            });
        }
    }

    // Add pull requests section
    if !story.pull_requests.is_empty() {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(vec![Span::styled(
            "Pull Requests:",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        for pr in &story.pull_requests {
            let status = if pr.merged {
                Span::styled("merged", Style::default().fg(Color::Magenta))
            } else if pr.closed {
                Span::styled("closed", Style::default().fg(Color::Red))
            } else if pr.draft {
                Span::styled("draft", Style::default().fg(Color::Yellow))
            } else {
                Span::styled("open", Style::default().fg(Color::Green))
            };

            text_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&pr.title, Style::default().fg(Color::White)),
                Span::raw(" ["),
                status,
                Span::raw("]"),
            ]));
            let pr_url_line_index = text_lines.len();
            text_lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(&pr.url, Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
            ]));
            app.clickable_urls.push(ClickableUrl {
                url: pr.url.clone(),
                row: pr_url_line_index as u16,
                start_col: 4, // "    " is 4 chars
                end_col: 4 + pr.url.len() as u16,
            });
        }
    }

    // Add commits section (show last 5 commits)
    if !story.commits.is_empty() {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(vec![Span::styled(
            "Recent Commits:",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        let recent_commits: Vec<_> = story.commits.iter().take(5).collect();
        for commit in recent_commits {
            let short_hash = if commit.hash.len() > 7 {
                &commit.hash[..7]
            } else {
                &commit.hash
            };
            let first_line = commit.message.lines().next().unwrap_or(&commit.message);
            text_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(short_hash, Style::default().fg(Color::Yellow)),
                Span::raw(" - "),
                Span::raw(first_line),
            ]));
            let commit_url_line_index = text_lines.len();
            text_lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(&commit.url, Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
            ]));
            app.clickable_urls.push(ClickableUrl {
                url: commit.url.clone(),
                row: commit_url_line_index as u16,
                start_col: 4, // "    " is 4 chars
                end_col: 4 + commit.url.len() as u16,
            });
        }
        if story.commits.len() > 5 {
            text_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("... and {} more commits", story.commits.len() - 5),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    // Add comments section
    if !story.comments.is_empty() {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(vec![Span::styled(
            "Comments:",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        text_lines.push(Line::from(""));

        for comment in &story.comments {
            // Resolve author name from member cache
            let author_name = app
                .member_cache
                .get(&comment.author_id)
                .cloned()
                .unwrap_or_else(|| comment.author_id.clone());

            // Add author and timestamp
            text_lines.push(Line::from(vec![
                Span::styled(
                    author_name,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(
                    comment.created_at.clone(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // Add comment text with proper line wrapping
            for line in comment.text.lines() {
                text_lines.push(Line::from(format!("  {line}")));
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
    let start_line = app
        .detail_scroll_offset
        .min(total_lines.saturating_sub(visible_lines));
    let end_line = (start_line + visible_lines).min(total_lines);
    let visible_text_lines = if start_line < total_lines {
        text_lines[start_line..end_line].to_vec()
    } else {
        text_lines
    };

    // Adjust clickable URL positions based on scroll offset
    // Only keep URLs that are visible and adjust their row positions
    app.clickable_urls.retain_mut(|url| {
        if url.row >= start_line as u16 && url.row < end_line as u16 {
            // Adjust row to be relative to visible area (add 1 for border)
            url.row = (url.row - start_line as u16) + 1;
            true
        } else {
            false
        }
    });

    // Create title with scroll indicator
    let scroll_indicator = if total_lines > content_height {
        format!(
            " Story Details ({}/{}) ",
            start_line + 1,
            total_lines.saturating_sub(content_height) + 1
        )
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

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(list, area);
}

fn draw_create_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 55, frame.area());
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
            Constraint::Length(3), // Epic field
            Constraint::Min(1),    // Space
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Name field - render TextArea widget
    let mut name_textarea = app.create_popup_state.name_textarea.clone();
    if app.create_popup_state.selected_field == CreateField::Name {
        name_textarea.set_block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        name_textarea.set_cursor_line_style(Style::default());
    } else {
        name_textarea.set_block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    }
    frame.render_widget(&name_textarea, chunks[0]);

    // Description field - render TextArea widget
    let mut desc_textarea = app.create_popup_state.description_textarea.clone();
    if app.create_popup_state.selected_field == CreateField::Description {
        desc_textarea.set_block(
            Block::default()
                .title("Description")
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        desc_textarea.set_cursor_line_style(Style::default());
    } else {
        desc_textarea.set_block(
            Block::default()
                .title("Description")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    }
    frame.render_widget(&desc_textarea, chunks[1]);

    // Type field
    let type_style = if app.create_popup_state.selected_field == CreateField::Type {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
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

    // Epic field
    let epic_style = if app.create_popup_state.selected_field == CreateField::Epic {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let epic_block = Block::default()
        .title("Epic")
        .borders(Borders::ALL)
        .border_style(epic_style);

    let epic_text = if let Some(epic_id) = app.create_popup_state.epic_id {
        // Find epic name from the list
        app.epics
            .iter()
            .find(|e| e.id == epic_id)
            .map(|e| {
                if app.create_popup_state.selected_field == CreateField::Epic {
                    format!("< {} >", e.name)
                } else {
                    e.name.clone()
                }
            })
            .unwrap_or_else(|| "Unknown Epic".to_string())
    } else if app.create_popup_state.selected_field == CreateField::Epic {
        "< None >".to_string()
    } else {
        "None".to_string()
    };

    let epic_widget = Paragraph::new(epic_text)
        .block(epic_block)
        .alignment(Alignment::Center);
    frame.render_widget(epic_widget, chunks[3]);

    // Help text
    let help_text = match app.create_popup_state.selected_field {
        CreateField::Type => "[↑/↓] change type | [Tab] next field | [Enter] next | [Esc] cancel",
        CreateField::Epic => "[↑/↓] change epic | [Tab] next field | [Enter] submit | [Esc] cancel",
        _ => "[Tab] next field | [Enter] next/submit | [Esc] cancel",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

fn draw_create_epic_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    // Create the main popup block
    let popup = Block::default()
        .title("Create New Epic")
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
            Constraint::Min(5),    // Description field
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Name field - render TextArea widget
    let mut name_textarea = app.create_epic_popup_state.name_textarea.clone();
    if app.create_epic_popup_state.selected_field == CreateEpicField::Name {
        name_textarea.set_block(
            Block::default()
                .title("Epic Name")
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        name_textarea.set_cursor_line_style(Style::default());
    } else {
        name_textarea.set_block(
            Block::default()
                .title("Epic Name")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    }
    frame.render_widget(&name_textarea, chunks[0]);

    // Description field - render TextArea widget
    let mut description_textarea = app.create_epic_popup_state.description_textarea.clone();
    if app.create_epic_popup_state.selected_field == CreateEpicField::Description {
        description_textarea.set_block(
            Block::default()
                .title("Description")
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        description_textarea.set_cursor_line_style(Style::default());
    } else {
        description_textarea.set_block(
            Block::default()
                .title("Description")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    }
    frame.render_widget(&description_textarea, chunks[1]);

    // Help text
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("[Tab] ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch fields  "),
            Span::styled("[Enter] ", Style::default().fg(Color::Yellow)),
            Span::raw("Next/Create  "),
            Span::styled("[Esc] ", Style::default().fg(Color::Yellow)),
            Span::raw("Cancel"),
        ]),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn draw_edit_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 55, frame.area());
    frame.render_widget(Clear, area);

    // Create the main popup block
    let popup = Block::default()
        .title(format!("Edit Story #{}", app.edit_popup_state.story_id))
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
            Constraint::Length(3), // Epic field
            Constraint::Min(1),    // Space
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Name field - render TextArea widget
    let mut name_textarea = app.edit_popup_state.name_textarea.clone();
    if app.edit_popup_state.selected_field == EditField::Name {
        name_textarea.set_block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        name_textarea.set_cursor_line_style(Style::default());
    } else {
        name_textarea.set_block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    }
    frame.render_widget(&name_textarea, chunks[0]);

    // Description field - render TextArea widget
    let mut desc_textarea = app.edit_popup_state.description_textarea.clone();
    if app.edit_popup_state.selected_field == EditField::Description {
        desc_textarea.set_block(
            Block::default()
                .title("Description")
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        desc_textarea.set_cursor_line_style(Style::default());
    } else {
        desc_textarea.set_block(
            Block::default()
                .title("Description")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    }
    frame.render_widget(&desc_textarea, chunks[1]);

    // Type field
    let type_style = if app.edit_popup_state.selected_field == EditField::Type {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let type_block = Block::default()
        .title("Type")
        .borders(Borders::ALL)
        .border_style(type_style);

    let type_text = if app.edit_popup_state.selected_field == EditField::Type {
        format!("< {} >", app.edit_popup_state.story_type)
    } else {
        app.edit_popup_state.story_type.clone()
    };

    let type_widget = Paragraph::new(type_text)
        .block(type_block)
        .alignment(Alignment::Center);
    frame.render_widget(type_widget, chunks[2]);

    // Epic field
    let epic_style = if app.edit_popup_state.selected_field == EditField::Epic {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let epic_block = Block::default()
        .title("Epic")
        .borders(Borders::ALL)
        .border_style(epic_style);

    let epic_text = if let Some(epic_id) = app.edit_popup_state.epic_id {
        // Find epic name from the list
        app.epics
            .iter()
            .find(|e| e.id == epic_id)
            .map(|e| {
                if app.edit_popup_state.selected_field == EditField::Epic {
                    format!("< {} >", e.name)
                } else {
                    e.name.clone()
                }
            })
            .unwrap_or_else(|| "Unknown Epic".to_string())
    } else if app.edit_popup_state.selected_field == EditField::Epic {
        "< None >".to_string()
    } else {
        "None".to_string()
    };

    let epic_widget = Paragraph::new(epic_text)
        .block(epic_block)
        .alignment(Alignment::Center);
    frame.render_widget(epic_widget, chunks[3]);

    // Help text
    let help_text = match app.edit_popup_state.selected_field {
        EditField::Type => "[↑/↓] change type | [Tab] next field | [Enter] next | [Esc] cancel",
        EditField::Epic => "[↑/↓] change epic | [Tab] next field | [Enter] save | [Esc] cancel",
        _ => "[Tab] next field | [Enter] next/save | [Esc] cancel",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

fn draw_git_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    // Create the main popup block
    let popup = Block::default()
        .title("Create Git Branch")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));
    frame.render_widget(popup, area);

    // Create inner area for content
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Create layout chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Branch name
            Constraint::Length(3), // Worktree path (if bare repo)
            Constraint::Min(4),    // Options
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Branch name field - render TextArea widget
    let mut branch_textarea = app.git_popup_state.branch_name_textarea.clone();
    let branch_title = if app.git_popup_state.editing_branch_name {
        "Branch Name (editing...)"
    } else {
        "Branch Name [Tab/e to edit]"
    };
    let branch_border_style = if app.git_popup_state.editing_branch_name {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    };
    branch_textarea.set_block(
        Block::default()
            .title(branch_title)
            .borders(Borders::ALL)
            .border_style(branch_border_style),
    );
    if app.git_popup_state.editing_branch_name {
        branch_textarea.set_cursor_line_style(Style::default());
    }
    frame.render_widget(&branch_textarea, chunks[0]);

    // Worktree path field (only for bare repos) - render TextArea widget
    if app.git_context.is_bare_repo() {
        let mut worktree_textarea = app.git_popup_state.worktree_path_textarea.clone();
        let worktree_title = if app.git_popup_state.editing_worktree_path {
            "Worktree Path (editing...)"
        } else {
            "Worktree Path [w to edit]"
        };
        let worktree_border_style = if app.git_popup_state.editing_worktree_path {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Blue)
        };
        worktree_textarea.set_block(
            Block::default()
                .title(worktree_title)
                .borders(Borders::ALL)
                .border_style(worktree_border_style),
        );
        if app.git_popup_state.editing_worktree_path {
            worktree_textarea.set_cursor_line_style(Style::default());
        }
        frame.render_widget(&worktree_textarea, chunks[1]);
    }

    // Options
    let mut options = Vec::new();

    if !app.git_context.is_bare_repo() {
        let create_branch_style =
            if app.git_popup_state.selected_option == GitBranchOption::CreateBranch {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
        options.push(ListItem::new("Create Branch").style(create_branch_style));
    }

    if app.git_context.is_bare_repo() {
        let create_worktree_style =
            if app.git_popup_state.selected_option == GitBranchOption::CreateWorktree {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
        options.push(ListItem::new("Create Worktree").style(create_worktree_style));
    }

    let cancel_style = if app.git_popup_state.selected_option == GitBranchOption::Cancel {
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    options.push(ListItem::new("Cancel").style(cancel_style));

    let list = List::new(options).block(
        Block::default()
            .title("Options")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(list, chunks[2]);

    // Help text
    let repo_type = if app.git_context.is_bare_repo() {
        "bare"
    } else {
        "normal"
    };
    let current_branch = app
        .git_context
        .current_branch
        .as_deref()
        .unwrap_or("unknown");
    let help_text = if app.git_popup_state.editing_branch_name {
        "Editing branch name | [Enter] save | [Esc] cancel | [←/→] move cursor | [Home/End] | [Ctrl+A/Ctrl+E] | [Backspace/Del] | Type to edit".to_string()
    } else if app.git_popup_state.editing_worktree_path {
        "Editing worktree path | [Enter] save | [Esc] cancel | [←/→] move cursor | [Home/End] | [Ctrl+A/Ctrl+E] | [Backspace/Del] | Type to edit".to_string()
    } else {
        let base_help = format!(
            "Git repo: {repo_type} | Current branch: {current_branch} | [↑/↓] select | [Tab/e] edit name | [Enter] confirm | [Esc] cancel"
        );
        if app.git_context.is_bare_repo() {
            format!("{base_help} | [w] edit worktree path")
        } else {
            base_help
        }
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(help, chunks[3]);
}

fn draw_git_result_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    // Main popup block
    let title = if app.git_result_state.success {
        "✅ Git Operation Successful"
    } else {
        "❌ Git Operation Failed"
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if app.git_result_state.success {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Message
            Constraint::Min(3),    // Options (if available)
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Message
    let message_block = Block::default()
        .title("Result")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let message_text = Paragraph::new(app.git_result_state.message.as_str())
        .block(message_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(message_text, chunks[0]);

    // Options (only for successful worktree creation)
    if app.git_result_state.success && app.git_result_state.worktree_path.is_some() {
        let options_block = Block::default()
            .title("Options")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let continue_style = if app.git_result_state.selected_option == GitResultOption::Continue {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let exit_style = if app.git_result_state.selected_option == GitResultOption::ExitAndChange {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let options = vec![
            ListItem::new("Continue working in current session").style(continue_style),
            ListItem::new(format!(
                "Exit and change to worktree directory: {}",
                app.git_result_state.worktree_path.as_deref().unwrap_or("")
            ))
            .style(exit_style),
        ];

        let list = List::new(options).block(options_block);
        frame.render_widget(list, chunks[1]);
    }

    // Help text
    let help_text = if app.git_result_state.success && app.git_result_state.worktree_path.is_some()
    {
        "[↑/↓] select option | [Enter] confirm | [Esc] continue"
    } else {
        "[Enter] or [Esc] continue"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(help, chunks[2]);
}

fn draw_list_view(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.all_stories_list.is_empty() {
        // No stories
        let empty = Paragraph::new("No stories found")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    }

    // Calculate visible area and update scroll
    let content_height = area.height.saturating_sub(2) as usize; // Account for borders
    app.update_list_scroll(content_height);

    // Calculate which stories to show
    let visible_stories = content_height / 2; // Each story takes 2 lines
    let start_idx = app.list_scroll_offset;
    let end_idx = (start_idx + visible_stories).min(app.all_stories_list.len());

    // Available width for text content
    let available_width = area.width.saturating_sub(4) as usize;

    // Create list items for visible stories only
    let items: Vec<ListItem> = app.all_stories_list[start_idx..end_idx]
        .iter()
        .enumerate()
        .map(|(relative_idx, story)| {
            let story_idx = start_idx + relative_idx;
            // Check if story is owned by current user
            let is_owned = app
                .current_user_id
                .as_ref()
                .map(|uid| story.owner_ids.contains(uid))
                .unwrap_or(false);

            let is_selected = story_idx == app.list_selected_index;

            let style = if is_selected {
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
                "bug" => "🐞",
                "chore" => "🔧",
                _ => "📝",
            };

            // Get state name
            let state_name = app
                .workflow_state_map
                .get(&story.workflow_state_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            // Create first line with story info
            let prefix = format!("[#{}] {} [{}] ", story.id, type_icon, state_name);
            let first_line_width = available_width.saturating_sub(prefix.len());

            let mut line1_text = prefix.clone();
            let mut line2_text = String::new();

            if story.name.len() <= first_line_width {
                // Story name fits on first line
                line1_text.push_str(&story.name);
            } else {
                // Story name needs to wrap to second line
                line2_text = if story.name.len() > available_width {
                    story
                        .name
                        .chars()
                        .take(available_width.saturating_sub(3))
                        .collect::<String>()
                        + "..."
                } else {
                    story.name.clone()
                };
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

    // Create title with scroll indicators
    let visible_stories = content_height / 2;
    let has_scroll = app.all_stories_list.len() > visible_stories;
    let title = if has_scroll {
        let total_stories = app.all_stories_list.len();
        let showing_start = start_idx + 1;
        let showing_end = end_idx;
        format!(
            " All Stories ({total_stories}) - List View ({showing_start}-{showing_end} of {total_stories}) "
        )
    } else {
        format!(" All Stories ({}) - List View ", app.all_stories_list.len())
    };
    let title_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(title_style),
    );

    frame.render_widget(list, area);
}

fn draw_column_view(frame: &mut Frame, app: &App, area: Rect) {
    // Create columns for workflow states
    if !app.workflow_states.is_empty() {
        let num_columns = app.workflow_states.len();
        let column_constraints: Vec<Constraint> = (0..num_columns)
            .map(|_| Constraint::Percentage((100 / num_columns) as u16))
            .collect();

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(column_constraints)
            .split(area);

        // Render each workflow state column
        for (idx, (state_id, state_name)) in app.workflow_states.iter().enumerate() {
            let is_selected_column = idx == app.selected_column;

            // Get the actual column width
            let column_rect = columns[idx];
            // Account for borders (2) and some padding (2)
            let available_width = column_rect.width.saturating_sub(4) as usize;

            // Get stories for this state
            let stories = app
                .stories_by_state
                .get(state_id)
                .map(|s| s.as_slice())
                .unwrap_or(&[]);

            // Create list items
            let items: Vec<ListItem> = stories
                .iter()
                .enumerate()
                .map(|(story_idx, story)| {
                    // Check if story is owned by current user
                    let is_owned = app
                        .current_user_id
                        .as_ref()
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
                        "bug" => "🐞",
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
                                    line2_text = story
                                        .name
                                        .chars()
                                        .take(second_line_width.saturating_sub(3))
                                        .collect::<String>()
                                        + "...";
                                } else {
                                    line2_text = story.name.clone();
                                }
                            } else {
                                // Normal word wrapping
                                let mut current_length = 0;
                                let mut on_second_line = false;

                                for (i, word) in words.iter().enumerate() {
                                    let word_len = word.len() + if i > 0 { 1 } else { 0 }; // +1 for space

                                    if !on_second_line
                                        && current_length + word_len <= first_line_width
                                    {
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
                                            line2_text = word
                                                .chars()
                                                .take(second_line_width.saturating_sub(3))
                                                .collect::<String>()
                                                + "...";
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
                                                line2_text = line2_text
                                                    .chars()
                                                    .take(second_line_width.saturating_sub(3))
                                                    .collect::<String>()
                                                    + "...";
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
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let title = format!(" {} ({}) ", state_name, stories.len());

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(title_style),
            );

            frame.render_widget(list, columns[idx]);
        }
    } else {
        // No stories
        let empty = Paragraph::new("No stories found")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
    }
}

fn draw_epic_selector_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    // Create list items for epics
    let mut items: Vec<ListItem> = Vec::new();

    // Add "All Stories" option
    let all_stories_style = if app.epic_selector_index == 0 {
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    items.push(ListItem::new(" All Stories (no filter) ").style(all_stories_style));

    // Add each epic
    for (idx, epic) in app.epics.iter().enumerate() {
        let is_selected = idx + 1 == app.epic_selector_index;
        let is_current_filter = Some(epic.id) == app.selected_epic_filter;

        let style = if is_selected {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if is_current_filter {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let display_text = format!(" {} ", epic.name);
        items.push(ListItem::new(display_text).style(style));
    }

    // Create title with current filter status
    let title = if let Some(epic_id) = app.selected_epic_filter {
        if let Some(epic) = app.epics.iter().find(|e| e.id == epic_id) {
            format!(" Filter by Epic (Current: {}) ", epic.name)
        } else {
            " Filter by Epic ".to_string()
        }
    } else {
        " Filter by Epic (Current: All Stories) ".to_string()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(list, area);
}

fn draw_help_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, area);

    // Define keyboard shortcuts
    let shortcuts = vec![
        (
            "Navigation",
            vec![
                ("↑/k", "Move up"),
                ("↓/j", "Move down"),
                ("←/h", "Move left (column view)"),
                ("→/l", "Move right (column view)"),
            ],
        ),
        (
            "View",
            vec![
                ("Enter", "Show story details"),
                ("v", "Toggle list/column view"),
                ("f", "Filter by epic"),
                ("r", "Refresh all stories"),
                ("n", "Load more stories"),
            ],
        ),
        (
            "Story Actions",
            vec![
                ("Space", "Move story to another state"),
                ("o", "Take ownership of story"),
                ("e", "Edit story"),
                ("a", "Add new story"),
                ("E", "Create new epic"),
                ("g", "Create git branch (if in git repo)"),
            ],
        ),
        (
            "Application",
            vec![("?", "Show/hide this help"), ("q", "Quit application")],
        ),
    ];

    // Create the help content
    let mut text_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Keyboard Shortcuts",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let mut command_count = 0;
    for (section, commands) in &shortcuts {
        text_lines.push(Line::from(Span::styled(
            format!("  {}", section),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        text_lines.push(Line::from(""));

        for (key, description) in commands {
            let is_selected = command_count == app.help_selected_index;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled("    ", style),
                Span::styled(format!("{:<10}", key), style.fg(Color::Green)),
                Span::styled(format!(" {}", description), style),
            ]);
            text_lines.push(line);
            command_count += 1;
        }
        text_lines.push(Line::from(""));
    }

    text_lines.push(Line::from(""));
    text_lines.push(Line::from(Span::styled(
        "  Press Esc or ? to close",
        Style::default().fg(Color::DarkGray),
    )));

    let help_text = Paragraph::new(text_lines).block(
        Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(help_text, area);
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
