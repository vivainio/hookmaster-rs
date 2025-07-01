use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Represents a Git hook type
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum GitHook {
    PreCommit,
    PrepareCommitMsg,
    CommitMsg,
    PostCommit,
    PrePush,
    PostReceive,
    PreReceive,
    Update,
    Custom(String),
}

impl GitHook {
    /// Convert hook to its file name
    pub fn to_filename(&self) -> String {
        match self {
            GitHook::PreCommit => "pre-commit".to_string(),
            GitHook::PrepareCommitMsg => "prepare-commit-msg".to_string(),
            GitHook::CommitMsg => "commit-msg".to_string(),
            GitHook::PostCommit => "post-commit".to_string(),
            GitHook::PrePush => "pre-push".to_string(),
            GitHook::PostReceive => "post-receive".to_string(),
            GitHook::PreReceive => "pre-receive".to_string(),
            GitHook::Update => "update".to_string(),
            GitHook::Custom(name) => name.clone(),
        }
    }

    /// Parse from string
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "pre-commit" => GitHook::PreCommit,
            "prepare-commit-msg" => GitHook::PrepareCommitMsg,
            "commit-msg" => GitHook::CommitMsg,
            "post-commit" => GitHook::PostCommit,
            "pre-push" => GitHook::PrePush,
            "post-receive" => GitHook::PostReceive,
            "pre-receive" => GitHook::PreReceive,
            "update" => GitHook::Update,
            _ => GitHook::Custom(s.to_string()),
        }
    }

    /// Generate the hook script content
    pub fn generate_script_content(&self) -> String {
        match self {
            GitHook::PrepareCommitMsg => r#"#!/bin/sh
hookmaster prepare-commit-msg "$@"
"#
            .to_string(),
            _ => {
                format!(
                    r#"#!/bin/sh
hookmaster run {} "$@"
"#,
                    self.to_filename()
                )
            }
        }
    }

    /// Install the hook to a git repository
    pub fn install_to_repo(&self, repo_path: &Path) -> Result<()> {
        let hooks_dir = repo_path.join(".git").join("hooks");
        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir).with_context(|| {
                format!("Failed to create hooks directory: {}", hooks_dir.display())
            })?;
        }

        let hook_file = hooks_dir.join(self.to_filename());
        let script_content = self.generate_script_content();

        fs::write(&hook_file, script_content)
            .with_context(|| format!("Failed to write hook file: {}", hook_file.display()))?;

        // Make the hook executable
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&hook_file)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_file, perms).with_context(|| {
                format!("Failed to make hook executable: {}", hook_file.display())
            })?;
        }

        Ok(())
    }

    /// Get all standard Git hooks
    pub fn standard_hooks() -> Vec<GitHook> {
        vec![
            GitHook::PreCommit,
            GitHook::PrepareCommitMsg,
            GitHook::CommitMsg,
            GitHook::PostCommit,
            GitHook::PrePush,
        ]
    }
}

/// Check if a directory is a git repository
pub fn is_git_repository(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Find all git repositories under a given path
pub fn find_git_repositories(path: &Path) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();

    if is_git_repository(path) {
        repos.push(path.to_path_buf());
    }

    visit_dirs(path, &mut repos)?;
    Ok(repos)
}

/// Recursively visit directories looking for git repositories
fn visit_dirs(dir: &Path, repos: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() && is_git_repository(&path) {
            repos.push(path.clone());
        } else if path.is_dir()
            && !path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with('.')
        {
            // Recursively search subdirectories, but skip hidden directories
            visit_dirs(&path, repos)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_hook_filename() {
        assert_eq!(GitHook::PreCommit.to_filename(), "pre-commit");
        assert_eq!(
            GitHook::PrepareCommitMsg.to_filename(),
            "prepare-commit-msg"
        );
        assert_eq!(
            GitHook::Custom("custom-hook".to_string()).to_filename(),
            "custom-hook"
        );
    }

    #[test]
    fn test_git_hook_from_str() {
        assert_eq!(GitHook::from_str("pre-commit"), GitHook::PreCommit);
        assert_eq!(
            GitHook::from_str("prepare-commit-msg"),
            GitHook::PrepareCommitMsg
        );
        assert_eq!(
            GitHook::from_str("custom"),
            GitHook::Custom("custom".to_string())
        );
    }

    #[test]
    fn test_script_content_generation() {
        let pre_commit = GitHook::PreCommit;
        let content = pre_commit.generate_script_content();
        assert!(content.contains("#!/bin/sh"));
        assert!(content.contains("hookmaster run pre-commit"));

        let prepare_commit = GitHook::PrepareCommitMsg;
        let content = prepare_commit.generate_script_content();
        assert!(content.contains("hookmaster prepare-commit-msg"));
    }
}
