use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

mod config;
mod git_hooks;
mod commit_msg;
mod hook_manager;

use config::GitHooksConfig;
use git_hooks::GitHook;
use hook_manager::HookManager;

#[derive(Parser)]
#[command(name = "hookmaster")]
#[command(about = "Some nice git hooks for your pleasure")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Add hookmaster hooks to all projects under the specified path
    Add {
        /// Path to add hooks to (searches recursively for git repositories)
        path: PathBuf,
    },
    /// Initialize current repository with sample githooks.toml
    Init,
    /// Run a specific hook command
    Run {
        /// Hook name to run (e.g., pre-commit, commit-msg, etc.)
        hook_name: String,
        /// Additional arguments to pass to the hook
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Process prepare-commit-msg hook
    PrepareCommitMsg {
        /// Path to the commit message file
        commit_msg_file: PathBuf,
        /// Commit source (optional)
        commit_source: Option<String>,
        /// SHA1 of the commit (optional)
        commit_sha: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("hookmaster={}", log_level))
        .init();

    match cli.command {
        Commands::Add { path } => {
            info!("Adding hookmaster hooks to repositories under: {}", path.display());
            let hook_manager = HookManager::new();
            hook_manager.add_hooks_to_path(&path)?;
        }
        Commands::Init => {
            info!("Initializing repository with sample githooks.toml");
            let hook_manager = HookManager::new();
            hook_manager.init_repository()?;
        }
        Commands::Run { hook_name, args } => {
            info!("Running hook: {}", hook_name);
            let hook_manager = HookManager::new();
            hook_manager.run_hook(&hook_name, &args)?;
        }
        Commands::PrepareCommitMsg { commit_msg_file, commit_source, commit_sha } => {
            info!("Processing prepare-commit-msg hook");
            let hook_manager = HookManager::new();
            hook_manager.prepare_commit_msg(&commit_msg_file, commit_source.as_deref(), commit_sha.as_deref())?;
        }
    }

    Ok(())
} 