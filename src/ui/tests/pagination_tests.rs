use super::super::*;
use crate::api::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stories() -> Vec<Story> {
        vec![
            Story {
                id: 1,
                name: "Test Story 1".to_string(),
                description: "Test Description 1".to_string(),
                workflow_state_id: 100,
                app_url: "https://example.com/1".to_string(),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![],
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
                position: 1,
            },
            Story {
                id: 2,
                name: "Test Story 2".to_string(),
                description: "Test Description 2".to_string(),
                workflow_state_id: 200,
                app_url: "https://example.com/2".to_string(),
                story_type: "bug".to_string(),
                labels: vec![],
                owner_ids: vec![],
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
                position: 1,
            },
        ]
    }

    fn create_test_workflows() -> Vec<Workflow> {
        vec![Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 100,
                    name: "To Do".to_string(),
                    position: 1,
                    color: "#cccccc".to_string(),
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 200,
                    name: "In Progress".to_string(),
                    position: 2,
                    color: "#0000ff".to_string(),
                    state_type: "started".to_string(),
                },
                WorkflowState {
                    id: 300,
                    name: "Done".to_string(),
                    position: 3,
                    color: "#00ff00".to_string(),
                    state_type: "done".to_string(),
                },
            ],
        }]
    }

    #[test]
    fn test_app_pagination_state() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();
        let next_page_token = Some("next_token_123".to_string());

        let app = App::new(
            stories,
            workflows,
            search_query.clone(),
            next_page_token.clone(),
        );

        assert_eq!(app.search_query, search_query);
        assert_eq!(app.next_page_token, next_page_token);
        assert_eq!(app.total_loaded_stories, 2);
        assert!(!app.is_loading);
        assert!(!app.load_more_requested);
        assert!(app.has_more_stories());
    }

    #[test]
    fn test_app_no_more_stories() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();

        let app = App::new(stories, workflows, search_query, None);

        assert!(!app.has_more_stories());
    }

    #[test]
    fn test_request_load_more() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();
        let next_page_token = Some("next_token_123".to_string());

        let mut app = App::new(stories, workflows, search_query, next_page_token);

        // Should be able to request load more
        app.request_load_more();
        assert!(app.load_more_requested);
        assert!(app.is_loading);
    }

    #[test]
    fn test_request_load_more_no_token() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();

        let mut app = App::new(stories, workflows, search_query, None);

        // Should not request load more when no token
        app.request_load_more();
        assert!(!app.load_more_requested);
        assert!(!app.is_loading);
    }

    #[test]
    fn test_request_load_more_already_loading() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();
        let next_page_token = Some("next_token_123".to_string());

        let mut app = App::new(stories, workflows, search_query, next_page_token);

        // Set already loading
        app.is_loading = true;

        // Should not request load more when already loading
        app.request_load_more();
        assert!(!app.load_more_requested);
        assert!(app.is_loading);
    }

    #[test]
    fn test_merge_stories() {
        let initial_stories = vec![create_test_stories()[0].clone()];
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();
        let initial_token = Some("initial_token".to_string());

        let mut app = App::new(initial_stories, workflows, search_query, initial_token);

        // Simulate requesting load more
        app.load_more_requested = true;
        app.is_loading = true;

        // Merge new stories
        let new_stories = vec![create_test_stories()[1].clone()];
        let new_token = Some("new_token".to_string());

        app.merge_stories(new_stories, new_token.clone());

        // Check state after merge
        assert_eq!(app.total_loaded_stories, 2);
        assert_eq!(app.next_page_token, new_token);
        assert!(!app.is_loading);
        assert!(!app.load_more_requested);

        // Check that stories were merged correctly
        assert_eq!(app.stories_by_state.get(&100).unwrap().len(), 1);
        assert_eq!(app.stories_by_state.get(&200).unwrap().len(), 1);
    }

    #[test]
    fn test_merge_stories_same_state() {
        let initial_stories = vec![create_test_stories()[0].clone()];
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();

        let mut app = App::new(initial_stories, workflows, search_query, None);

        // Create another story in the same state
        let mut new_story = create_test_stories()[0].clone();
        new_story.id = 3;
        new_story.name = "Test Story 3".to_string();
        new_story.position = 2;

        let new_stories = vec![new_story];

        app.merge_stories(new_stories, None);

        // Check that both stories are in the same state and sorted by position
        let state_100_stories = app.stories_by_state.get(&100).unwrap();
        assert_eq!(state_100_stories.len(), 2);
        assert_eq!(state_100_stories[0].position, 1);
        assert_eq!(state_100_stories[1].position, 2);
        assert_eq!(app.total_loaded_stories, 2);
    }

    #[test]
    fn test_merge_stories_with_duplicates() {
        let initial_stories = create_test_stories();
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();

        let mut app = App::new(initial_stories, workflows, search_query, None);

        // Try to merge some of the same stories (should be ignored)
        let duplicate_stories = vec![
            create_test_stories()[0].clone(), // Duplicate of existing story
            create_test_stories()[1].clone(), // Duplicate of existing story
        ];

        app.merge_stories(duplicate_stories, None);

        // Should still have only 2 unique stories
        assert_eq!(app.total_loaded_stories, 2);
        assert_eq!(app.stories_by_state.get(&100).unwrap().len(), 1);
        assert_eq!(app.stories_by_state.get(&200).unwrap().len(), 1);
    }

    #[test]
    fn test_merge_stories_mixed_duplicates_and_new() {
        let initial_stories = vec![create_test_stories()[0].clone()];
        let workflows = create_test_workflows();
        let search_query = "owner:test".to_string();

        let mut app = App::new(initial_stories, workflows, search_query, None);

        // Mix of duplicate and new stories
        let mut new_story = create_test_stories()[0].clone();
        new_story.id = 3; // Make it unique
        new_story.name = "New Unique Story".to_string();

        let mixed_stories = vec![
            create_test_stories()[0].clone(), // Duplicate (should be ignored)
            new_story,                        // New (should be added)
        ];

        app.merge_stories(mixed_stories, None);

        // Should have 2 total stories (1 initial + 1 new, duplicate ignored)
        assert_eq!(app.total_loaded_stories, 2);
        assert_eq!(app.stories_by_state.get(&100).unwrap().len(), 2);
    }
}
