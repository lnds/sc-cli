use crate::api::{Story, Workflow, WorkflowState};
use crate::ui::App;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_test_story(id: i64, state_id: i64) -> Story {
        Story {
            id,
            name: format!("Story {}", id),
            description: "Test description".to_string(),
            workflow_state_id: state_id,
            app_url: format!("https://app.shortcut.com/org/story/{}", id),
            story_type: "feature".to_string(),
            labels: vec![],
            owner_ids: vec!["user1".to_string()],
            position: id * 1000, // Use id to generate position
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
            comments: vec![],
        }
    }

    fn create_test_workflow() -> Workflow {
        Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
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
                WorkflowState {
                    id: 30,
                    name: "Done".to_string(),
                    color: "#27ae60".to_string(),
                    position: 3,
                },
            ],
        }
    }

    #[test]
    fn test_toggle_state_selector() {
        let stories = vec![create_test_story(1, 10)];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Initially state selector should be hidden
        assert!(!app.show_state_selector);

        // Toggle state selector
        app.toggle_state_selector();
        assert!(app.show_state_selector);
        assert_eq!(app.state_selector_index, 0);
    }

    #[test]
    fn test_toggle_state_selector_empty_column() {
        let stories = vec![];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Should not show state selector for empty column
        app.toggle_state_selector();
        assert!(!app.show_state_selector);
    }

    #[test]
    fn test_get_available_states_for_story() {
        let stories = vec![create_test_story(1, 10)]; // Story in "To Do" state
        let workflows = vec![create_test_workflow()];
        let app = App::new(stories, workflows, "test query".to_string(), None);

        let story = app.get_selected_story().unwrap();
        let available_states = app.get_available_states_for_story(story);

        // Should have 2 available states (excluding current state "To Do")
        assert_eq!(available_states.len(), 2);
        assert_eq!(available_states[0], (20, "In Progress".to_string()));
        assert_eq!(available_states[1], (30, "Done".to_string()));
    }

    #[test]
    fn test_state_selector_navigation() {
        let stories = vec![create_test_story(1, 10)];
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        app.toggle_state_selector();
        assert_eq!(app.state_selector_index, 0);

        // Navigate next
        app.next_state_selection();
        assert_eq!(app.state_selector_index, 1);

        // Navigate next (should wrap)
        app.next_state_selection();
        assert_eq!(app.state_selector_index, 0);

        // Navigate previous (should wrap to end)
        app.previous_state_selection();
        assert_eq!(app.state_selector_index, 1);

        // Navigate previous
        app.previous_state_selection();
        assert_eq!(app.state_selector_index, 0);
    }

    #[test]
    fn test_get_selected_target_state() {
        let stories = vec![create_test_story(1, 10)]; // Story in "To Do" state
        let workflows = vec![create_test_workflow()];
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        app.toggle_state_selector();
        
        // First available state should be "In Progress" (id: 20)
        assert_eq!(app.get_selected_target_state(), Some(20));

        // Move to next state
        app.next_state_selection();
        // Second available state should be "Done" (id: 30)
        assert_eq!(app.get_selected_target_state(), Some(30));
    }
}