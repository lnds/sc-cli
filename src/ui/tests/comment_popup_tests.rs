#[cfg(test)]
pub mod tests {
    use crate::api::{Story, Workflow, WorkflowState};
    use crate::ui::{App, CommentPopupState};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tui_textarea::TextArea;

    fn create_test_story(id: i64) -> Story {
        Story {
            id,
            name: format!("Test Story {}", id),
            description: "Test description".to_string(),
            workflow_state_id: 100,
            app_url: format!("https://app.shortcut.com/org/story/{}", id),
            story_type: "feature".to_string(),
            labels: vec![],
            owner_ids: vec![],
            position: 1000,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            completed_at: None,
            moved_at: None,
            comments: vec![],
            formatted_vcs_branch_name: None,
            epic_id: None,
            branches: vec![],
            pull_requests: vec![],
            commits: vec![],
        }
    }

    fn create_test_workflow() -> Vec<Workflow> {
        vec![Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 100,
                    name: "To Do".to_string(),
                    color: "#cccccc".to_string(),
                    position: 1,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 200,
                    name: "In Progress".to_string(),
                    color: "#0000ff".to_string(),
                    position: 2,
                    state_type: "started".to_string(),
                },
            ],
        }]
    }

    #[test]
    fn test_comment_popup_initialization() {
        let stories = vec![create_test_story(1)];
        let workflows = create_test_workflow();
        let app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Initially, comment popup should not be shown
        assert!(!app.show_comment_popup);
        assert_eq!(app.comment_popup_state.story_id, 0);
    }

    #[test]
    fn test_open_comment_popup_from_detail_view() {
        let stories = vec![create_test_story(42)];
        let workflows = create_test_workflow();
        let mut app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Open detail view first
        app.show_detail = true;

        // Simulate pressing 'c' key while in detail view
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        app.handle_key_event(key).unwrap();

        // Comment popup should now be shown
        assert!(app.show_comment_popup);
        assert_eq!(app.comment_popup_state.story_id, 42);
    }

    #[test]
    fn test_comment_popup_cancel_with_esc() {
        let stories = vec![create_test_story(42)];
        let workflows = create_test_workflow();
        let mut app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Open comment popup
        app.show_detail = true;
        app.show_comment_popup = true;
        app.comment_popup_state = CommentPopupState {
            comment_textarea: {
                let mut ta = TextArea::default();
                ta.insert_str("Test comment");
                ta
            },
            story_id: 42,
        };

        // Press Esc to cancel
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key_event(key).unwrap();

        // Comment popup should be closed
        assert!(!app.show_comment_popup);
        // Comment text should be cleared
        assert!(app.comment_popup_state.comment_textarea.lines().join("").is_empty());
    }

    #[test]
    fn test_comment_popup_submit_with_ctrl_enter() {
        let stories = vec![create_test_story(42)];
        let workflows = create_test_workflow();
        let mut app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Open comment popup with some text
        app.show_comment_popup = true;
        app.comment_popup_state = CommentPopupState {
            comment_textarea: {
                let mut ta = TextArea::default();
                ta.insert_str("This is a test comment");
                ta
            },
            story_id: 42,
        };

        // Press Ctrl+Enter to submit
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
        app.handle_key_event(key).unwrap();

        // Comment popup should be closed
        assert!(!app.show_comment_popup);
        // Request flag should be set
        assert!(app.add_comment_requested);
    }

    #[test]
    fn test_comment_popup_ignores_submit_when_empty() {
        let stories = vec![create_test_story(42)];
        let workflows = create_test_workflow();
        let mut app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Open comment popup with empty text
        app.show_comment_popup = true;
        app.comment_popup_state = CommentPopupState {
            comment_textarea: TextArea::default(),
            story_id: 42,
        };

        // Press Ctrl+Enter to submit
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
        app.handle_key_event(key).unwrap();

        // Comment popup should still be open (not submitted)
        assert!(app.show_comment_popup);
        // Request flag should NOT be set
        assert!(!app.add_comment_requested);
    }

    #[test]
    fn test_comment_popup_text_input() {
        let stories = vec![create_test_story(42)];
        let workflows = create_test_workflow();
        let mut app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Open comment popup
        app.show_comment_popup = true;
        app.comment_popup_state = CommentPopupState {
            comment_textarea: TextArea::default(),
            story_id: 42,
        };

        // Type some characters
        let chars = vec!['H', 'e', 'l', 'l', 'o'];
        for c in chars {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            app.handle_key_event(key).unwrap();
        }

        // Check that text was added to the textarea
        let text = app.comment_popup_state.comment_textarea.lines().join("");
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_comment_popup_not_accessible_without_detail_view() {
        let stories = vec![create_test_story(42)];
        let workflows = create_test_workflow();
        let mut app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Ensure detail view is NOT open
        app.show_detail = false;

        // Try to press 'c' key
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        app.handle_key_event(key).unwrap();

        // Comment popup should NOT open
        assert!(!app.show_comment_popup);
    }

    #[test]
    fn test_comment_popup_multiline_input() {
        let stories = vec![create_test_story(42)];
        let workflows = create_test_workflow();
        let mut app = App::new(stories, workflows, "owner:test".to_string(), None);

        // Open comment popup
        app.show_comment_popup = true;
        app.comment_popup_state = CommentPopupState {
            comment_textarea: TextArea::default(),
            story_id: 42,
        };

        // Type first line
        app.comment_popup_state.comment_textarea.insert_str("First line");

        // Press Enter for new line (without Ctrl)
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.handle_key_event(key).unwrap();

        // Type second line
        app.comment_popup_state.comment_textarea.insert_str("Second line");

        // Verify multiline content
        let lines = app.comment_popup_state.comment_textarea.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "First line");
        assert_eq!(lines[1], "Second line");
    }
}