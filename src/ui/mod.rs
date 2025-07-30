use crate::api::Story;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

pub struct App {
    pub stories: Vec<Story>,
    pub list_state: ListState,
    pub show_detail: bool,
    pub workflow_state_map: HashMap<i64, String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(stories: Vec<Story>, workflow_state_map: HashMap<i64, String>) -> Self {
        let mut list_state = ListState::default();
        if !stories.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            stories,
            list_state,
            show_detail: false,
            workflow_state_map,
            should_quit: false,
        }
    }

    pub fn next(&mut self) {
        if self.stories.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.stories.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.stories.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.stories.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_detail(&mut self) {
        if !self.stories.is_empty() {
            self.show_detail = !self.show_detail;
        }
    }

    pub fn handle_events(&mut self) -> anyhow::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('j') | KeyCode::Down => self.next(),
                        KeyCode::Char('k') | KeyCode::Up => self.previous(),
                        KeyCode::Enter => self.toggle_detail(),
                        KeyCode::Esc if self.show_detail => self.show_detail = false,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
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

    // Story list
    let stories: Vec<ListItem> = app
        .stories
        .iter()
        .map(|story| {
            let content = format!("[#{}] {}", story.id, story.name);
            ListItem::new(content)
                .style(Style::default().fg(Color::White))
        })
        .collect();

    let stories_list = List::new(stories)
        .block(Block::default().borders(Borders::ALL).title("Stories"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(stories_list, chunks[1], &mut app.list_state.clone());

    // Footer
    let footer_text = if app.show_detail {
        "Press [Esc] to close detail | [q] to quit"
    } else {
        "Press [↑/k] [↓/j] to navigate | [Enter] for details | [q] to quit"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);

    // Detail popup
    if app.show_detail {
        if let Some(selected) = app.list_state.selected() {
            if let Some(story) = app.stories.get(selected) {
                draw_detail_popup(frame, story, &app.workflow_state_map);
            }
        }
    }
}

fn draw_detail_popup(frame: &mut Frame, story: &Story, workflow_map: &HashMap<i64, String>) {
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);

    let workflow_state = workflow_map
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
        Line::from(vec![
            Span::styled("Description:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
    ];

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