use crate::api::*;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_story_deserialization() {
        let json_data = json!({
            "id": 123,
            "name": "Test Story",
            "description": "A test story description",
            "workflow_state_id": 456,
            "app_url": "https://app.shortcut.com/org/story/123",
            "story_type": "feature",
            "labels": [
                {"id": 1, "name": "backend", "color": "#ff0000"}
            ],
            "owner_ids": ["user-123"],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let story: Story = serde_json::from_value(json_data).unwrap();
        
        assert_eq!(story.id, 123);
        assert_eq!(story.name, "Test Story");
        assert_eq!(story.description, "A test story description");
        assert_eq!(story.workflow_state_id, 456);
        assert_eq!(story.app_url, "https://app.shortcut.com/org/story/123");
        assert_eq!(story.story_type, "feature");
        assert_eq!(story.labels.len(), 1);
        assert_eq!(story.labels[0].name, "backend");
        assert_eq!(story.owner_ids, vec!["user-123"]);
    }

    #[test]
    fn test_story_deserialization_with_defaults() {
        let json_data = json!({
            "id": 123,
            "name": "Minimal Story",
            "workflow_state_id": 456,
            "app_url": "https://app.shortcut.com/org/story/123",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let story: Story = serde_json::from_value(json_data).unwrap();
        
        assert_eq!(story.id, 123);
        assert_eq!(story.name, "Minimal Story");
        assert_eq!(story.description, "");
        assert_eq!(story.story_type, "");
        assert!(story.labels.is_empty());
        assert!(story.owner_ids.is_empty());
    }

    #[test]
    fn test_workflow_deserialization() {
        let json_data = json!({
            "id": 1,
            "name": "Default Workflow",
            "states": [
                {"id": 10, "name": "To Do", "color": "#cccccc", "position": 1},
                {"id": 20, "name": "In Progress", "color": "#0000ff", "position": 2},
                {"id": 30, "name": "Done", "color": "#00ff00", "position": 3}
            ]
        });

        let workflow: Workflow = serde_json::from_value(json_data).unwrap();
        
        assert_eq!(workflow.id, 1);
        assert_eq!(workflow.name, "Default Workflow");
        assert_eq!(workflow.states.len(), 3);
        assert_eq!(workflow.states[0].name, "To Do");
        assert_eq!(workflow.states[0].position, 1);
        assert_eq!(workflow.states[1].name, "In Progress");
        assert_eq!(workflow.states[1].position, 2);
        assert_eq!(workflow.states[2].name, "Done");
        assert_eq!(workflow.states[2].position, 3);
    }

    #[test]
    fn test_search_response_deserialization() {
        let json_data = json!({
            "stories": {
                "data": [
                    {
                        "id": 1,
                        "name": "Story 1",
                        "workflow_state_id": 10,
                        "app_url": "https://app.shortcut.com/org/story/1",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-02T00:00:00Z"
                    },
                    {
                        "id": 2,
                        "name": "Story 2",
                        "workflow_state_id": 20,
                        "app_url": "https://app.shortcut.com/org/story/2",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-02T00:00:00Z"
                    }
                ]
            }
        });

        let response: SearchResponse = serde_json::from_value(json_data).unwrap();
        
        assert_eq!(response.stories.data.len(), 2);
        assert_eq!(response.stories.data[0].id, 1);
        assert_eq!(response.stories.data[0].name, "Story 1");
        assert_eq!(response.stories.data[1].id, 2);
        assert_eq!(response.stories.data[1].name, "Story 2");
    }
}