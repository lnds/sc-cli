use crate::api::{ShortcutApi, Story, Workflow};

use super::{GitContext, GitRepoType};

/// Request to create a git branch or worktree
#[derive(Debug, Clone)]
pub struct GitBranchRequest {
    pub branch_name: String,
    pub worktree_path: String,
    pub operation: GitOperation,
    pub story_id: i64,
}

/// Type of git operation to perform
#[derive(Debug, Clone, PartialEq)]
pub enum GitOperation {
    CreateBranch,
    CreateWorktree,
}

/// Result of a git branch operation
#[derive(Debug, Clone)]
pub struct GitBranchResult {
    pub success: bool,
    pub message: String,
    pub branch_name: String,
    pub worktree_path: Option<String>,
    pub story_id: i64,
    pub operation: GitOperation,
}

impl GitBranchResult {
    fn success(message: String, request: &GitBranchRequest) -> Self {
        Self {
            success: true,
            message,
            branch_name: request.branch_name.clone(),
            worktree_path: if request.operation == GitOperation::CreateWorktree {
                Some(request.worktree_path.clone())
            } else {
                None
            },
            story_id: request.story_id,
            operation: request.operation.clone(),
        }
    }

    fn failure(message: String, request: &GitBranchRequest) -> Self {
        Self {
            success: false,
            message,
            branch_name: request.branch_name.clone(),
            worktree_path: None,
            story_id: request.story_id,
            operation: request.operation.clone(),
        }
    }
}

/// Execute a git branch or worktree creation operation
pub fn execute_git_operation(request: &GitBranchRequest) -> GitBranchResult {
    match &request.operation {
        GitOperation::CreateBranch => execute_create_branch(request),
        GitOperation::CreateWorktree => execute_create_worktree(request),
    }
}

fn execute_create_branch(request: &GitBranchRequest) -> GitBranchResult {
    // Check if branch already exists
    match super::branch_exists(&request.branch_name) {
        Ok(true) => GitBranchResult::failure(
            format!("Branch '{}' already exists", request.branch_name),
            request,
        ),
        Ok(false) => {
            // Create the branch
            match super::create_branch(&request.branch_name) {
                Ok(()) => GitBranchResult::success(
                    format!(
                        "Successfully created and switched to branch '{}'",
                        request.branch_name
                    ),
                    request,
                ),
                Err(e) => GitBranchResult::failure(
                    format!("Failed to create branch '{}': {e}", request.branch_name),
                    request,
                ),
            }
        }
        Err(e) => {
            GitBranchResult::failure(format!("Failed to check if branch exists: {e}"), request)
        }
    }
}

fn execute_create_worktree(request: &GitBranchRequest) -> GitBranchResult {
    match super::create_worktree(&request.branch_name, &request.worktree_path) {
        Ok(()) => GitBranchResult::success(
            format!(
                "Successfully created worktree '{}' at '{}'",
                request.branch_name, request.worktree_path
            ),
            request,
        ),
        Err(e) => GitBranchResult::failure(format!("Failed to create worktree: {e}"), request),
    }
}

/// Find the "In Progress" state ID from workflows
pub fn find_in_progress_state_id(workflows: &[Workflow]) -> Option<i64> {
    workflows
        .iter()
        .flat_map(|w| &w.states)
        .find(|state| {
            state.state_type == "started"
                || state.name.to_lowercase().contains("progress")
                || state.name.to_lowercase().contains("doing")
        })
        .map(|state| state.id)
}

/// Move a story to "In Progress" state after successful git operation
pub fn move_story_to_in_progress<C: ShortcutApi>(
    client: &C,
    story_id: i64,
    workflows: &[Workflow],
    debug: bool,
) -> Option<Story> {
    if story_id <= 0 {
        return None;
    }

    let target_state_id = find_in_progress_state_id(workflows)?;

    match client.update_story_state(story_id, target_state_id) {
        Ok(updated_story) => {
            if debug {
                eprintln!("Moved story {story_id} to In Progress state");
            }
            Some(updated_story)
        }
        Err(e) => {
            if debug {
                eprintln!("Failed to move story to In Progress: {e}");
            }
            None
        }
    }
}

/// Check if git operations are available based on context
#[allow(dead_code)]
pub fn is_git_available(context: &GitContext) -> bool {
    context.repo_type != GitRepoType::NotARepo
}

/// Determine the appropriate operation type based on git context
#[allow(dead_code)]
pub fn default_operation_for_context(context: &GitContext) -> GitOperation {
    if context.is_bare_repo() {
        GitOperation::CreateWorktree
    } else {
        GitOperation::CreateBranch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{
        CurrentMember, Epic, Member, SearchStoriesResult, Story, Workflow, WorkflowState,
    };
    use anyhow::Result;

    // Mock implementation of ShortcutApi for testing
    struct MockShortcutApi {
        update_story_state_result: Result<Story>,
    }

    impl MockShortcutApi {
        fn new_success() -> Self {
            Self {
                update_story_state_result: Ok(create_test_story()),
            }
        }

        fn new_failure() -> Self {
            Self {
                update_story_state_result: Err(anyhow::anyhow!("API error")),
            }
        }
    }

    impl ShortcutApi for MockShortcutApi {
        fn search_stories(&self, _query: &str, _limit: Option<usize>) -> Result<Vec<Story>> {
            Ok(vec![])
        }

        fn search_stories_page(
            &self,
            _query: &str,
            _next_token: Option<String>,
        ) -> Result<SearchStoriesResult> {
            Ok(SearchStoriesResult {
                stories: vec![],
                next_page_token: None,
                total: None,
            })
        }

        fn get_workflows(&self) -> Result<Vec<Workflow>> {
            Ok(vec![])
        }

        fn get_story(&self, _story_id: i64) -> Result<Story> {
            Ok(create_test_story())
        }

        fn update_story_state(&self, _story_id: i64, _workflow_state_id: i64) -> Result<Story> {
            match &self.update_story_state_result {
                Ok(story) => Ok(story.clone()),
                Err(_) => Err(anyhow::anyhow!("API error")),
            }
        }

        fn get_current_member(&self) -> Result<CurrentMember> {
            Ok(CurrentMember {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                mention_name: "testuser".to_string(),
            })
        }

        fn update_story(&self, _story_id: i64, _owner_ids: Vec<String>) -> Result<Story> {
            Ok(create_test_story())
        }

        fn update_story_details(
            &self,
            _story_id: i64,
            _name: String,
            _description: String,
            _story_type: String,
            _epic_id: Option<i64>,
        ) -> Result<Story> {
            Ok(create_test_story())
        }

        fn get_members(&self) -> Result<Vec<Member>> {
            Ok(vec![])
        }

        fn create_story(
            &self,
            _name: String,
            _description: String,
            _story_type: String,
            _requested_by_id: String,
            _workflow_state_id: i64,
            _epic_id: Option<i64>,
        ) -> Result<Story> {
            Ok(create_test_story())
        }

        fn get_epics(&self) -> Result<Vec<Epic>> {
            Ok(vec![])
        }

        fn create_epic(&self, _name: String, _description: String) -> Result<Epic> {
            Ok(Epic {
                id: 1,
                name: "Test Epic".to_string(),
                description: "Description".to_string(),
                app_url: "https://app.shortcut.com/epic/1".to_string(),
                state: "to do".to_string(),
                owner_ids: vec![],
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        }

        fn add_comment(&self, _story_id: i64, _text: &str) -> Result<()> {
            Ok(())
        }
    }

    fn create_test_story() -> Story {
        Story {
            id: 123,
            name: "Test Story".to_string(),
            description: "Description".to_string(),
            workflow_state_id: 1,
            app_url: "https://app.shortcut.com/story/123".to_string(),
            story_type: "feature".to_string(),
            labels: vec![],
            owner_ids: vec![],
            position: 0,
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
        }
    }

    fn create_test_workflows() -> Vec<Workflow> {
        vec![Workflow {
            id: 1,
            name: "Development".to_string(),
            states: vec![
                WorkflowState {
                    id: 100,
                    name: "Backlog".to_string(),
                    color: "#ffffff".to_string(),
                    position: 0,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 101,
                    name: "In Progress".to_string(),
                    color: "#00ff00".to_string(),
                    position: 1,
                    state_type: "started".to_string(),
                },
                WorkflowState {
                    id: 102,
                    name: "Done".to_string(),
                    color: "#0000ff".to_string(),
                    position: 2,
                    state_type: "done".to_string(),
                },
            ],
        }]
    }

    #[test]
    fn test_git_branch_result_success() {
        let request = GitBranchRequest {
            branch_name: "feature/test".to_string(),
            worktree_path: "../feature-test".to_string(),
            operation: GitOperation::CreateBranch,
            story_id: 123,
        };

        let result = GitBranchResult::success("Success!".to_string(), &request);

        assert!(result.success);
        assert_eq!(result.branch_name, "feature/test");
        assert!(result.worktree_path.is_none()); // No worktree for CreateBranch
        assert_eq!(result.story_id, 123);
    }

    #[test]
    fn test_git_branch_result_success_worktree() {
        let request = GitBranchRequest {
            branch_name: "feature/test".to_string(),
            worktree_path: "../feature-test".to_string(),
            operation: GitOperation::CreateWorktree,
            story_id: 123,
        };

        let result = GitBranchResult::success("Success!".to_string(), &request);

        assert!(result.success);
        assert_eq!(result.worktree_path, Some("../feature-test".to_string()));
    }

    #[test]
    fn test_git_branch_result_failure() {
        let request = GitBranchRequest {
            branch_name: "feature/test".to_string(),
            worktree_path: "../feature-test".to_string(),
            operation: GitOperation::CreateBranch,
            story_id: 123,
        };

        let result = GitBranchResult::failure("Error!".to_string(), &request);

        assert!(!result.success);
        assert!(result.worktree_path.is_none());
    }

    #[test]
    fn test_default_operation_for_normal_repo() {
        let context = GitContext {
            repo_type: GitRepoType::Normal,
            current_branch: Some("main".to_string()),
        };

        assert_eq!(
            default_operation_for_context(&context),
            GitOperation::CreateBranch
        );
    }

    #[test]
    fn test_default_operation_for_bare_repo() {
        let context = GitContext {
            repo_type: GitRepoType::Bare,
            current_branch: None,
        };

        assert_eq!(
            default_operation_for_context(&context),
            GitOperation::CreateWorktree
        );
    }

    #[test]
    fn test_is_git_available_normal_repo() {
        let context = GitContext {
            repo_type: GitRepoType::Normal,
            current_branch: Some("main".to_string()),
        };

        assert!(is_git_available(&context));
    }

    #[test]
    fn test_is_git_available_bare_repo() {
        let context = GitContext {
            repo_type: GitRepoType::Bare,
            current_branch: None,
        };

        assert!(is_git_available(&context));
    }

    #[test]
    fn test_is_git_available_not_a_repo() {
        let context = GitContext {
            repo_type: GitRepoType::NotARepo,
            current_branch: None,
        };

        assert!(!is_git_available(&context));
    }

    #[test]
    fn test_find_in_progress_state_id_by_started_type() {
        let workflows = create_test_workflows();
        let result = find_in_progress_state_id(&workflows);

        assert_eq!(result, Some(101)); // "In Progress" state has id 101
    }

    #[test]
    fn test_find_in_progress_state_id_by_progress_name() {
        let workflows = vec![Workflow {
            id: 1,
            name: "Development".to_string(),
            states: vec![
                WorkflowState {
                    id: 100,
                    name: "Backlog".to_string(),
                    color: "#ffffff".to_string(),
                    position: 0,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 101,
                    name: "Work In Progress".to_string(),
                    color: "#00ff00".to_string(),
                    position: 1,
                    state_type: "active".to_string(), // Not "started"
                },
            ],
        }];

        let result = find_in_progress_state_id(&workflows);
        assert_eq!(result, Some(101)); // Found by name containing "progress"
    }

    #[test]
    fn test_find_in_progress_state_id_by_doing_name() {
        let workflows = vec![Workflow {
            id: 1,
            name: "Development".to_string(),
            states: vec![
                WorkflowState {
                    id: 100,
                    name: "Backlog".to_string(),
                    color: "#ffffff".to_string(),
                    position: 0,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 101,
                    name: "Doing".to_string(),
                    color: "#00ff00".to_string(),
                    position: 1,
                    state_type: "active".to_string(),
                },
            ],
        }];

        let result = find_in_progress_state_id(&workflows);
        assert_eq!(result, Some(101)); // Found by name containing "doing"
    }

    #[test]
    fn test_find_in_progress_state_id_not_found() {
        let workflows = vec![Workflow {
            id: 1,
            name: "Development".to_string(),
            states: vec![
                WorkflowState {
                    id: 100,
                    name: "Backlog".to_string(),
                    color: "#ffffff".to_string(),
                    position: 0,
                    state_type: "unstarted".to_string(),
                },
                WorkflowState {
                    id: 102,
                    name: "Done".to_string(),
                    color: "#0000ff".to_string(),
                    position: 2,
                    state_type: "done".to_string(),
                },
            ],
        }];

        let result = find_in_progress_state_id(&workflows);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_in_progress_state_id_empty_workflows() {
        let workflows: Vec<Workflow> = vec![];
        let result = find_in_progress_state_id(&workflows);
        assert_eq!(result, None);
    }

    #[test]
    fn test_move_story_to_in_progress_success() {
        let client = MockShortcutApi::new_success();
        let workflows = create_test_workflows();

        let result = move_story_to_in_progress(&client, 123, &workflows, false);

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 123);
    }

    #[test]
    fn test_move_story_to_in_progress_invalid_story_id_zero() {
        let client = MockShortcutApi::new_success();
        let workflows = create_test_workflows();

        let result = move_story_to_in_progress(&client, 0, &workflows, false);

        assert!(result.is_none());
    }

    #[test]
    fn test_move_story_to_in_progress_invalid_story_id_negative() {
        let client = MockShortcutApi::new_success();
        let workflows = create_test_workflows();

        let result = move_story_to_in_progress(&client, -1, &workflows, false);

        assert!(result.is_none());
    }

    #[test]
    fn test_move_story_to_in_progress_no_target_state() {
        let client = MockShortcutApi::new_success();
        let workflows: Vec<Workflow> = vec![]; // No workflows = no target state

        let result = move_story_to_in_progress(&client, 123, &workflows, false);

        assert!(result.is_none());
    }

    #[test]
    fn test_move_story_to_in_progress_api_error() {
        let client = MockShortcutApi::new_failure();
        let workflows = create_test_workflows();

        let result = move_story_to_in_progress(&client, 123, &workflows, false);

        assert!(result.is_none());
    }
}
