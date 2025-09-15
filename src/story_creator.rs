use crate::api::{ShortcutApi, Story};
use anyhow::{Context, Result};
use dialoguer::{Input, Select};
use std::io::{self, BufRead};

#[cfg(test)]
mod tests;

pub struct StoryCreator {
    pub name: String,
    pub description: String,
    pub story_type: String,
    pub requested_by_id: String,
    pub workflow_state_id: i64,
}

impl StoryCreator {
    /// Interactive prompt to create a new story with optional pre-filled values
    pub fn from_prompts(
        requested_by_id: String,
        workflow_state_id: i64,
        provided_name: Option<String>,
        provided_type: Option<String>,
    ) -> Result<Self> {
        // Story name - use provided or prompt
        let name = if let Some(name) = provided_name {
            name
        } else {
            Input::new()
                .with_prompt("Enter story name (short description)")
                .interact_text()?
        };

        // Multi-line description
        println!("Enter story description (press Enter twice to finish)");
        let mut description_lines = Vec::new();
        let mut empty_line_count = 0;

        let stdin = io::stdin();
        let mut handle = stdin.lock();

        loop {
            let mut line = String::new();
            handle.read_line(&mut line).context("Failed to read line")?;

            // Remove the newline character
            let line = line
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();

            if line.is_empty() {
                empty_line_count += 1;
                if empty_line_count >= 2 {
                    break;
                }
                description_lines.push(String::new());
            } else {
                empty_line_count = 0;
                description_lines.push(line);
            }
        }

        // Remove trailing empty lines
        while description_lines.last() == Some(&String::new()) {
            description_lines.pop();
        }

        let description = description_lines.join("\n");

        // Story type - use provided or prompt
        let story_type = if let Some(story_type) = provided_type {
            story_type
        } else {
            let story_types = vec!["feature", "bug", "chore"];
            let story_type_index = Select::new()
                .with_prompt("Select story type")
                .items(&story_types)
                .default(0)
                .interact()?;

            story_types[story_type_index].to_string()
        };

        Ok(Self {
            name,
            description,
            story_type,
            requested_by_id,
            workflow_state_id,
        })
    }

    /// Create a new StoryCreator with provided values (for TUI usage)
    #[allow(dead_code)]
    pub fn new(
        name: String,
        description: String,
        story_type: String,
        requested_by_id: String,
        workflow_state_id: i64,
    ) -> Self {
        Self {
            name,
            description,
            story_type,
            requested_by_id,
            workflow_state_id,
        }
    }

    /// Create the story using the API client
    pub fn create<T: ShortcutApi>(&self, client: &T) -> Result<Story> {
        client
            .create_story(
                self.name.clone(),
                self.description.clone(),
                self.story_type.clone(),
                self.requested_by_id.clone(),
                self.workflow_state_id,
                None, // Epic ID not supported in CLI story creator yet
            )
            .context("Failed to create story")
    }
}
