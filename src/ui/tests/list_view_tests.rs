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
            },
            Story {
                id: 3,
                name: "Third Story".to_string(),
                description: "Third description".to_string(),
                workflow_state_id: 20, // Changed from 30 (Done) to 20 (In Progress)
                app_url: "https://app.shortcut.com/org/story/3".to_string(),
                story_type: "chore".to_string(),
                labels: vec![],
                owner_ids: vec!["user3".to_string()],
                position: 500, // Smallest position, should be first in list
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
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
    fn test_app_creation_with_list_view_fields() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let app = App::new(stories.clone(), workflows, "test query".to_string(), None);

        // Should start in column view mode
        assert!(!app.list_view_mode);
        assert_eq!(app.list_selected_index, 0);
        
        // all_stories_list should be populated and sorted by position
        assert_eq!(app.all_stories_list.len(), 3);
        // Should be sorted by position: Story 3 (500), Story 1 (1000), Story 2 (2000)
        assert_eq!(app.all_stories_list[0].id, 3);
        assert_eq!(app.all_stories_list[1].id, 1);
        assert_eq!(app.all_stories_list[2].id, 2);
    }

    #[test]
    fn test_toggle_view_mode() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Initially in column view
        assert!(!app.list_view_mode);
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);

        // Toggle to list view
        app.toggle_view_mode();
        assert!(app.list_view_mode);
        assert_eq!(app.list_selected_index, 0);

        // Toggle back to column view
        app.toggle_view_mode();
        assert!(!app.list_view_mode);
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.selected_row, 0);
    }

    #[test]
    fn test_navigation_in_list_view() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Switch to list view
        app.toggle_view_mode();
        assert!(app.list_view_mode);
        assert_eq!(app.list_selected_index, 0);

        // Navigate forward
        app.next();
        assert_eq!(app.list_selected_index, 1);

        app.next();
        assert_eq!(app.list_selected_index, 2);

        // Should wrap around
        app.next();
        assert_eq!(app.list_selected_index, 0);

        // Navigate backward
        app.previous();
        assert_eq!(app.list_selected_index, 2);

        app.previous();
        assert_eq!(app.list_selected_index, 1);

        app.previous();
        assert_eq!(app.list_selected_index, 0);
    }

    #[test]
    fn test_get_selected_story_in_list_view() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Switch to list view
        app.toggle_view_mode();
        
        // Should select first story in sorted order (Story 3 with position 500)
        let selected = app.get_selected_story().unwrap();
        assert_eq!(selected.id, 3);
        assert_eq!(selected.position, 500);

        // Navigate to next story
        app.next();
        let selected = app.get_selected_story().unwrap();
        assert_eq!(selected.id, 1);
        assert_eq!(selected.position, 1000);

        // Navigate to last story
        app.next();
        let selected = app.get_selected_story().unwrap();
        assert_eq!(selected.id, 2);
        assert_eq!(selected.position, 2000);
    }

    #[test]
    fn test_get_selected_story_in_column_view() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Should be in column view by default
        assert!(!app.list_view_mode);

        // Should select first story in first non-empty column
        let selected = app.get_selected_story().unwrap();
        // The selection depends on which column has stories first
        assert!(selected.id == 1 || selected.id == 2 || selected.id == 3);
    }

    #[test]
    fn test_navigation_in_column_view_vs_list_view() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // In column view, navigation should work differently
        assert!(!app.list_view_mode);

        // Navigate in column view
        app.next();
        // Could move to next row in same column or wrap

        // Switch to list view
        app.toggle_view_mode();
        assert!(app.list_view_mode);
        assert_eq!(app.list_selected_index, 0); // Reset to 0 when switching

        // Navigate in list view
        app.next();
        assert_eq!(app.list_selected_index, 1);

        // Switch back to column view
        app.toggle_view_mode();
        assert!(!app.list_view_mode);
        assert_eq!(app.selected_column, 0); // Reset to 0 when switching
        assert_eq!(app.selected_row, 0);
    }

    #[test]
    fn test_empty_stories_list_view() {
        let stories = vec![];
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Switch to list view
        app.toggle_view_mode();
        assert!(app.list_view_mode);
        assert_eq!(app.all_stories_list.len(), 0);
        assert_eq!(app.list_selected_index, 0);

        // Navigation should not crash
        app.next();
        assert_eq!(app.list_selected_index, 0);

        app.previous();
        assert_eq!(app.list_selected_index, 0);

        // get_selected_story should return None
        assert!(app.get_selected_story().is_none());
    }

    #[test]
    fn test_merge_stories_updates_list_view() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Switch to list view
        app.toggle_view_mode();
        assert_eq!(app.all_stories_list.len(), 3);

        // Add new stories via merge_stories
        let new_stories = vec![
            Story {
                id: 4,
                name: "Fourth Story".to_string(),
                description: "Fourth description".to_string(),
                workflow_state_id: 10,
                app_url: "https://app.shortcut.com/org/story/4".to_string(),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec!["user4".to_string()],
                position: 100, // Should be first in sorted order
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
            },
        ];

        app.merge_stories(new_stories, None);

        // all_stories_list should be updated and re-sorted
        assert_eq!(app.all_stories_list.len(), 4);
        // Should be sorted by position: Story 4 (100), Story 3 (500), Story 1 (1000), Story 2 (2000)
        assert_eq!(app.all_stories_list[0].id, 4);
        assert_eq!(app.all_stories_list[1].id, 3);
        assert_eq!(app.all_stories_list[2].id, 1);
        assert_eq!(app.all_stories_list[3].id, 2);
    }

    #[test]
    fn test_keyboard_event_view_toggle() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Initially in column view
        assert!(!app.list_view_mode);

        // Simulate pressing 'v' key
        app.handle_key_event(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('v'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();

        // Should now be in list view mode
        assert!(app.list_view_mode);

        // Press 'v' again to toggle back
        app.handle_key_event(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('v'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();

        // Should be back to column view
        assert!(!app.list_view_mode);
    }

    #[test]
    fn test_list_view_scroll_initialization() {
        let stories = create_test_stories();
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Switch to list view
        app.toggle_view_mode();
        
        // Initial scroll offset should be 0
        assert_eq!(app.list_scroll_offset, 0);
        assert_eq!(app.list_selected_index, 0);
    }

    #[test]
    fn test_update_list_scroll_with_small_list() {
        let stories = create_test_stories(); // 3 stories
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        app.toggle_view_mode();
        
        // With a large visible height (20), all stories should be visible
        app.update_list_scroll(20); // 20 lines visible = 10 stories visible
        assert_eq!(app.list_scroll_offset, 0);
        
        // Navigate to last story
        app.list_selected_index = 2;
        app.update_list_scroll(20);
        assert_eq!(app.list_scroll_offset, 0); // Still no scrolling needed
    }

    #[test]
    fn test_update_list_scroll_with_large_list() {
        let mut stories = create_test_stories();
        // Add more stories to force scrolling
        for i in 4..20 {
            stories.push(Story {
                id: i,
                name: format!("Story {}", i),
                description: format!("Description {}", i),
                workflow_state_id: 10,
                app_url: format!("https://app.shortcut.com/org/story/{}", i),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![format!("user{}", i)],
                position: i as i64 * 1000,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
            });
        }
        
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        app.toggle_view_mode();
        
        // With visible height 6, only 3 stories visible (6 lines / 2 lines per story)
        app.update_list_scroll(6);
        assert_eq!(app.list_scroll_offset, 0);
        
        // Navigate to story index 4 (should trigger scrolling)
        app.list_selected_index = 4;
        app.update_list_scroll(6);
        assert_eq!(app.list_scroll_offset, 2); // Should scroll to keep selected item visible
        
        // Navigate to story index 10
        app.list_selected_index = 10;
        app.update_list_scroll(6);
        assert_eq!(app.list_scroll_offset, 8); // Should scroll further
        
        // Navigate back to beginning
        app.list_selected_index = 0;
        app.update_list_scroll(6);
        assert_eq!(app.list_scroll_offset, 0); // Should scroll back to top
    }

    #[test]
    fn test_scroll_bounds() {
        let mut stories = create_test_stories();
        // Add more stories
        for i in 4..10 {
            stories.push(Story {
                id: i,
                name: format!("Story {}", i),
                description: format!("Description {}", i),
                workflow_state_id: 10,
                app_url: format!("https://app.shortcut.com/org/story/{}", i),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![format!("user{}", i)],
                position: i as i64 * 1000,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
            });
        }
        
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        app.toggle_view_mode();
        
        // With visible height 4, only 2 stories visible
        // Total stories: 9, visible: 2, max scroll offset: 9 - 2 = 7
        app.list_selected_index = 8; // Last story
        app.update_list_scroll(4);
        assert_eq!(app.list_scroll_offset, 7); // Max scroll to show last story
        
        // Try to scroll beyond bounds
        app.list_scroll_offset = 100;
        app.update_list_scroll(4);
        assert_eq!(app.list_scroll_offset, 7); // Should be clamped to max
    }

    #[test]
    fn test_scroll_offset_reset_on_view_toggle() {
        let mut stories = create_test_stories();
        for i in 4..10 {
            stories.push(Story {
                id: i,
                name: format!("Story {}", i),
                description: format!("Description {}", i),
                workflow_state_id: 10,
                app_url: format!("https://app.shortcut.com/org/story/{}", i),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![format!("user{}", i)],
                position: i as i64 * 1000,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
            });
        }
        
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        // Switch to list view and scroll
        app.toggle_view_mode();
        app.list_selected_index = 5;
        app.update_list_scroll(4);
        assert!(app.list_scroll_offset > 0);
        
        // Toggle back to column view
        app.toggle_view_mode();
        assert!(!app.list_view_mode);
        
        // Toggle back to list view - scroll should be reset
        app.toggle_view_mode();
        assert!(app.list_view_mode);
        assert_eq!(app.list_scroll_offset, 0);
        assert_eq!(app.list_selected_index, 0);
    }

    #[test]
    fn test_navigation_with_scrolling() {
        let mut stories = create_test_stories();
        for i in 4..8 {
            stories.push(Story {
                id: i,
                name: format!("Story {}", i),
                description: format!("Description {}", i),
                workflow_state_id: 10,
                app_url: format!("https://app.shortcut.com/org/story/{}", i),
                story_type: "feature".to_string(),
                labels: vec![],
                owner_ids: vec![format!("user{}", i)],
                position: i as i64 * 1000,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-02T00:00:00Z".to_string(),
                completed_at: None,
                moved_at: None,
                comments: vec![],
            });
        }
        
        let workflows = create_test_workflows();
        let mut app = App::new(stories, workflows, "test query".to_string(), None);

        app.toggle_view_mode();
        
        // Navigate through list
        assert_eq!(app.list_selected_index, 0);
        
        app.next();
        assert_eq!(app.list_selected_index, 1);
        
        app.next();
        assert_eq!(app.list_selected_index, 2);
        
        // Navigate to end and wrap around
        for _ in 0..4 {
            app.next();
        }
        assert_eq!(app.list_selected_index, 6); // Last story
        
        app.next(); // Should wrap to beginning
        assert_eq!(app.list_selected_index, 0);
        
        // Navigate backwards
        app.previous(); // Should wrap to end
        assert_eq!(app.list_selected_index, 6);
        
        app.previous();
        assert_eq!(app.list_selected_index, 5);
    }
}