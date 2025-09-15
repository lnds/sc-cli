use crate::api::{ShortcutApi, Story};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use std::io::{self, BufRead};

#[cfg(test)]
mod tests;

pub struct StoryEditor {
    pub story_id: i64,
    pub name: String,
    pub description: String,
    pub story_type: String,
}

impl StoryEditor {
    /// Create a new StoryEditor from an existing story
    pub fn from_story(story: &Story) -> Self {
        Self {
            story_id: story.id,
            name: story.name.clone(),
            description: story.description.clone(),
            story_type: story.story_type.clone(),
        }
    }

    /// Interactive prompt to edit story details with pre-filled current values
    pub fn edit_with_prompts(&mut self) -> Result<bool> {
        println!("\n🔧 Editing Story #{}", self.story_id);
        println!("Press Enter to keep current values, or type new values to change them.\n");

        // Edit story name
        let new_name: String = Input::new()
            .with_prompt("Story name")
            .with_initial_text(&self.name)
            .interact_text()
            .context("Failed to read story name")?;

        // Edit description with multi-line support
        println!("\nCurrent description:");
        if self.description.is_empty() {
            println!("  (no description)");
        } else {
            for line in self.description.lines() {
                println!("  {line}");
            }
        }

        let edit_description = Confirm::new()
            .with_prompt("Do you want to edit the description?")
            .default(false)
            .interact()
            .context("Failed to read confirmation")?;

        let new_description = if edit_description {
            println!("\nEnter new description (press Enter twice to finish):");
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

            description_lines.join("\n")
        } else {
            self.description.clone()
        };

        // Edit story type
        let story_types = vec!["feature", "bug", "chore"];
        let current_type_index = story_types
            .iter()
            .position(|&t| t == self.story_type)
            .unwrap_or(0);

        let story_type_index = Select::new()
            .with_prompt("Story type")
            .items(&story_types)
            .default(current_type_index)
            .interact()
            .context("Failed to read story type")?;

        let new_story_type = story_types[story_type_index].to_string();

        // Check if anything changed
        let changed = new_name != self.name
            || new_description != self.description
            || new_story_type != self.story_type;

        if !changed {
            println!("\n📝 No changes made to the story.");
            return Ok(false);
        }

        // Update the struct with new values
        self.name = new_name;
        self.description = new_description;
        self.story_type = new_story_type;

        // Show summary of changes
        println!("\n📋 Summary of changes:");
        println!("  Name: {}", self.name);
        println!("  Type: {}", self.story_type);
        if self.description.is_empty() {
            println!("  Description: (empty)");
        } else {
            let first_line = self.description.lines().next().unwrap_or("");
            println!(
                "  Description: {}...",
                if first_line.len() > 50 {
                    &first_line[..50]
                } else {
                    first_line
                }
            );
        }

        // Confirm changes
        let confirm = Confirm::new()
            .with_prompt("Save these changes?")
            .default(true)
            .interact()
            .context("Failed to read confirmation")?;

        Ok(confirm)
    }

    /// Update the story using the API client
    pub fn update<T: ShortcutApi>(&self, client: &T) -> Result<Story> {
        client
            .update_story_details(
                self.story_id,
                self.name.clone(),
                self.description.clone(),
                self.story_type.clone(),
                None, // Epic ID not supported in CLI story editor yet
            )
            .context("Failed to update story")
    }
}
