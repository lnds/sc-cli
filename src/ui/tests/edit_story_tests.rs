#[cfg(test)]
mod tests {
    use crate::ui::{App, EditPopupState, EditField};
    use crate::api::{Story, Workflow, WorkflowState};

    fn create_test_story() -> Story {
        Story {
            id: 123,
            name: "Test Story".to_string(),
            description: "Original description".to_string(),
            workflow_state_id: 1,
            app_url: "https://app.shortcut.com/test/story/123".to_string(),
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
        }
    }

    fn create_test_workflow() -> Workflow {
        Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 1,
                    name: "To Do".to_string(),
                    color: "#ffffff".to_string(),
                    position: 1,
                    state_type: "unstarted".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_edit_popup_state_from_story() {
        let story = create_test_story();
        let edit_state = EditPopupState::from_story(&story);
        
        assert_eq!(edit_state.name, "Test Story");
        assert_eq!(edit_state.description, "Original description");
        assert_eq!(edit_state.story_type, "feature");
        assert_eq!(edit_state.story_type_index, 0); // feature is at index 0
        assert_eq!(edit_state.story_id, 123);
        assert_eq!(edit_state.selected_field, EditField::Name);
    }

    #[test]
    fn test_edit_popup_state_story_type_index() {
        let mut story = create_test_story();
        
        // Test bug type
        story.story_type = "bug".to_string();
        let edit_state = EditPopupState::from_story(&story);
        assert_eq!(edit_state.story_type_index, 1);
        
        // Test chore type
        story.story_type = "chore".to_string();
        let edit_state = EditPopupState::from_story(&story);
        assert_eq!(edit_state.story_type_index, 2);
        
        // Test unknown type defaults to feature
        story.story_type = "unknown".to_string();
        let edit_state = EditPopupState::from_story(&story);
        assert_eq!(edit_state.story_type_index, 0);
    }

    #[test]
    fn test_edit_story_trigger() {
        let stories = vec![create_test_story()];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Initially edit popup should be hidden
        assert!(!app.show_edit_popup);
        assert!(!app.edit_story_requested);
        
        // Simulate pressing 'e' key
        let key_event = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('e'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        app.handle_key_event(key_event).unwrap();
        
        // Edit popup should now be shown
        assert!(app.show_edit_popup);
        assert_eq!(app.edit_popup_state.story_id, 123);
        assert_eq!(app.edit_popup_state.name, "Test Story");
        assert_eq!(app.edit_popup_state.description, "Original description");
        assert_eq!(app.edit_popup_state.story_type, "feature");
    }

    #[test]
    fn test_edit_popup_navigation() {
        let story = create_test_story();
        let stories = vec![story.clone()];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Show edit popup
        app.show_edit_popup = true;
        app.edit_popup_state = EditPopupState::from_story(&story);
        
        // Initially on Name field
        assert_eq!(app.edit_popup_state.selected_field, EditField::Name);
        
        // Press Tab to move to Description
        let tab_event = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Tab,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        app.handle_key_event(tab_event).unwrap();
        assert_eq!(app.edit_popup_state.selected_field, EditField::Description);
        
        // Press Tab to move to Type
        app.handle_key_event(tab_event).unwrap();
        assert_eq!(app.edit_popup_state.selected_field, EditField::Type);
        
        // Press Tab to cycle back to Name
        app.handle_key_event(tab_event).unwrap();
        assert_eq!(app.edit_popup_state.selected_field, EditField::Name);
    }

    #[test]
    fn test_edit_popup_story_type_navigation() {
        let story = create_test_story();
        let stories = vec![story.clone()];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Show edit popup and navigate to Type field
        app.show_edit_popup = true;
        app.edit_popup_state = EditPopupState::from_story(&story);
        app.edit_popup_state.selected_field = EditField::Type;
        
        // Initially feature (index 0)
        assert_eq!(app.edit_popup_state.story_type, "feature");
        assert_eq!(app.edit_popup_state.story_type_index, 0);
        
        // Press Down to move to bug
        let down_event = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Down,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        app.handle_key_event(down_event).unwrap();
        assert_eq!(app.edit_popup_state.story_type, "bug");
        assert_eq!(app.edit_popup_state.story_type_index, 1);
        
        // Press Down to move to chore
        app.handle_key_event(down_event).unwrap();
        assert_eq!(app.edit_popup_state.story_type, "chore");
        assert_eq!(app.edit_popup_state.story_type_index, 2);
        
        // Press Down to cycle back to feature
        app.handle_key_event(down_event).unwrap();
        assert_eq!(app.edit_popup_state.story_type, "feature");
        assert_eq!(app.edit_popup_state.story_type_index, 0);
    }

    #[test]
    fn test_edit_popup_text_input() {
        let story = create_test_story();
        let stories = vec![story.clone()];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Show edit popup
        app.show_edit_popup = true;
        app.edit_popup_state = EditPopupState::from_story(&story);
        app.edit_popup_state.name = String::new(); // Clear name for testing
        
        // Type some characters
        let char_event = |c| crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char(c),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        app.handle_key_event(char_event('H')).unwrap();
        app.handle_key_event(char_event('e')).unwrap();
        app.handle_key_event(char_event('l')).unwrap();
        app.handle_key_event(char_event('l')).unwrap();
        app.handle_key_event(char_event('o')).unwrap();
        
        assert_eq!(app.edit_popup_state.name, "Hello");
        
        // Test backspace
        let backspace_event = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Backspace,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        app.handle_key_event(backspace_event).unwrap();
        assert_eq!(app.edit_popup_state.name, "Hell");
    }

    #[test]
    fn test_edit_popup_submit() {
        let story = create_test_story();
        let stories = vec![story.clone()];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Show edit popup and navigate to Type field
        app.show_edit_popup = true;
        app.edit_popup_state = EditPopupState::from_story(&story);
        app.edit_popup_state.selected_field = EditField::Type;
        app.edit_popup_state.name = "Updated Story".to_string();
        
        // Press Enter to submit
        let enter_event = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Enter,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        app.handle_key_event(enter_event).unwrap();
        
        // Should set edit_story_requested and hide popup
        assert!(app.edit_story_requested);
        assert!(!app.show_edit_popup);
    }

    #[test]
    fn test_edit_popup_cancel() {
        let story = create_test_story();
        let stories = vec![story.clone()];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Show edit popup
        app.show_edit_popup = true;
        app.edit_popup_state = EditPopupState::from_story(&story);
        app.edit_popup_state.name = "Modified".to_string();
        
        // Press Escape to cancel
        let escape_event = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Esc,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        app.handle_key_event(escape_event).unwrap();
        
        // Should reset state and hide popup
        assert!(!app.show_edit_popup);
        assert!(!app.edit_story_requested);
        assert_eq!(app.edit_popup_state.name, "");
        assert_eq!(app.edit_popup_state.story_id, 0);
    }
}