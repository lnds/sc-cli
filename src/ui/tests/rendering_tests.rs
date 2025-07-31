use crate::api::{Story, Workflow, WorkflowState};
use crate::ui::{App, draw};
use ratatui::{backend::TestBackend, Terminal};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        let stories = vec![
            Story {
                id: 123,
                name: "Test Story".to_string(),
                description: "A test description that is long enough to test wrapping behavior in the detail view".to_string(),
                workflow_state_id: 456,
                app_url: "https://app.shortcut.com/org/story/123".to_string(),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec!["test-user".to_string()],
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
            },
        ];

        let workflows = vec![Workflow {
            id: 1,
            name: "Default Workflow".to_string(),
            states: vec![WorkflowState {
                id: 456,
                name: "In Progress".to_string(),
                color: "#f39c12".to_string(),
                position: 1,
            }],
        }];

        App::new(stories, workflows)
    }

    #[test]
    fn test_render_main_view() {
        let app = create_test_app();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| draw(f, &app)).unwrap();

        let buffer = terminal.backend().buffer();
        
        // Check header is rendered
        // Convert buffer to string for easier testing
        let mut buffer_str = String::new();
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                if let Some(cell) = buffer.cell((x, y)) {
                    buffer_str.push_str(cell.symbol());
                }
            }
        }
        
        // Check header is rendered
        assert!(buffer_str.contains("Shortcut Stories TUI"));

        // Check story list contains our story
        assert!(buffer_str.contains("[#123]"));
        assert!(buffer_str.contains("Test Story"));

        // Check footer is rendered - at least parts of it should be visible
        // The footer text might be truncated on a small terminal
        // Let's just check that some footer content exists
        let has_footer_content = buffer_str.contains("columns") || 
                                buffer_str.contains("navigate") || 
                                buffer_str.contains("details") ||
                                buffer_str.contains("[q]") || 
                                buffer_str.contains("quit");
        assert!(has_footer_content, "Footer should contain navigation instructions");
    }

    #[test]
    fn test_render_detail_view() {
        let mut app = create_test_app();
        app.show_detail = true;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| draw(f, &app)).unwrap();

        let buffer = terminal.backend().buffer();
        
        // Convert buffer to string for easier testing
        let mut buffer_str = String::new();
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                if let Some(cell) = buffer.cell((x, y)) {
                    buffer_str.push_str(cell.symbol());
                }
            }
        }
        
        // Check detail popup is rendered
        assert!(buffer_str.contains("Story Details"));
        assert!(buffer_str.contains("ID:"));
        assert!(buffer_str.contains("123"));
        assert!(buffer_str.contains("Name:"));
        assert!(buffer_str.contains("Test Story"));
        assert!(buffer_str.contains("Type:"));
        assert!(buffer_str.contains("feature"));
        assert!(buffer_str.contains("State:"));
        assert!(buffer_str.contains("In Progress"));
        assert!(buffer_str.contains("Description:"));
        assert!(buffer_str.contains("test description"));

        // Check footer shows different text
        assert!(buffer_str.contains("Esc"));
        assert!(buffer_str.contains("close detail"));
    }

    #[test]
    fn test_render_empty_list() {
        let stories = vec![];
        let workflows = vec![];
        let app = App::new(stories, workflows);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| draw(f, &app)).unwrap();

        let buffer = terminal.backend().buffer();
        
        // Convert buffer to string for easier testing
        let mut buffer_str = String::new();
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                if let Some(cell) = buffer.cell((x, y)) {
                    buffer_str.push_str(cell.symbol());
                }
            }
        }
        
        // Should still render header and footer
        assert!(buffer_str.contains("Shortcut Stories TUI"));
        assert!(buffer_str.contains("navigate"));
    }

    #[test]
    fn test_render_multiple_stories() {
        let stories = vec![
            Story {
                id: 1,
                name: "First Story".to_string(),
                description: "".to_string(),
                workflow_state_id: 10,
                app_url: "".to_string(),
                story_type: "".to_string(),
                labels: vec![],
                owner_ids: vec![],
                created_at: "".to_string(),
                updated_at: "".to_string(),
            },
            Story {
                id: 2,
                name: "Second Story".to_string(),
                description: "".to_string(),
                workflow_state_id: 20,
                app_url: "".to_string(),
                story_type: "".to_string(),
                labels: vec![],
                owner_ids: vec![],
                created_at: "".to_string(),
                updated_at: "".to_string(),
            },
        ];

        let workflows = vec![
            Workflow {
                id: 1,
                name: "Default Workflow".to_string(),
                states: vec![
                    WorkflowState {
                        id: 10,
                        name: "To Do".to_string(),
                        color: "#000000".to_string(),
                        position: 1,
                    },
                    WorkflowState {
                        id: 20,
                        name: "In Progress".to_string(),
                        color: "#f39c12".to_string(),
                        position: 2,
                    },
                ],
            },
        ];
        let app = App::new(stories, workflows);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| draw(f, &app)).unwrap();

        let buffer = terminal.backend().buffer();
        
        // Convert buffer to string for easier testing
        let mut buffer_str = String::new();
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                if let Some(cell) = buffer.cell((x, y)) {
                    buffer_str.push_str(cell.symbol());
                }
            }
        }
        
        // Check both stories are rendered
        assert!(buffer_str.contains("[#1]"));
        assert!(buffer_str.contains("First Story"));
        assert!(buffer_str.contains("[#2]"));
        assert!(buffer_str.contains("Second Story"));
    }
}