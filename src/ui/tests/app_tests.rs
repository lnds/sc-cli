use crate::api::{Story, Workflow, WorkflowState};
use crate::ui::App;

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
                position: 1000,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
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
                position: 2000,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
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
                position: 3000,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: Some("2025-08-01T10:00:00Z".to_string()),
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
        ]
    }

    fn create_test_workflows() -> Vec<Workflow> {
        vec![Workflow {
            id: 1,
            name: "Default Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 10,
                    name: "To Do".to_string(),
                    color: "#000000".to_string(),
                    position: 1,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 20,
                    name: "In Progress".to_string(),
                    color: "#f39c12".to_string(),
                    position: 2,
                    state_type: "started".to_string(),
                },
                WorkflowState {
                    id: 30,
                    name: "Done".to_string(),
                    color: "#27ae60".to_string(),
                    position: 3,
                    state_type: "done".to_string(),
                },
            ],
        }]
    }

    #[test]
    fn test_app_creation() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let app = App::new(stories.clone(), workflows, "test query".to_string(), None);

        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);
        assert!(!app.show_detail);
        assert!(!app.should_quit);
        assert_eq!(app.workflow_state_map.len(), 3);
        assert_eq!(app.workflow_states.len(), 3);
        assert_eq!(app.stories_by_state.len(), 2); // Only 2 because Done story is filtered out
    }

    #[test]
    fn test_app_creation_empty_stories() {
        let stories = vec![];
        let workflows = create_test_workflows();
        let app = App::new(stories, workflows, "test query".to_string(), None);

        // Should show all workflow states even with no stories
        assert_eq!(app.workflow_states.len(), 3);
        assert_eq!(app.stories_by_state.len(), 0);
    }

    #[test]
    fn test_navigation_next() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // We have 3 stories, each in different workflow state
        // The app should have 3 columns, one for each state
        assert_eq!(app.workflow_states.len(), 3);
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);

        // Since each workflow state has only one story,
        // next() should wrap around to the same story
        app.next();
        assert_eq!(app.selected_row, 0);

        // Switch to next column and test navigation there
        app.next_column();
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.selected_row, 0);
    }

    #[test]
    fn test_navigation_previous() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Start at column 0, row 0
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);

        // Since each workflow state has only one story,
        // previous() should wrap around to the same story
        app.previous();
        assert_eq!(app.selected_row, 0);

        // Test column navigation
        app.previous_column();
        assert_eq!(app.selected_column, 2); // Wrapped to last column

        app.previous_column();
        assert_eq!(app.selected_column, 1);

        app.previous_column();
        assert_eq!(app.selected_column, 0);
    }

    #[test]
    fn test_navigation_empty_stories() {
        let stories = vec![];
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Should not crash on empty list
        app.next();
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);

        app.previous();
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);
        
        // With empty stories but 3 workflow states, column navigation should work
        app.next_column();
        assert_eq!(app.selected_column, 1);
        
        app.next_column();
        assert_eq!(app.selected_column, 2);
        
        app.next_column();
        assert_eq!(app.selected_column, 0); // Wrap around
        
        app.previous_column();
        assert_eq!(app.selected_column, 2); // Wrap around
    }

    #[test]
    fn test_toggle_detail() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

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
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Should not toggle on empty list
        app.toggle_detail();
        assert!(!app.show_detail);
    }

    #[test]
    fn test_set_current_user_id() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Initially no current user
        assert_eq!(app.current_user_id, None);

        // Set current user
        app.set_current_user_id("user1".to_string());
        assert_eq!(app.current_user_id, Some("user1".to_string()));

        // Change current user
        app.set_current_user_id("user2".to_string());
        assert_eq!(app.current_user_id, Some("user2".to_string()));
    }

    #[test]
    fn test_stories_sorted_by_position() {
        let stories = vec![
            Story {
                id: 3,
                name: "Third Story".to_string(),
                description: "".to_string(),
                workflow_state_id: 10,
                app_url: "".to_string(),
                story_type: "".to_string(),
                labels: vec![],
                owner_ids: vec![],
                position: 3000, // Higher position
                created_at: "".to_string(),
                updated_at: "".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
            Story {
                id: 1,
                name: "First Story".to_string(),
                description: "".to_string(),
                workflow_state_id: 10,
                app_url: "".to_string(),
                story_type: "".to_string(),
                labels: vec![],
                owner_ids: vec![],
                position: 1000, // Lower position (should come first)
                created_at: "".to_string(),
                updated_at: "".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
            Story {
                id: 2,
                name: "Second Story".to_string(),
                description: "".to_string(),
                workflow_state_id: 10,
                app_url: "".to_string(),
                story_type: "".to_string(),
                labels: vec![],
                owner_ids: vec![],
                position: 2000, // Middle position
                created_at: "".to_string(),
                updated_at: "".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
        ];
        
        let workflows = vec![Workflow {
            id: 1,
            name: "Default Workflow".to_string(),
            states: vec![WorkflowState {
                id: 10,
                name: "To Do".to_string(),
                color: "#000000".to_string(),
                position: 1,
                state_type: "unstarted".to_string(),
            }],
        }];
        
        let app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Check that stories are sorted by position
        let sorted_stories = app.stories_by_state.get(&10).unwrap();
        assert_eq!(sorted_stories.len(), 3);
        assert_eq!(sorted_stories[0].id, 1); // First by position
        assert_eq!(sorted_stories[1].id, 2); // Second by position
        assert_eq!(sorted_stories[2].id, 3); // Third by position
    }

    #[test]
    fn test_create_story_popup() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Initially popup should not be shown
        assert!(!app.show_create_popup);
        
        // Simulate pressing 'a' key
        app.handle_key_event(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();
        
        // Popup should now be shown
        assert!(app.show_create_popup);
        assert_eq!(app.create_popup_state.selected_field, crate::ui::CreateField::Name);
        
        // Test typing in name field
        app.handle_key_event(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('T'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();
        
        assert_eq!(app.create_popup_state.name_textarea.lines()[0], "T");
        
        // Test Tab to move to description
        app.handle_key_event(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Tab,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();
        
        assert_eq!(app.create_popup_state.selected_field, crate::ui::CreateField::Description);
        
        // Test Esc to close
        app.handle_key_event(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Esc,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();
        
        assert!(!app.show_create_popup);
    }

    #[test]
    fn test_detail_scroll_functionality() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);
        
        // Initially scroll offset should be 0
        assert_eq!(app.detail_scroll_offset, 0);
        
        // Show detail view
        app.toggle_detail();
        assert!(app.show_detail);
        assert_eq!(app.detail_scroll_offset, 0); // Should reset on open
        
        // Test scrolling down
        app.detail_scroll_offset += 1;
        assert_eq!(app.detail_scroll_offset, 1);
        
        app.detail_scroll_offset += 1;
        assert_eq!(app.detail_scroll_offset, 2);
        
        // Test scrolling up
        app.scroll_detail_up();
        assert_eq!(app.detail_scroll_offset, 1);
        
        app.scroll_detail_up();
        assert_eq!(app.detail_scroll_offset, 0);
        
        // Test that scrolling up at 0 doesn't go negative
        app.scroll_detail_up();
        assert_eq!(app.detail_scroll_offset, 0);
        
        // Test that closing detail view resets scroll
        app.detail_scroll_offset = 5;
        app.toggle_detail(); // Close
        assert!(!app.show_detail);
        // Scroll offset should still be 5 until we open again
        app.toggle_detail(); // Open
        assert!(app.show_detail);
        assert_eq!(app.detail_scroll_offset, 0); // Reset on open
    }

    #[test]
    fn test_initial_selection_first_non_empty_column() {
        // Create stories where first workflow state is empty, second has stories
        let stories = vec![
            Story {
                id: 1,
                name: "Story in second state".to_string(),
                description: "".to_string(),
                workflow_state_id: 20, // Second workflow state
                app_url: "".to_string(),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![],
                position: 1000,
                created_at: "".to_string(),
                updated_at: "".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
            Story {
                id: 2,
                name: "Another story in second state".to_string(),
                description: "".to_string(),
                workflow_state_id: 20, // Second workflow state
                app_url: "".to_string(),
                story_type: "bug".to_string(),
                labels: vec![],
                owner_ids: vec![],
                position: 2000,
                created_at: "".to_string(),
                updated_at: "".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
        ];

        let workflows = vec![Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 10,
                    name: "Empty State".to_string(),
                    color: "#000000".to_string(),
                    position: 1, // First position but empty
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 20,
                    name: "Has Stories".to_string(),
                    color: "#f39c12".to_string(),
                    position: 2, // Second position with stories
                    state_type: "started".to_string(),
                },
                WorkflowState {
                    id: 30,
                    name: "Another Empty".to_string(),
                    color: "#27ae60".to_string(),
                    position: 3, // Third position but empty
                    state_type: "done".to_string(),
                },
            ],
        }];

        let app = App::new(stories, workflows, "test query".to_string(), None);

        // Should select column 1 (second workflow state) since first is empty
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.selected_row, 0);
        
        // Verify the selected story is correct
        let selected_story = app.get_selected_story().unwrap();
        assert_eq!(selected_story.id, 1); // First story in the second state
    }

    #[test]
    fn test_initial_selection_first_column_has_stories() {
        // Create stories where first workflow state has stories
        let stories = vec![
            Story {
                id: 1,
                name: "Story in first state".to_string(),
                description: "".to_string(),
                workflow_state_id: 10, // First workflow state
                app_url: "".to_string(),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![],
                position: 1000,
                created_at: "".to_string(),
                updated_at: "".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
        ];

        let workflows = vec![Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 10,
                    name: "Has Stories".to_string(),
                    color: "#000000".to_string(),
                    position: 1, // First position with stories
                    state_type: "started".to_string(),
                },
                WorkflowState {
                    id: 20,
                    name: "Empty State".to_string(),
                    color: "#f39c12".to_string(),
                    position: 2, // Second position but empty
                    state_type: "unstarted".to_string(),
                },
            ],
        }];

        let app = App::new(stories, workflows, "test query".to_string(), None);

        // Should select column 0 (first workflow state) since it has stories
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);
        
        // Verify the selected story is correct
        let selected_story = app.get_selected_story().unwrap();
        assert_eq!(selected_story.id, 1);
    }

    #[test]
    fn test_initial_selection_all_columns_empty() {
        // Create no stories
        let stories = vec![];

        let workflows = vec![Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 10,
                    name: "Empty State 1".to_string(),
                    color: "#000000".to_string(),
                    position: 1,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 20,
                    name: "Empty State 2".to_string(),
                    color: "#f39c12".to_string(),
                    position: 2,
                    state_type: "unstarted".to_string(),
                },
            ],
        }];

        let app = App::new(stories, workflows, "test query".to_string(), None);

        // Should still select column 0 (fallback behavior)
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);
        
        // Verify no story is selected
        assert!(app.get_selected_story().is_none());
    }

    #[test]
    fn test_initial_selection_last_column_has_stories() {
        // Create stories where only the last workflow state has stories
        let stories = vec![
            Story {
                id: 1,
                name: "Story in last state".to_string(),
                description: "".to_string(),
                workflow_state_id: 30, // Last workflow state
                app_url: "".to_string(),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![],
                position: 1000,
                created_at: "".to_string(),
                updated_at: "".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
                formatted_vcs_branch_name: None,
            epic_id: None,
            },
        ];

        let workflows = vec![Workflow {
            id: 1,
            name: "Test Workflow".to_string(),
            states: vec![
                WorkflowState {
                    id: 10,
                    name: "Empty State 1".to_string(),
                    color: "#000000".to_string(),
                    position: 1,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 20,
                    name: "Empty State 2".to_string(),
                    color: "#f39c12".to_string(),
                    position: 2,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 30,
                    name: "Has Stories".to_string(),
                    color: "#27ae60".to_string(),
                    position: 3, // Last position with stories
                    state_type: "started".to_string(),
                },
            ],
        }];

        let app = App::new(stories, workflows, "test query".to_string(), None);

        // Should select column 2 (last workflow state) since others are empty
        assert_eq!(app.selected_column, 2);
        assert_eq!(app.selected_row, 0);
        
        // Verify the selected story is correct
        let selected_story = app.get_selected_story().unwrap();
        assert_eq!(selected_story.id, 1);
    }

    // Note: Event handling tests would require mocking crossterm events
    // which is complex for unit tests. These are better suited for integration tests.
}