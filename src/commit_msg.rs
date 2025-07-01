use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info};

/// Commit message processor that formats messages based on branch names
pub struct CommitMessageProcessor {
    ticket_regex: Regex,
    branch_cleanup_regex: Regex,
}

impl Default for CommitMessageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitMessageProcessor {
    /// Create a new commit message processor
    pub fn new() -> Self {
        // Regex to extract ticket numbers like SOMETICKET-123
        let ticket_regex = Regex::new(r"([A-Z][A-Z0-9]+-\d+)").expect("Invalid ticket regex");
        
        // Regex to clean up branch names (remove common prefixes and convert to title case)
        let branch_cleanup_regex = Regex::new(r"^(?:feature/|bugfix/|hotfix/|fix/)?[A-Z][A-Z0-9]+-\d+(?:-(.+))?$")
            .expect("Invalid branch cleanup regex");
            
        Self {
            ticket_regex,
            branch_cleanup_regex,
        }
    }

    /// Process commit message file for prepare-commit-msg hook
    pub fn process_commit_msg_file(&self, commit_msg_file: &Path, _commit_source: Option<&str>, _commit_sha: Option<&str>) -> Result<()> {
        // Read current commit message
        let current_msg = fs::read_to_string(commit_msg_file)
            .with_context(|| format!("Failed to read commit message file: {}", commit_msg_file.display()))?;

        // Skip if message already has content (not just comments)
        if !current_msg.lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .collect::<Vec<_>>()
            .is_empty()
        {
            debug!("Commit message already has content, skipping formatting");
            return Ok(());
        }

        // Get current branch name
        let branch_name = self.get_current_branch_name()?;
        debug!("Current branch: {}", branch_name);

        // Generate formatted message
        if let Some(formatted_msg) = self.format_commit_message_from_branch(&branch_name) {
            info!("Generated commit message: {}", formatted_msg);
            
            // Prepend the formatted message to existing content
            let new_content = format!("{}\n\n{}", formatted_msg, current_msg);
            
            fs::write(commit_msg_file, new_content)
                .with_context(|| format!("Failed to write commit message file: {}", commit_msg_file.display()))?;
        } else {
            debug!("No ticket found in branch name, skipping message formatting");
        }

        Ok(())
    }

    /// Get current branch name from git repository
    fn get_current_branch_name(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .with_context(|| "Failed to execute git command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git command failed: {}", stderr));
        }
        
        let branch_name = String::from_utf8(output.stdout)
            .with_context(|| "Invalid UTF-8 in git output")?
            .trim()
            .to_string();
        
        Ok(branch_name)
    }

    /// Format commit message based on branch name
    /// Converts something like "/bugfixes/SOMETICKET-123-do-stuff" to "SOMETICKET-123: Do stuff"
    pub fn format_commit_message_from_branch(&self, branch_name: &str) -> Option<String> {
        // Extract ticket number
        let ticket = self.ticket_regex.find(branch_name)?;
        let ticket_id = ticket.as_str();
        
        // Extract and clean up the description part
        let description = self.branch_cleanup_regex.captures(branch_name)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .unwrap_or("");
        
        if description.is_empty() {
            return Some(format!("{}: ", ticket_id));
        }
        
        // Convert to title case and replace hyphens/underscores with spaces
        let formatted_description = self.to_title_case(&description.replace('-', " ").replace('_', " "));
        
        Some(format!("{}: {}", ticket_id, formatted_description))
    }

    /// Convert string to title case
    fn to_title_case(&self, s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_commit_message_from_branch() {
        let processor = CommitMessageProcessor::new();
        
        // Test with typical branch names
        assert_eq!(
            processor.format_commit_message_from_branch("feature/JIRA-123-add-new-feature"),
            Some("JIRA-123: Add New Feature".to_string())
        );
        
        assert_eq!(
            processor.format_commit_message_from_branch("bugfix/TICKET-456-fix-important-bug"),
            Some("TICKET-456: Fix Important Bug".to_string())
        );
        
        assert_eq!(
            processor.format_commit_message_from_branch("hotfix/ABC-789-urgent_fix"),
            Some("ABC-789: Urgent Fix".to_string())
        );
        
        // Test with branch name that has no description
        assert_eq!(
            processor.format_commit_message_from_branch("feature/JIRA-123"),
            Some("JIRA-123: ".to_string())
        );
        
        // Test with branch name that has no ticket
        assert_eq!(
            processor.format_commit_message_from_branch("main"),
            None
        );
        
        assert_eq!(
            processor.format_commit_message_from_branch("feature/some-feature"),
            None
        );
    }

    #[test]
    fn test_to_title_case() {
        let processor = CommitMessageProcessor::new();
        
        assert_eq!(processor.to_title_case("hello world"), "Hello World");
        assert_eq!(processor.to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(processor.to_title_case("hELLo WoRLd"), "Hello World");
        assert_eq!(processor.to_title_case("single"), "Single");
        assert_eq!(processor.to_title_case(""), "");
    }
} 