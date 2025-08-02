use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod client;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub workflow_state_id: i64,
    pub app_url: String,
    #[serde(default)]
    pub story_type: String,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub owner_ids: Vec<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub text: String,
    pub author_id: String,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: i64,
    pub name: String,
    pub states: Vec<WorkflowState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub color: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub stories: StoriesData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoriesData {
    pub data: Vec<Story>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub profile: MemberProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberProfile {
    pub name: String,
    pub mention_name: String,
}

// Structure for the /member endpoint (current user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMember {
    pub id: String,
    pub name: String,
    pub mention_name: String,
}

pub trait ShortcutApi {
    fn search_stories(&self, query: &str, limit: Option<usize>) -> Result<Vec<Story>>;
    fn get_workflows(&self) -> Result<Vec<Workflow>>;
    fn update_story_state(&self, story_id: i64, workflow_state_id: i64) -> Result<Story>;
    fn get_current_member(&self) -> Result<CurrentMember>;
    fn update_story(&self, story_id: i64, owner_ids: Vec<String>) -> Result<Story>;
    fn get_members(&self) -> Result<Vec<Member>>;
    fn create_story(&self, name: String, description: String, story_type: String, requested_by_id: String, workflow_state_id: i64) -> Result<Story>;
}