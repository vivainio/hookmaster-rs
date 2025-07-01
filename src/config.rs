use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration structure for githooks.toml
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GitHooksConfig {
    /// Map of hook names to commands
    #[serde(flatten)]
    pub hooks: HashMap<String, String>,
}

impl GitHooksConfig {
    /// Load configuration from githooks.toml file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let config: Self = toml::from_str(&content)
            .with_context(|| "Failed to parse githooks.toml")?;
        
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
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config to TOML")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    /// Get command for a specific hook
    pub fn get_hook_command(&self, hook_name: &str) -> Option<&String> {
        self.hooks.get(hook_name)
    }

    /// Create a sample configuration
    pub fn create_sample() -> Self {
        let mut hooks = HashMap::new();
        hooks.insert("pre-commit".to_string(), "cargo fmt --check && cargo clippy -- -D warnings".to_string());
        hooks.insert("pre-push".to_string(), "cargo test".to_string());
        hooks.insert("commit-msg".to_string(), "".to_string()); // Empty string does nothing
        
        Self { hooks }
    }

    /// Check if a hook is defined and not empty
    pub fn has_active_hook(&self, hook_name: &str) -> bool {
        self.hooks.get(hook_name)
            .map(|cmd| !cmd.trim().is_empty())
            .unwrap_or(false)
    }
} 