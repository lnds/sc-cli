#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::api::{ShortcutApi, Story, Workflow, Member, CurrentMember};
    use anyhow::Result;

    struct MockApi {
        should_fail: bool,
        expected_story: Story,
    }

    impl ShortcutApi for MockApi {
        fn search_stories(&self, _query: &str, _limit: Option<usize>) -> Result<Vec<Story>> {
            unimplemented!()
        }

        fn get_workflows(&self) -> Result<Vec<Workflow>> {
            unimplemented!()
        }

        fn update_story_state(&self, _story_id: i64, _workflow_state_id: i64) -> Result<Story> {
            unimplemented!()
        }

        fn get_current_member(&self) -> Result<CurrentMember> {
            unimplemented!()
        }

        fn update_story(&self, _story_id: i64, _owner_ids: Vec<String>) -> Result<Story> {
            unimplemented!()
        }

        fn get_members(&self) -> Result<Vec<Member>> {
            unimplemented!()
        }

        fn create_story(&self, _name: String, _description: String, _story_type: String, _requested_by_id: String, _workflow_state_id: i64) -> Result<Story> {
            if self.should_fail {
                Err(anyhow::anyhow!("API Error"))
            } else {
                Ok(self.expected_story.clone())
            }
        }
    }

    #[test]
    fn test_story_creator_new() {
        let creator = StoryCreator::new(
            "Test Story".to_string(),
            "Test Description".to_string(),
            "feature".to_string(),
            "user-123".to_string(),
            456
        );

        assert_eq!(creator.name, "Test Story");
        assert_eq!(creator.description, "Test Description");
        assert_eq!(creator.story_type, "feature");
        assert_eq!(creator.requested_by_id, "user-123");
    }

    #[test]
    fn test_story_creator_create_success() {
        let mock_story = Story {
            id: 123,
            name: "Test Story".to_string(),
            description: "Test Description".to_string(),
            workflow_state_id: 456,
            app_url: "https://app.shortcut.com/org/story/123".to_string(),
            story_type: "feature".to_string(),
            labels: vec![],
            owner_ids: vec![],
            position: 1000,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            comments: vec![],
        };

        let mock_api = MockApi {
            should_fail: false,
            expected_story: mock_story.clone(),
        };

        let creator = StoryCreator::new(
            "Test Story".to_string(),
            "Test Description".to_string(),
            "feature".to_string(),
            "user-123".to_string(),
            456
        );

        let result = creator.create(&mock_api);
        assert!(result.is_ok());
        
        let created_story = result.unwrap();
        assert_eq!(created_story.id, 123);
        assert_eq!(created_story.name, "Test Story");
    }

    #[test]
    fn test_story_creator_create_failure() {
        let mock_story = Story {
            id: 0,
            name: String::new(),
            description: String::new(),
            workflow_state_id: 0,
            app_url: String::new(),
            story_type: String::new(),
            labels: vec![],
            owner_ids: vec![],
            position: 0,
            created_at: String::new(),
            updated_at: String::new(),
            comments: vec![],
        };

        let mock_api = MockApi {
            should_fail: true,
            expected_story: mock_story,
        };

        let creator = StoryCreator::new(
            "Test Story".to_string(),
            "Test Description".to_string(),
            "feature".to_string(),
            "user-123".to_string(),
            456
        );

        let result = creator.create(&mock_api);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to create story"));
    }
}