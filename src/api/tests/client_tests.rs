use crate::api::{client::ShortcutClient, ShortcutApi};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_test_client(base_url: &str) -> ShortcutClient {
        ShortcutClient {
            client: reqwest::blocking::Client::new(),
            api_token: "test-token".to_string(),
            base_url: base_url.to_string(),
            debug: false,
        }
    }

    #[test]
    fn test_search_stories_success() {
        let mut server = mockito::Server::new();
        let url = server.url();
        
        let mock_response = json!({
            "stories": {
                "data": [
                    {
                        "id": 123,
                        "name": "Test Story",
                        "description": "Test description",
                        "workflow_state_id": 456,
                        "app_url": "https://app.shortcut.com/org/story/123",
                        "story_type": "feature",
                        "labels": [],
                        "owner_ids": ["user-123"],
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-02T00:00:00Z"
                    }
                ]
            }
        });

        let _m = server.mock("GET", "/search")
            .match_query(mockito::Matcher::UrlEncoded("query".to_string(), "owner:test".to_string()))
            .match_header("Shortcut-Token", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = create_test_client(&url);
        let stories = client.search_stories("owner:test").unwrap();

        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].id, 123);
        assert_eq!(stories[0].name, "Test Story");
        assert_eq!(stories[0].description, "Test description");
    }

    #[test]
    fn test_search_stories_empty_results() {
        let mut server = mockito::Server::new();
        let url = server.url();
        
        let mock_response = json!({
            "stories": {
                "data": []
            }
        });

        let _m = server.mock("GET", "/search")
            .match_query(mockito::Matcher::UrlEncoded("query".to_string(), "owner:nobody".to_string()))
            .match_header("Shortcut-Token", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = create_test_client(&url);
        let stories = client.search_stories("owner:nobody").unwrap();

        assert!(stories.is_empty());
    }

    #[test]
    fn test_search_stories_api_error() {
        let mut server = mockito::Server::new();
        let url = server.url();
        
        let _m = server.mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .with_body("Unauthorized")
            .create();

        let client = create_test_client(&url);
        let result = client.search_stories("owner:test");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("401"));
    }

    #[test]
    fn test_get_workflows_success() {
        let mut server = mockito::Server::new();
        let url = server.url();
        
        let mock_response = json!([
            {
                "id": 1,
                "name": "Default Workflow",
                "states": [
                    {"id": 10, "name": "To Do", "color": "#cccccc", "position": 1},
                    {"id": 20, "name": "In Progress", "color": "#0000ff", "position": 2},
                    {"id": 30, "name": "Done", "color": "#00ff00", "position": 3}
                ]
            }
        ]);

        let _m = server.mock("GET", "/workflows")
            .match_header("Shortcut-Token", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = create_test_client(&url);
        let workflows = client.get_workflows().unwrap();

        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].id, 1);
        assert_eq!(workflows[0].name, "Default Workflow");
        assert_eq!(workflows[0].states.len(), 3);
    }


    #[test]
    fn test_debug_mode_output() {
        let mut server = mockito::Server::new();
        let url = server.url();
        
        let mock_response = json!({
            "stories": {
                "data": []
            }
        });

        let _m = server.mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = ShortcutClient {
            client: reqwest::blocking::Client::new(),
            api_token: "test-token".to_string(),
            base_url: url.to_string(),
            debug: true,
        };

        // This test primarily ensures debug mode doesn't crash
        // In a real test environment, we'd capture stderr to verify output
        let _ = client.search_stories("owner:test").unwrap();
    }
}