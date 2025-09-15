use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workspaces: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_workspace: Option<String>,
    #[serde(flatten)]
    pub workspace_configs: HashMap<String, WorkspaceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub api_key: String,
    pub user_id: String,
    #[serde(default = "default_fetch_limit")]
    pub fetch_limit: usize,
}

fn default_fetch_limit() -> usize {
    50
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::find_config_path()?;

        if !config_path.exists() {
            anyhow::bail!(
                "Config file not found. Create a config.toml file with your workspace settings or use --workspace to create one interactively."
            );
        }

        let contents = fs::read_to_string(&config_path).context(format!(
            "Failed to read config file at: {}",
            config_path.display()
        ))?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file. Make sure it's valid TOML.")?;

        Ok(config)
    }

    pub fn load_or_create(workspace_name: &str) -> Result<(Self, bool)> {
        match Self::load() {
            Ok(mut config) => {
                // Check if workspace exists
                if config.workspace_configs.contains_key(workspace_name) {
                    Ok((config, false))
                } else {
                    // Ask if user wants to add this workspace
                    if Confirm::new()
                        .with_prompt(format!(
                            "Workspace '{workspace_name}' not found. Would you like to create it?"
                        ))
                        .default(true)
                        .interact()?
                    {
                        let workspace_config = Self::prompt_for_workspace_config()?;
                        config.add_workspace(workspace_name, workspace_config)?;
                        Ok((config, true))
                    } else {
                        anyhow::bail!(
                            "Workspace '{}' not found and creation cancelled",
                            workspace_name
                        );
                    }
                }
            }
            Err(_) => {
                // No config file exists, create new one
                if Confirm::new()
                    .with_prompt("No configuration file found. Would you like to create one?")
                    .default(true)
                    .interact()?
                {
                    let config = Self::create_with_workspace(workspace_name)?;
                    Ok((config, true))
                } else {
                    anyhow::bail!("Configuration required to use workspace feature");
                }
            }
        }
    }

    fn create_with_workspace(workspace_name: &str) -> Result<Self> {
        let config_path = Self::prompt_for_config_location()?;

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let workspace_config = Self::prompt_for_workspace_config()?;

        let mut workspace_configs = HashMap::new();
        workspace_configs.insert(workspace_name.to_string(), workspace_config);

        let config = Config {
            workspaces: vec![workspace_name.to_string()],
            default_workspace: Some(workspace_name.to_string()),
            workspace_configs,
        };

        config.save(&config_path)?;
        println!("Configuration saved to: {}", config_path.display());

        Ok(config)
    }

    fn prompt_for_workspace_config() -> Result<WorkspaceConfig> {
        let api_key: String = Input::new()
            .with_prompt("Enter your Shortcut API key")
            .interact_text()?;

        let user_id: String = Input::new()
            .with_prompt("Enter your Shortcut mention name")
            .interact_text()?;

        let fetch_limit: usize = Input::new()
            .with_prompt("Enter the default number of stories to fetch")
            .default(50)
            .interact_text()?;

        Ok(WorkspaceConfig {
            api_key,
            user_id,
            fetch_limit,
        })
    }

    fn prompt_for_config_location() -> Result<PathBuf> {
        let default_path = Self::default_config_path()?;
        let default_str = default_path.to_string_lossy();

        let choices = vec![
            format!("Default location: {}", default_str),
            "Current directory: ./config.toml".to_string(),
            "Custom location".to_string(),
        ];

        let selection = Select::new()
            .with_prompt("Where would you like to save the configuration?")
            .items(&choices)
            .default(0)
            .interact()?;

        match selection {
            0 => Ok(default_path),
            1 => Ok(PathBuf::from("config.toml")),
            2 => {
                let custom_path: String = Input::new()
                    .with_prompt("Enter the full path for the config file")
                    .default(default_str.to_string())
                    .interact_text()?;
                Ok(PathBuf::from(custom_path))
            }
            _ => unreachable!(),
        }
    }

    pub fn add_workspace(&mut self, name: &str, config: WorkspaceConfig) -> Result<()> {
        if !self.workspaces.contains(&name.to_string()) {
            self.workspaces.push(name.to_string());
        }
        self.workspace_configs.insert(name.to_string(), config);

        // If this is the first workspace and no default is set, make it default
        if self.workspaces.len() == 1 && self.default_workspace.is_none() {
            self.default_workspace = Some(name.to_string());
        }

        // Save the updated config
        let config_path = Self::find_config_path()?;
        self.save(&config_path)?;

        Ok(())
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let toml_string =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(path, toml_string).context("Failed to write config file")?;

        Ok(())
    }

    pub fn get_workspace(&self, name: &str) -> Result<&WorkspaceConfig> {
        self.workspace_configs
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Workspace '{}' not found in config", name))
    }

    pub fn get_default_workspace(&self) -> Option<String> {
        // If default_workspace is explicitly set, use it
        if let Some(ref default) = self.default_workspace
            && self.workspace_configs.contains_key(default)
        {
            return Some(default.clone());
        }

        // If only one workspace exists, use it as default
        if self.workspaces.len() == 1 {
            return Some(self.workspaces[0].clone());
        }

        None
    }

    fn find_config_path() -> Result<PathBuf> {
        // First check current directory
        let current_dir = std::env::current_dir()?;
        let local_config = current_dir.join("config.toml");
        if local_config.exists() {
            return Ok(local_config);
        }

        // Then check home directory ~/.config/sc-cli/config.toml
        if let Some(home_dir) = dirs::home_dir() {
            let config_dir = home_dir.join(".config").join("sc-cli");
            let home_config = config_dir.join("config.toml");
            if home_config.exists() {
                return Ok(home_config);
            }
        }

        // Default to current directory
        Ok(current_dir.join("config.toml"))
    }

    fn default_config_path() -> Result<PathBuf> {
        // Default to home directory ~/.config/sc-cli/config.toml
        if let Some(home_dir) = dirs::home_dir() {
            let config_dir = home_dir.join(".config").join("sc-cli");
            Ok(config_dir.join("config.toml"))
        } else {
            // Fallback to current directory
            Ok(PathBuf::from("config.toml"))
        }
    }

    #[allow(dead_code)]
    pub fn example() -> String {
        r#"# SC-TUI Configuration File
# 
# List all workspace names
workspaces = ["personal", "work", "client"]

# Optional: specify default workspace (if not set, single workspace will be used as default)
default_workspace = "personal"

# Configuration for 'personal' workspace
[personal]
api_key = "your-personal-api-key"
user_id = "your-mention-name"
fetch_limit = 50  # Optional: defaults to 50 if not specified

# Configuration for 'work' workspace  
[work]
api_key = "your-work-api-key"
user_id = "your-work-mention-name"
fetch_limit = 50  # Fetch more stories for work workspace

# Configuration for 'client' workspace
[client]
api_key = "your-client-api-key"
user_id = "your-client-mention-name"
# fetch_limit not specified, will use default of 50
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config_content = r#"
workspaces = ["test", "prod"]

[test]
api_key = "test-key"
user_id = "test.user"
fetch_limit = 30

[prod]
api_key = "prod-key"
user_id = "prod.user"
"#;

        let config: Config = toml::from_str(config_content).unwrap();

        assert_eq!(config.workspaces.len(), 2);
        assert_eq!(config.workspaces[0], "test");
        assert_eq!(config.workspaces[1], "prod");

        let test_workspace = config.get_workspace("test").unwrap();
        assert_eq!(test_workspace.api_key, "test-key");
        assert_eq!(test_workspace.user_id, "test.user");
        assert_eq!(test_workspace.fetch_limit, 30); // Explicitly set

        let prod_workspace = config.get_workspace("prod").unwrap();
        assert_eq!(prod_workspace.api_key, "prod-key");
        assert_eq!(prod_workspace.user_id, "prod.user");
        assert_eq!(prod_workspace.fetch_limit, 50); // Default value
    }

    #[test]
    fn test_workspace_not_found() {
        let config_content = r#"
workspaces = ["test"]

[test]
api_key = "test-key"
user_id = "test.user"
"#;

        let config: Config = toml::from_str(config_content).unwrap();

        let result = config.get_workspace("nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Workspace 'nonexistent' not found")
        );
    }

    #[test]
    fn test_default_workspace_single() {
        let config_content = r#"
workspaces = ["only"]

[only]
api_key = "only-key"
user_id = "only.user"
"#;

        let config: Config = toml::from_str(config_content).unwrap();

        // Should return the only workspace as default
        assert_eq!(config.get_default_workspace(), Some("only".to_string()));
    }

    #[test]
    fn test_default_workspace_explicit() {
        let config_content = r#"
workspaces = ["test", "prod"]
default_workspace = "prod"

[test]
api_key = "test-key"
user_id = "test.user"

[prod]
api_key = "prod-key"
user_id = "prod.user"
"#;

        let config: Config = toml::from_str(config_content).unwrap();

        // Should return the explicitly set default
        assert_eq!(config.get_default_workspace(), Some("prod".to_string()));
    }

    #[test]
    fn test_default_workspace_multiple_no_default() {
        let config_content = r#"
workspaces = ["test", "prod"]

[test]
api_key = "test-key"
user_id = "test.user"

[prod]
api_key = "prod-key"
user_id = "prod.user"
"#;

        let config: Config = toml::from_str(config_content).unwrap();

        // Should return None when multiple workspaces and no default set
        assert_eq!(config.get_default_workspace(), None);
    }

    #[test]
    fn test_default_workspace_invalid_default() {
        let config_content = r#"
workspaces = ["test"]
default_workspace = "nonexistent"

[test]
api_key = "test-key"
user_id = "test.user"
"#;

        let config: Config = toml::from_str(config_content).unwrap();

        // Should fallback to single workspace when default is invalid
        assert_eq!(config.get_default_workspace(), Some("test".to_string()));
    }

    #[test]
    fn test_fetch_limit_various_scenarios() {
        // Test with explicit fetch_limit
        let config_content = r#"
workspaces = ["workspace1"]

[workspace1]
api_key = "key1"
user_id = "user1"
fetch_limit = 100
"#;
        let config: Config = toml::from_str(config_content).unwrap();
        let workspace = config.get_workspace("workspace1").unwrap();
        assert_eq!(workspace.fetch_limit, 100);

        // Test without fetch_limit (should use default)
        let config_content = r#"
workspaces = ["workspace2"]

[workspace2]
api_key = "key2"
user_id = "user2"
"#;
        let config: Config = toml::from_str(config_content).unwrap();
        let workspace = config.get_workspace("workspace2").unwrap();
        assert_eq!(workspace.fetch_limit, 50);

        // Test with zero fetch_limit
        let config_content = r#"
workspaces = ["workspace3"]

[workspace3]
api_key = "key3"
user_id = "user3"
fetch_limit = 0
"#;
        let config: Config = toml::from_str(config_content).unwrap();
        let workspace = config.get_workspace("workspace3").unwrap();
        assert_eq!(workspace.fetch_limit, 0);
    }
}
