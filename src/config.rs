use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workspaces: Vec<String>,
    #[serde(flatten)]
    pub workspace_configs: HashMap<String, WorkspaceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub api_key: String,
    pub user_id: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::find_config_path()?;
        
        if !config_path.exists() {
            anyhow::bail!(
                "Config file not found. Create a config.toml file with your workspace settings or use --workspace to create one interactively."
            );
        }
        
        let contents = fs::read_to_string(&config_path)
            .context(format!("Failed to read config file at: {}", config_path.display()))?;
            
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
                        .with_prompt(format!("Workspace '{}' not found. Would you like to create it?", workspace_name))
                        .default(true)
                        .interact()? 
                    {
                        let workspace_config = Self::prompt_for_workspace_config()?;
                        config.add_workspace(workspace_name, workspace_config)?;
                        Ok((config, true))
                    } else {
                        anyhow::bail!("Workspace '{}' not found and creation cancelled", workspace_name);
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
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let workspace_config = Self::prompt_for_workspace_config()?;
        
        let mut workspace_configs = HashMap::new();
        workspace_configs.insert(workspace_name.to_string(), workspace_config);
        
        let config = Config {
            workspaces: vec![workspace_name.to_string()],
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
            
        Ok(WorkspaceConfig { api_key, user_id })
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
        
        // Save the updated config
        let config_path = Self::find_config_path()?;
        self.save(&config_path)?;
        
        Ok(())
    }
    
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
            
        fs::write(path, toml_string)
            .context("Failed to write config file")?;
            
        Ok(())
    }
    
    pub fn get_workspace(&self, name: &str) -> Result<&WorkspaceConfig> {
        self.workspace_configs
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Workspace '{}' not found in config", name))
    }
    
    fn find_config_path() -> Result<PathBuf> {
        // First check current directory
        let current_dir = std::env::current_dir()?;
        let local_config = current_dir.join("config.toml");
        if local_config.exists() {
            return Ok(local_config);
        }
        
        // Then check home directory ~/.config/sc-tui/config.toml
        if let Some(home_dir) = dirs::home_dir() {
            let config_dir = home_dir.join(".config").join("sc-tui");
            let home_config = config_dir.join("config.toml");
            if home_config.exists() {
                return Ok(home_config);
            }
        }
        
        // Default to current directory
        Ok(current_dir.join("config.toml"))
    }
    
    fn default_config_path() -> Result<PathBuf> {
        // Default to home directory ~/.config/sc-tui/config.toml
        if let Some(home_dir) = dirs::home_dir() {
            let config_dir = home_dir.join(".config").join("sc-tui");
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

# Configuration for 'personal' workspace
[personal]
api_key = "your-personal-api-key"
user_id = "your-mention-name"

# Configuration for 'work' workspace  
[work]
api_key = "your-work-api-key"
user_id = "your-work-mention-name"

# Configuration for 'client' workspace
[client]
api_key = "your-client-api-key"
user_id = "your-client-mention-name"
"#.to_string()
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
        
        let prod_workspace = config.get_workspace("prod").unwrap();
        assert_eq!(prod_workspace.api_key, "prod-key");
        assert_eq!(prod_workspace.user_id, "prod.user");
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
        assert!(result.unwrap_err().to_string().contains("Workspace 'nonexistent' not found"));
    }
}