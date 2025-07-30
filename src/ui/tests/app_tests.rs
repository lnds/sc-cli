use crate::api::Story;
use crate::ui::App;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_test_stories() -> Vec<Story> {
        vec![
            Story {
                id: 1,
                name: "First Story".to_string(),
                description: "First description".to_string(),
                workflow_state_id: 10,
                app_url: "https://app.shortcut.com/org/story/1".to_string(),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec!["user1".to_string()],
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
            },
            Story {
                id: 2,
                name: "Second Story".to_string(),
                description: "Second description".to_string(),
                workflow_state_id: 20,
                app_url: "https://app.shortcut.com/org/story/2".to_string(),
                story_type: "bug".to_string(),
                labels: vec![],
                owner_ids: vec!["user2".to_string()],
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
            },
            Story {
                id: 3,
                name: "Third Story".to_string(),
                description: "Third description".to_string(),
                workflow_state_id: 30,
                app_url: "https://app.shortcut.com/org/story/3".to_string(),
                story_type: "chore".to_string(),
                labels: vec![],
                owner_ids: vec!["user3".to_string()],
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
            },
        ]
    }

    fn create_test_workflow_map() -> HashMap<i64, String> {
        let mut map = HashMap::new();
        map.insert(10, "To Do".to_string());
        map.insert(20, "In Progress".to_string());
        map.insert(30, "Done".to_string());
        map
    }

    #[test]
    fn test_app_creation() {
        let stories = create_test_stories();
        let workflow_map = create_test_workflow_map();
        let app = App::new(stories.clone(), workflow_map.clone());

        assert_eq!(app.stories.len(), 3);
        assert_eq!(app.list_state.selected(), Some(0));
        assert!(!app.show_detail);
        assert!(!app.should_quit);
        assert_eq!(app.workflow_state_map.len(), 3);
    }

    #[test]
    fn test_app_creation_empty_stories() {
        let stories = vec![];
        let workflow_map = create_test_workflow_map();
        let app = App::new(stories, workflow_map);

        assert!(app.stories.is_empty());
        assert_eq!(app.list_state.selected(), None);
    }

    #[test]
    fn test_navigation_next() {
        let stories = create_test_stories();
        let workflow_map = create_test_workflow_map();
        let mut app = App::new(stories, workflow_map);

        // Start at index 0
        assert_eq!(app.list_state.selected(), Some(0));

        // Move to next
        app.next();
        assert_eq!(app.list_state.selected(), Some(1));

        // Move to next
        app.next();
        assert_eq!(app.list_state.selected(), Some(2));

        // Wrap around to beginning
        app.next();
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_navigation_previous() {
        let stories = create_test_stories();
        let workflow_map = create_test_workflow_map();
        let mut app = App::new(stories, workflow_map);

        // Start at index 0
        assert_eq!(app.list_state.selected(), Some(0));

        // Move to previous (wraps to end)
        app.previous();
        assert_eq!(app.list_state.selected(), Some(2));

        // Move to previous
        app.previous();
        assert_eq!(app.list_state.selected(), Some(1));

        // Move to previous
        app.previous();
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_navigation_empty_stories() {
        let stories = vec![];
        let workflow_map = create_test_workflow_map();
        let mut app = App::new(stories, workflow_map);

        // Should not crash on empty list
        app.next();
        assert_eq!(app.list_state.selected(), None);

        app.previous();
        assert_eq!(app.list_state.selected(), None);
    }

    #[test]
    fn test_toggle_detail() {
        let stories = create_test_stories();
        let workflow_map = create_test_workflow_map();
        let mut app = App::new(stories, workflow_map);

        assert!(!app.show_detail);

        // Toggle on
        app.toggle_detail();
        assert!(app.show_detail);

        // Toggle off
        app.toggle_detail();
        assert!(!app.show_detail);
    }

    #[test]
    fn test_toggle_detail_empty_stories() {
        let stories = vec![];
        let workflow_map = create_test_workflow_map();
        let mut app = App::new(stories, workflow_map);

        // Should not toggle on empty list
        app.toggle_detail();
        assert!(!app.show_detail);
    }

    // Note: Event handling tests would require mocking crossterm events
    // which is complex for unit tests. These are better suited for integration tests.
}