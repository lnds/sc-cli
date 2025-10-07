use super::*;
use super::{CurrentMember, Epic};
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
    fn search_stories(&self, query: &str, limit: Option<usize>) -> Result<Vec<Story>> {
        let url = format!("{}/search", self.base_url);
        let mut all_stories = Vec::new();
        let page_size = 25; // Maximum allowed by Shortcut API
        let mut next_token: Option<String> = None;

        if self.debug {
            eprintln!("Searching with query: {query}");
            if let Some(l) = limit {
                eprintln!("Limit: {l}");
            }
        }

        loop {
            // Build query parameters
            let mut params = vec![
                ("query", query.to_string()),
                ("page_size", page_size.to_string()),
            ];
            if let Some(ref token) = next_token {
                params.push(("next", token.clone()));
            }

            let response = self
                .client
                .get(&url)
                .headers(self.headers())
                .query(&params)
                .send()
                .context("Failed to send search request")?;

            let status = response.status();
            if self.debug {
                eprintln!("Response status: {status}");
            }

            if !status.is_success() {
                let error_text = response
                    .text()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                anyhow::bail!(
                    "API request failed with status: {}. Error: {}",
                    status,
                    error_text
                );
            }

            let response_text = response.text().context("Failed to read response text")?;
            if self.debug && next_token.is_none() {
                eprintln!(
                    "Response preview: {}",
                    &response_text.chars().take(500).collect::<String>()
                );
            }

            let search_response: SearchResponse =
                serde_json::from_str(&response_text).context("Failed to parse search response")?;

            let stories_count = search_response.stories.data.len();
            if self.debug {
                eprintln!("Found {stories_count} stories in this page");
                if let Some(total) = search_response.stories.total {
                    eprintln!("Total available stories: {total}");
                }
            }

            all_stories.extend(search_response.stories.data);

            // Check if we have enough stories
            if let Some(l) = limit
                && all_stories.len() >= l
            {
                all_stories.truncate(l);
                break;
            }

            // Check if we have a next page
            next_token = search_response.next.or(search_response.stories.next);

            if next_token.is_none() || stories_count == 0 {
                break;
            }
        }

        if self.debug {
            eprintln!("Total stories fetched: {}", all_stories.len());
        }

        Ok(all_stories)
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

    fn get_story(&self, story_id: i64) -> Result<Story> {
        let url = format!("{}/stories/{}", self.base_url, story_id);

        if self.debug {
            eprintln!("Fetching story #{story_id}...");
        }

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .context("Failed to send story request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Story response status: {status}");
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            if status.as_u16() == 404 {
                anyhow::bail!("Story #{story_id} not found");
            } else {
                anyhow::bail!("Failed to get story: {}. Error: {}", status, error_text);
            }
        }

        let story: Story = response.json().context("Failed to parse story response")?;

        if self.debug {
            eprintln!("Successfully fetched story #{} - {}", story.id, story.name);
        }

        Ok(story)
    }

    fn update_story_state(&self, story_id: i64, workflow_state_id: i64) -> Result<Story> {
        let url = format!("{}/stories/{}", self.base_url, story_id);

        let update_payload = serde_json::json!({
            "workflow_state_id": workflow_state_id
        });

        if self.debug {
            eprintln!("Updating story {story_id} to workflow state {workflow_state_id}");
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
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to update story state: {}. Error: {}",
                status,
                error_text
            );
        }

        let updated_story: Story = response
            .json()
            .context("Failed to parse updated story response")?;

        Ok(updated_story)
    }

    fn get_current_member(&self) -> Result<CurrentMember> {
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
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to get current member: {}. Error: {}",
                status,
                error_text
            );
        }

        let member: CurrentMember = response.json().context("Failed to parse member response")?;

        Ok(member)
    }

    fn update_story(&self, story_id: i64, owner_ids: Vec<String>) -> Result<Story> {
        let url = format!("{}/stories/{}", self.base_url, story_id);

        let update_payload = serde_json::json!({
            "owner_ids": owner_ids
        });

        if self.debug {
            eprintln!("Updating story {story_id} owners to {owner_ids:?}");
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
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to update story owners: {}. Error: {}",
                status,
                error_text
            );
        }

        let updated_story: Story = response
            .json()
            .context("Failed to parse updated story response")?;

        Ok(updated_story)
    }

    fn update_story_details(
        &self,
        story_id: i64,
        name: String,
        description: String,
        story_type: String,
        epic_id: Option<i64>,
    ) -> Result<Story> {
        let url = format!("{}/stories/{}", self.base_url, story_id);

        let mut update_payload = serde_json::json!({
            "name": name,
            "description": description,
            "story_type": story_type
        });

        // Add epic_id if provided (null to unset)
        if let Some(payload_obj) = update_payload.as_object_mut() {
            payload_obj.insert(
                "epic_id".to_string(),
                epic_id
                    .map(|id| serde_json::json!(id))
                    .unwrap_or(serde_json::Value::Null),
            );
        }

        if self.debug {
            eprintln!(
                "Updating story {story_id} details: name='{name}', description='{description}', type='{story_type}', epic_id={:?}",
                epic_id
            );
        }

        let response = self
            .client
            .put(&url)
            .headers(self.headers())
            .json(&update_payload)
            .send()
            .context("Failed to send story details update request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Update story details response status: {status}");
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to update story details: {}. Error: {}",
                status,
                error_text
            );
        }

        let updated_story: Story = response
            .json()
            .context("Failed to parse updated story response")?;

        if self.debug {
            eprintln!(
                "Successfully updated story #{} - {}",
                updated_story.id, updated_story.name
            );
        }

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
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to get members: {}. Error: {}", status, error_text);
        }

        let response_text = response.text().context("Failed to read members response")?;

        if self.debug {
            eprintln!(
                "Members response preview: {}",
                &response_text.chars().take(500).collect::<String>()
            );
        }

        let members: Vec<Member> =
            serde_json::from_str(&response_text).context("Failed to parse members response")?;

        if self.debug {
            eprintln!("Fetched {} members", members.len());
        }

        Ok(members)
    }

    fn create_story(
        &self,
        name: String,
        description: String,
        story_type: String,
        requested_by_id: String,
        workflow_state_id: i64,
        epic_id: Option<i64>,
    ) -> Result<Story> {
        let url = format!("{}/stories", self.base_url);

        let mut create_payload = serde_json::json!({
            "name": name,
            "description": description,
            "story_type": story_type,
            "requested_by_id": requested_by_id,
            "workflow_state_id": workflow_state_id
        });

        // Add epic_id if provided
        if let Some(id) = epic_id
            && let Some(payload_obj) = create_payload.as_object_mut()
        {
            payload_obj.insert("epic_id".to_string(), serde_json::json!(id));
        }

        if self.debug {
            eprintln!(
                "Creating story with payload: {}",
                serde_json::to_string_pretty(&create_payload)?
            );
        }

        let response = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&create_payload)
            .send()
            .context("Failed to send story creation request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Create story response status: {status}");
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to create story: {}. Error: {}", status, error_text);
        }

        let created_story: Story = response
            .json()
            .context("Failed to parse created story response")?;

        if self.debug {
            eprintln!(
                "Successfully created story #{} - {}",
                created_story.id, created_story.name
            );
        }

        Ok(created_story)
    }

    fn search_stories_page(
        &self,
        query: &str,
        next_token: Option<String>,
    ) -> Result<super::SearchStoriesResult> {
        let url = format!("{}/search", self.base_url);
        let page_size = 25; // Maximum allowed by Shortcut API

        if self.debug {
            eprintln!("Searching single page with query: {query}");
            if let Some(ref token) = next_token {
                eprintln!("Using next token: {token}");
            }
        }

        // Build query parameters
        let mut params = vec![
            ("query", query.to_string()),
            ("page_size", page_size.to_string()),
        ];
        if let Some(ref token) = next_token {
            params.push(("next", token.clone()));
        }

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(&params)
            .send()
            .context("Failed to send search request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Response status: {status}");
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "API request failed with status: {}. Error: {}",
                status,
                error_text
            );
        }

        let response_text = response.text().context("Failed to read response text")?;
        if self.debug {
            eprintln!(
                "Response preview: {}",
                &response_text.chars().take(500).collect::<String>()
            );
        }

        let search_response: super::SearchResponse =
            serde_json::from_str(&response_text).context("Failed to parse search response")?;

        let stories_count = search_response.stories.data.len();
        if self.debug {
            eprintln!("Found {stories_count} stories in this page");
            if let Some(total) = search_response.stories.total {
                eprintln!("Total available stories: {total}");
            }
        }

        // Get next page token
        let next_page_token = search_response.next.or(search_response.stories.next);

        Ok(super::SearchStoriesResult {
            stories: search_response.stories.data,
            next_page_token,
            total: search_response.stories.total,
        })
    }

    fn get_epics(&self) -> Result<Vec<Epic>> {
        let url = format!("{}/epics", self.base_url);

        if self.debug {
            eprintln!("Fetching epics...");
        }

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .context("Failed to send epics request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Epics response status: {status}");
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to get epics: {}. Error: {}", status, error_text);
        }

        let epics: Vec<Epic> = response.json().context("Failed to parse epics response")?;

        if self.debug {
            eprintln!("Successfully fetched {} epics", epics.len());
        }

        Ok(epics)
    }

    fn create_epic(&self, name: String, description: String) -> Result<Epic> {
        let url = format!("{}/epics", self.base_url);

        #[derive(Serialize, Debug)]
        struct CreateEpicRequest {
            name: String,
            description: String,
        }

        let request_body = CreateEpicRequest { name, description };

        if self.debug {
            eprintln!("Creating epic: {:?}", request_body);
        }

        let response = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&request_body)
            .send()
            .context("Failed to send create epic request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Create epic response status: {status}");
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to create epic: {}. Error: {}", status, error_text);
        }

        let epic: Epic = response.json().context("Failed to parse epic response")?;

        if self.debug {
            eprintln!("Successfully created epic: {}", epic.name);
        }

        Ok(epic)
    }

    fn add_comment(&self, story_id: i64, text: &str) -> Result<()> {
        let url = format!("{}/stories/{}/comments", self.base_url, story_id);

        #[derive(Serialize, Debug)]
        struct AddCommentRequest {
            text: String,
        }

        let request_body = AddCommentRequest {
            text: text.to_string(),
        };

        if self.debug {
            eprintln!("Adding comment to story #{}: {} chars", story_id, text.len());
        }

        let response = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&request_body)
            .send()
            .context("Failed to send comment request")?;

        let status = response.status();
        if self.debug {
            eprintln!("Add comment response status: {status}");
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to add comment: {}. Error: {}", status, error_text);
        }

        if self.debug {
            eprintln!("Successfully added comment to story #{}", story_id);
        }

        Ok(())
    }
}
