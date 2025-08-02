use super::*;
use crate::api::Story;

#[test]
fn test_story_editor_from_story() {
    let story = Story {
        id: 123,
        name: "Test Story".to_string(),
        description: "Test description".to_string(),
        workflow_state_id: 1,
        app_url: "https://example.com".to_string(),
        story_type: "feature".to_string(),
        labels: vec![],
        owner_ids: vec![],
        position: 1,
        created_at: "2023-01-01T00:00:00Z".to_string(),
        updated_at: "2023-01-01T00:00:00Z".to_string(),
        completed_at: None,
        moved_at: None,
        comments: vec![],
    };

    let editor = StoryEditor::from_story(&story);

    assert_eq!(editor.story_id, 123);
    assert_eq!(editor.name, "Test Story");
    assert_eq!(editor.description, "Test description");
    assert_eq!(editor.story_type, "feature");
}