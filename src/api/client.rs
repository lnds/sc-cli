use super::*;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::collections::HashMap;

pub struct ShortcutClient {
    client: Client,
    api_token: String,
    base_url: String,
    debug: bool,
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
            eprintln!("Searching with query: {}", query);
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
            eprintln!("Response status: {}", status);
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

    fn get_workflow_state_map(&self) -> Result<HashMap<i64, String>> {
        let workflows = self.get_workflows()?;
        let mut state_map = HashMap::new();

        for workflow in workflows {
            for state in workflow.states {
                state_map.insert(state.id, state.name);
            }
        }

        Ok(state_map)
    }
}