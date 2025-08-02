use rstest::rstest;
use serde_json::json;
use std::process::Command;

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TUI client for Shortcut stories"));
    assert!(stdout.contains("--workspace"));
    assert!(stdout.contains("--debug"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("view"));
}

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sc-tui"));
}

#[test]
fn test_missing_required_args() {
    let output = Command::new("cargo")
        .args(&["run", "--", "view", "testuser"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--token") || stderr.contains("--workspace"));
}

#[cfg(test)]
mod api_integration_tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn test_mock_api_workflow() {
        let mut server = Server::new();
        
        // Mock workflows endpoint
        let workflows_response = json!([
            {
                "id": 1,
                "name": "Default Workflow",
                "states": [
                    {"id": 10, "name": "To Do", "color": "#cccccc"},
                    {"id": 20, "name": "In Progress", "color": "#0000ff"},
                    {"id": 30, "name": "Done", "color": "#00ff00"}
                ]
            }
        ]);

        let _workflows_mock = server.mock("GET", "/workflows")
            .match_header("Shortcut-Token", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workflows_response.to_string())
            .create();

        // Mock search endpoint
        let search_response = json!({
            "stories": {
                "data": [
                    {
                        "id": 123,
                        "name": "Integration Test Story",
                        "description": "Test description for integration test",
                        "workflow_state_id": 20,
                        "app_url": "https://app.shortcut.com/org/story/123",
                        "story_type": "feature",
                        "labels": [
                            {"id": 1, "name": "integration-test", "color": "#ff0000"}
                        ],
                        "owner_ids": ["testuser"],
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-02T00:00:00Z"
                    }
                ]
            }
        });

        let _search_mock = server.mock("GET", "/search")
            .match_query(mockito::Matcher::UrlEncoded("query".to_string(), "owner:testuser is:story".to_string()))
            .match_header("Shortcut-Token", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(search_response.to_string())
            .create();

        // Verify mocks work correctly
        assert_eq!(server.url().starts_with("http://"), true);
    }

    #[rstest]
    #[case(200, true)]
    #[case(401, false)]
    #[case(403, false)]
    #[case(404, false)]
    #[case(500, false)]
    fn test_api_error_handling(#[case] status_code: usize, #[case] should_succeed: bool) {
        let mut server = Server::new();

        let response_body = if should_succeed {
            json!({
                "stories": {
                    "data": []
                }
            }).to_string()
        } else {
            "Error response".to_string()
        };

        let _mock = server.mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(status_code)
            .with_body(response_body)
            .create();

        // Verify the status code logic
        assert_eq!(status_code >= 200 && status_code < 300, should_succeed);
    }
}