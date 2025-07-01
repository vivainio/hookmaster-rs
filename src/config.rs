use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration structure for githooks.toml
#[derive(Debug, Default)]
pub struct GitHooksConfig {
    /// Map of hook names to commands
    pub hooks: HashMap<String, String>,
}

impl GitHooksConfig {
    /// Load configuration from githooks.toml file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config = Self::parse_toml(&content).with_context(|| "Failed to parse githooks.toml")?;

        Ok(config)
    }

    /// Load configuration from current directory
    pub fn load_from_current_dir() -> Result<Self> {
        let config_path = Path::new("githooks.toml");
        if config_path.exists() {
            Self::load_from_file(config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to githooks.toml file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = self.to_toml_string();

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Simple TOML parser for key = "value" pairs
    fn parse_toml(content: &str) -> Result<Self> {
        let mut hooks = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key = "value" or key = 'value'
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let value_part = line[eq_pos + 1..].trim();

                // Remove quotes if present
                let value = if (value_part.starts_with('"') && value_part.ends_with('"'))
                    || (value_part.starts_with('\'') && value_part.ends_with('\''))
                {
                    value_part[1..value_part.len() - 1].to_string()
                } else {
                    value_part.to_string()
                };

                hooks.insert(key, value);
            }
        }

        Ok(GitHooksConfig { hooks })
    }

    /// Convert to TOML string
    fn to_toml_string(&self) -> String {
        let mut lines = Vec::new();

        // Sort keys for consistent output
        let mut sorted_hooks: Vec<_> = self.hooks.iter().collect();
        sorted_hooks.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_hooks {
            lines.push(format!("{key} = \"{value}\""));
        }

        lines.join("\n") + "\n"
    }

    /// Get command for a specific hook
    pub fn get_hook_command(&self, hook_name: &str) -> Option<&String> {
        self.hooks.get(hook_name)
    }

    /// Create a sample configuration
    pub fn create_sample() -> Self {
        let mut hooks = HashMap::new();
        hooks.insert(
            "pre-commit".to_string(),
            "cargo fmt --check && cargo clippy -- -D warnings".to_string(),
        );
        hooks.insert("pre-push".to_string(), "cargo test".to_string());
        hooks.insert("commit-msg".to_string(), "".to_string()); // Empty string does nothing

        Self { hooks }
    }

    /// Check if a hook is defined and not empty
    pub fn has_active_hook(&self, hook_name: &str) -> bool {
        self.hooks
            .get(hook_name)
            .map(|cmd| !cmd.trim().is_empty())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_toml() {
        let content = r#"
# Comment
pre-commit = "cargo fmt --check"
pre-push = "cargo test"
commit-msg = ""
"#;

        let config = GitHooksConfig::parse_toml(content).unwrap();

        assert_eq!(
            config.hooks.get("pre-commit"),
            Some(&"cargo fmt --check".to_string())
        );
        assert_eq!(
            config.hooks.get("pre-push"),
            Some(&"cargo test".to_string())
        );
        assert_eq!(config.hooks.get("commit-msg"), Some(&"".to_string()));
    }

    #[test]
    fn test_to_toml_string() {
        let mut hooks = HashMap::new();
        hooks.insert("pre-commit".to_string(), "test command".to_string());
        hooks.insert("pre-push".to_string(), "test2".to_string());

        let config = GitHooksConfig { hooks };
        let toml_str = config.to_toml_string();

        assert!(toml_str.contains("pre-commit = \"test command\""));
        assert!(toml_str.contains("pre-push = \"test2\""));
    }
}
