use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn, error};

use crate::config::GitHooksConfig;
use crate::git_hooks::{GitHook, find_git_repositories};
use crate::commit_msg::CommitMessageProcessor;

/// Main hook manager that orchestrates all hookmaster functionality
pub struct HookManager {
    commit_processor: CommitMessageProcessor,
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HookManager {
    /// Create a new hook manager
    pub fn new() -> Self {
        Self {
            commit_processor: CommitMessageProcessor::new(),
        }
    }

    /// Add hookmaster hooks to all repositories under the given path
    pub fn add_hooks_to_path(&self, path: &Path) -> Result<()> {
        info!("Searching for git repositories under: {}", path.display());
        
        let repositories = find_git_repositories(path)
            .with_context(|| format!("Failed to find git repositories under: {}", path.display()))?;

        if repositories.is_empty() {
            warn!("No git repositories found under: {}", path.display());
            return Ok(());
        }

        info!("Found {} git repositories", repositories.len());

        for repo in repositories {
            info!("Installing hooks to: {}", repo.display());
            self.install_hooks_to_repo(&repo)?;
        }

        info!("Successfully installed hooks to all repositories");
        Ok(())
    }

    /// Install hooks to a specific repository
    fn install_hooks_to_repo(&self, repo_path: &Path) -> Result<()> {
        // Install standard hooks
        for hook in GitHook::standard_hooks() {
            hook.install_to_repo(repo_path)
                .with_context(|| format!("Failed to install {} hook to {}", hook.to_filename(), repo_path.display()))?;
            debug!("Installed {} hook", hook.to_filename());
        }

        info!("Installed all hooks to: {}", repo_path.display());
        Ok(())
    }

    /// Initialize current repository with sample githooks.toml
    pub fn init_repository(&self) -> Result<()> {
        let config_path = Path::new("githooks.toml");
        
        if config_path.exists() {
            warn!("githooks.toml already exists, skipping initialization");
            return Ok(());
        }

        // Create sample configuration
        let sample_config = GitHooksConfig::create_sample();
        sample_config.save_to_file(config_path)
            .with_context(|| "Failed to create sample githooks.toml")?;

        info!("Created sample githooks.toml");

        // Install hooks to current repository
        let current_dir = std::env::current_dir()
            .with_context(|| "Failed to get current directory")?;
        
        if crate::git_hooks::is_git_repository(&current_dir) {
            self.install_hooks_to_repo(&current_dir)?;
            info!("Installed hooks to current repository");
        } else {
            warn!("Current directory is not a git repository, hooks not installed");
        }

        Ok(())
    }

    /// Run a specific hook command
    pub fn run_hook(&self, hook_name: &str, args: &[String]) -> Result<()> {
        debug!("Running hook: {} with args: {:?}", hook_name, args);

        // Load configuration
        let config = GitHooksConfig::load_from_current_dir()
            .with_context(|| "Failed to load githooks.toml")?;

        // Check if hook is defined and active
        if !config.has_active_hook(hook_name) {
            debug!("Hook '{}' is not defined or is empty, skipping", hook_name);
            return Ok(());
        }

        let command = config.get_hook_command(hook_name)
            .ok_or_else(|| anyhow::anyhow!("Hook '{}' not found in configuration", hook_name))?;

        info!("Executing hook '{}': {}", hook_name, command);

        // Execute the command
        let exit_status = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .status()
        } else {
            Command::new("sh")
                .args(["-c", command])
                .status()
        };

        match exit_status {
            Ok(status) => {
                if status.success() {
                    info!("Hook '{}' completed successfully", hook_name);
                } else {
                    let code = status.code().unwrap_or(-1);
                    error!("Hook '{}' failed with exit code: {}", hook_name, code);
                    return Err(anyhow::anyhow!("Hook '{}' failed with exit code: {}", hook_name, code));
                }
            }
            Err(e) => {
                error!("Failed to execute hook '{}': {}", hook_name, e);
                return Err(anyhow::anyhow!("Failed to execute hook '{}': {}", hook_name, e));
            }
        }

        Ok(())
    }

    /// Handle prepare-commit-msg hook
    pub fn prepare_commit_msg(&self, commit_msg_file: &Path, commit_source: Option<&str>, commit_sha: Option<&str>) -> Result<()> {
        debug!("Processing prepare-commit-msg hook");
        debug!("Commit message file: {}", commit_msg_file.display());
        debug!("Commit source: {:?}", commit_source);
        debug!("Commit SHA: {:?}", commit_sha);

        self.commit_processor.process_commit_msg_file(commit_msg_file, commit_source, commit_sha)
            .with_context(|| "Failed to process commit message")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_init_repository() {
        let temp_dir = TempDir::new().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let hook_manager = HookManager::new();
        let result = hook_manager.init_repository();
        
        // Restore original directory
        std::env::set_current_dir(old_dir).unwrap();
        
        assert!(result.is_ok());
        assert!(temp_dir.path().join("githooks.toml").exists());
    }

    #[test]
    fn test_run_hook_with_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Create empty config
        let config = GitHooksConfig::default();
        config.save_to_file("githooks.toml").unwrap();
        
        let hook_manager = HookManager::new();
        let result = hook_manager.run_hook("non-existent", &[]);
        
        // Restore original directory
        std::env::set_current_dir(old_dir).unwrap();
        
        // Should succeed but do nothing for empty/non-existent hooks
        assert!(result.is_ok());
    }
} 