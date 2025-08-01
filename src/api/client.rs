use super::*;
use anyhow::{Context, Result};
use reqwest::blocking::Client;

pub struct ShortcutClient {
    pub(crate) client: Client,
    pub(crate) api_token: String,
    pub(crate) base_url: String,
    pub(crate) debug: bool,
}

impl ShortcutClient {
    pub fn new(api_token: String, debug: bool) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_token,
            base_url: "https://api.app.shortcut.com/api/v3".to_string(),
            debug,
        })
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(token_value) = self.api_token.parse() {
            headers.insert("Shortcut-Token", token_value);
        }
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers
    }
}

impl ShortcutApi for ShortcutClient {
    fn search_stories(&self, query: &str) -> Result<Vec<Story>> {
        let url = format!("{}/search", self.base_url);
        if self.debug {
            eprintln!("Searching with query: {query}");
        }
        
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(&[("query", query)])
            .send()
            .context("Failed to send search request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Response status: {status}");
        }
        
        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("API request failed with status: {}. Error: {}", status, error_text);
        }

        let response_text = response.text().context("Failed to read response text")?;
        if self.debug {
            eprintln!("Response preview: {}", &response_text.chars().take(200).collect::<String>());
        }
        
        let search_response: SearchResponse = serde_json::from_str(&response_text)
            .context("Failed to parse search response")?;

        if self.debug {
            eprintln!("Found {} stories", search_response.stories.data.len());
        }
        Ok(search_response.stories.data)
    }

    fn get_workflows(&self) -> Result<Vec<Workflow>> {
        let url = format!("{}/workflows", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .context("Failed to send workflows request")?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed with status: {}", response.status());
        }

        let workflows: Vec<Workflow> = response
            .json()
            .context("Failed to parse workflows response")?;

        Ok(workflows)
    }

    fn update_story_state(&self, story_id: i64, workflow_state_id: i64) -> Result<Story> {
        let url = format!("{}/stories/{}", self.base_url, story_id);
        
        let update_payload = serde_json::json!({
            "workflow_state_id": workflow_state_id
        });

        if self.debug {
            eprintln!("Updating story {} to workflow state {}", story_id, workflow_state_id);
        }
        
        let response = self
            .client
            .put(&url)
            .headers(self.headers())
            .json(&update_payload)
            .send()
            .context("Failed to send story update request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Update response status: {status}");
        }

        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to update story state: {}. Error: {}", status, error_text);
        }

        let updated_story: Story = response
            .json()
            .context("Failed to parse updated story response")?;

        Ok(updated_story)
    }

    fn get_current_member(&self) -> Result<Member> {
        let url = format!("{}/member", self.base_url);
        
        if self.debug {
            eprintln!("Fetching current member...");
        }
        
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .context("Failed to send member request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Member response status: {status}");
        }

        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to get current member: {}. Error: {}", status, error_text);
        }

        let member: Member = response
            .json()
            .context("Failed to parse member response")?;

        Ok(member)
    }

    fn update_story(&self, story_id: i64, owner_ids: Vec<String>) -> Result<Story> {
        let url = format!("{}/stories/{}", self.base_url, story_id);
        
        let update_payload = serde_json::json!({
            "owner_ids": owner_ids
        });

        if self.debug {
            eprintln!("Updating story {} owners to {:?}", story_id, owner_ids);
        }
        
        let response = self
            .client
            .put(&url)
            .headers(self.headers())
            .json(&update_payload)
            .send()
            .context("Failed to send story update request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Update response status: {status}");
        }

        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to update story owners: {}. Error: {}", status, error_text);
        }

        let updated_story: Story = response
            .json()
            .context("Failed to parse updated story response")?;

        Ok(updated_story)
    }

    fn get_members(&self) -> Result<Vec<Member>> {
        let url = format!("{}/members", self.base_url);
        
        if self.debug {
            eprintln!("Fetching all members...");
        }
        
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .context("Failed to send members request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Members response status: {status}");
        }

        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to get members: {}. Error: {}", status, error_text);
        }

        let members: Vec<Member> = response
            .json()
            .context("Failed to parse members response")?;

        if self.debug {
            eprintln!("Fetched {} members", members.len());
        }

        Ok(members)
    }
}