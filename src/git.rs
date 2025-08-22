use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum GitRepoType {
    Normal,
    Bare,
    NotARepo,
}

#[derive(Debug, Clone)]
pub struct GitContext {
    pub repo_type: GitRepoType,
    pub current_branch: Option<String>,
}

impl GitContext {
    pub fn detect() -> Result<Self> {
        let repo_type = detect_git_repo_type()?;
        let current_branch = if repo_type != GitRepoType::NotARepo {
            get_current_branch().ok()
        } else {
            None
        };
        
        Ok(GitContext {
            repo_type,
            current_branch,
        })
    }
    
    pub fn is_git_repo(&self) -> bool {
        self.repo_type != GitRepoType::NotARepo
    }
    
    pub fn is_bare_repo(&self) -> bool {
        self.repo_type == GitRepoType::Bare
    }
}

/// Detect if the current directory is a git repository and what type
pub fn detect_git_repo_type() -> Result<GitRepoType> {
    // First check if we're in a git repository at all
    let is_repo = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("Failed to execute git command")?
        .status
        .success();
    
    if !is_repo {
        return Ok(GitRepoType::NotARepo);
    }
    
    // Check if it's a bare repository
    let is_bare_output = Command::new("git")
        .args(["rev-parse", "--is-bare-repository"])
        .output()
        .context("Failed to check if repository is bare")?;
    
    if !is_bare_output.status.success() {
        return Ok(GitRepoType::NotARepo);
    }
    
    let is_bare = String::from_utf8_lossy(&is_bare_output.stdout)
        .trim()
        .eq_ignore_ascii_case("true");
    
    if is_bare {
        Ok(GitRepoType::Bare)
    } else {
        Ok(GitRepoType::Normal)
    }
}

/// Get the current git branch name
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;
    
    if !output.status.success() {
        anyhow::bail!("Git command failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Create a new git branch
pub fn create_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", "-b", branch_name])
        .output()
        .context("Failed to create git branch")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create branch '{}': {}", branch_name, error);
    }
    
    Ok(())
}

/// Create a new git worktree for bare repositories
pub fn create_worktree(branch_name: &str, worktree_path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "add", "-b", branch_name, worktree_path])
        .output()
        .context("Failed to create git worktree")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create worktree '{}' at '{}': {}", branch_name, worktree_path, error);
    }
    
    Ok(())
}

/// Check if a branch already exists
pub fn branch_exists(branch_name: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch_name)])
        .output()
        .context("Failed to check if branch exists")?;
    
    Ok(output.status.success())
}

/// Generate a safe worktree directory name from branch name
pub fn generate_worktree_path(branch_name: &str) -> String {
    // Replace slashes and other problematic characters with dashes
    let safe_name = branch_name
        .replace('/', "-")
        .replace('\\', "-")
        .replace(' ', "-");
    
    format!("../{}", safe_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    fn setup_test_repo(bare: bool) -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let mut cmd = Command::new("git");
        cmd.args(["init"]);
        
        if bare {
            cmd.arg("--bare");
        }
        
        let output = cmd
            .current_dir(temp_dir.path())
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("Failed to initialize test repo");
        }
        
        if !bare {
            // Create an initial commit for non-bare repos
            fs::write(temp_dir.path().join("README.md"), "# Test repo")?;
            
            Command::new("git")
                .args(["add", "README.md"])
                .current_dir(temp_dir.path())
                .output()?;
            
            Command::new("git")
                .args(["-c", "user.email=test@example.com", "-c", "user.name=Test User", "commit", "-m", "Initial commit"])
                .current_dir(temp_dir.path())
                .output()?;
        }
        
        Ok(temp_dir)
    }
    
    #[test]
    fn test_generate_worktree_path() {
        assert_eq!(generate_worktree_path("feature/test"), "../feature-test");
        assert_eq!(generate_worktree_path("edo/sc-63/story-name"), "../edo-sc-63-story-name");
        assert_eq!(generate_worktree_path("simple"), "../simple");
    }
    
    #[test]
    fn test_detect_non_git_directory() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = detect_git_repo_type().unwrap();
        assert_eq!(result, GitRepoType::NotARepo);
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}