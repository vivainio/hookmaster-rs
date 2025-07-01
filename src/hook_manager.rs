use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::commit_msg::CommitMessageProcessor;
use crate::config::GitHooksConfig;
use crate::git_hooks::{find_git_repositories, GitHook};

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
        let repositories = find_git_repositories(path).with_context(|| {
            format!("Failed to find git repositories under: {}", path.display())
        })?;

        if repositories.is_empty() {
            eprintln!("No git repositories found under: {}", path.display());
            return Ok(());
        }

        println!("Found {} git repositories", repositories.len());

        for repo in repositories {
            println!("Installing hooks to: {}", repo.display());
            self.install_hooks_to_repo(&repo)?;
        }

        println!("Successfully installed hooks to all repositories");
        Ok(())
    }

    /// Install hooks to a specific repository
    fn install_hooks_to_repo(&self, repo_path: &Path) -> Result<()> {
        // Install standard hooks
        for hook in GitHook::standard_hooks() {
            hook.install_to_repo(repo_path).with_context(|| {
                format!(
                    "Failed to install {} hook to {}",
                    hook.to_filename(),
                    repo_path.display()
                )
            })?;
        }

        Ok(())
    }

    /// Initialize current repository with sample githooks.toml
    pub fn init_repository(&self) -> Result<()> {
        let config_path = Path::new("githooks.toml");

        if config_path.exists() {
            eprintln!("githooks.toml already exists, skipping initialization");
            return Ok(());
        }

        // Create sample configuration
        let sample_config = GitHooksConfig::create_sample();
        sample_config
            .save_to_file(config_path)
            .with_context(|| "Failed to create sample githooks.toml")?;

        println!("Created sample githooks.toml");

        // Install hooks to current repository
        let current_dir =
            std::env::current_dir().with_context(|| "Failed to get current directory")?;

        if crate::git_hooks::is_git_repository(&current_dir) {
            self.install_hooks_to_repo(&current_dir)?;
            println!("Installed hooks to current repository");
        } else {
            eprintln!("Current directory is not a git repository, hooks not installed");
        }

        Ok(())
    }

    /// Run a specific hook command
    pub fn run_hook(&self, hook_name: &str, _args: &[String]) -> Result<()> {
        // Load configuration
        let config = GitHooksConfig::load().with_context(|| "Failed to load githooks.toml")?;

        // Check if hook is defined and active
        if !config.has_active_hook(hook_name) {
            return Ok(());
        }

        let command = config
            .get_hook_command(hook_name)
            .ok_or_else(|| anyhow::anyhow!("Hook '{}' not found in configuration", hook_name))?;

        // Execute the command
        let exit_status = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command]).status()
        } else {
            Command::new("sh").args(["-c", command]).status()
        };

        match exit_status {
            Ok(status) => {
                if !status.success() {
                    let code = status.code().unwrap_or(-1);
                    eprintln!("Hook '{hook_name}' failed with exit code: {code}");
                    return Err(anyhow::anyhow!(
                        "Hook '{}' failed with exit code: {}",
                        hook_name,
                        code
                    ));
                }
            }
            Err(e) => {
                eprintln!("Failed to execute hook '{hook_name}': {e}");
                return Err(anyhow::anyhow!(
                    "Failed to execute hook '{}': {}",
                    hook_name,
                    e
                ));
            }
        }

        Ok(())
    }

    /// Handle prepare-commit-msg hook
    pub fn prepare_commit_msg(
        &self,
        commit_msg_file: &Path,
        commit_source: Option<&str>,
        commit_sha: Option<&str>,
    ) -> Result<()> {
        self.commit_processor
            .process_commit_msg_file(commit_msg_file, commit_source, commit_sha)
            .with_context(|| "Failed to process commit message")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_repository() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();

        // Test creating sample config in a specific path (not current dir)
        let config_path = temp_dir.path().join("githooks.toml");
        let sample_config = GitHooksConfig::create_sample();
        let result = sample_config.save_to_file(&config_path);

        assert!(result.is_ok());
        assert!(config_path.exists());

        // Verify the content
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("pre-commit"));
        assert!(content.contains("pre-push"));
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
