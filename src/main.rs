use anyhow::{anyhow, Result};
use std::path::PathBuf;

mod commit_msg;
mod config;
mod git_hooks;
mod hook_manager;

use hook_manager::HookManager;

const HELP: &str = "\
hookmaster 0.1.0
Some nice git hooks for your pleasure

USAGE:
    hookmaster [OPTIONS] <COMMAND> [ARGS]...

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
    -v, --verbose    Enable verbose output

COMMANDS:
    add                 Add hookmaster hooks to all projects under the specified path
    init                Initialize current repository with sample githooks.toml
    run                 Run a specific hook command
    prepare-commit-msg  Process prepare-commit-msg hook

Use 'hookmaster <command> --help' for more information on a specific command.
";

const VERSION: &str = "hookmaster 0.1.0";

enum Command {
    Add {
        path: PathBuf,
    },
    Init,
    Run {
        hook_name: String,
        args: Vec<String>,
    },
    PrepareCommitMsg {
        commit_msg_file: PathBuf,
        commit_source: Option<String>,
        commit_sha: Option<String>,
    },
}

fn print_help_for_command(command: &str) {
    match command {
        "add" => println!(
            "\
Add hookmaster hooks to all projects under the specified path

USAGE:
    hookmaster add <PATH>

ARGS:
    <PATH>    Path to add hooks to (searches recursively for git repositories)
"
        ),
        "init" => println!(
            "\
Initialize current repository with sample githooks.toml

USAGE:
    hookmaster init
"
        ),
        "run" => println!(
            "\
Run a specific hook command

USAGE:
    hookmaster run <HOOK_NAME> [ARGS]...

ARGS:
    <HOOK_NAME>    Hook name to run (e.g., pre-commit, commit-msg, etc.)
    [ARGS]...      Additional arguments to pass to the hook
"
        ),
        "prepare-commit-msg" => println!(
            "\
Process prepare-commit-msg hook

USAGE:
    hookmaster prepare-commit-msg <COMMIT_MSG_FILE> [COMMIT_SOURCE] [COMMIT_SHA]

ARGS:
    <COMMIT_MSG_FILE>    Path to the commit message file
    [COMMIT_SOURCE]      Commit source (optional)
    [COMMIT_SHA]         SHA1 of the commit (optional)
"
        ),
        _ => {
            eprintln!("Unknown command: {command}");
            eprintln!("Run 'hookmaster --help' for usage information.");
        }
    }
}

#[allow(clippy::type_complexity)]
fn parse_args() -> Result<(bool, Command)> {
    let mut args = pico_args::Arguments::from_env();

    // Handle version
    if args.contains(["-V", "--version"]) {
        println!("{VERSION}");
        std::process::exit(0);
    }

    // Handle global help flag first
    if args.contains(["-h", "--help"]) {
        // Check if there's a subcommand after the help flag
        if let Ok(subcommand) = args.free_from_str::<String>() {
            print_help_for_command(&subcommand);
        } else {
            println!("{HELP}");
        }
        std::process::exit(0);
    }

    // Parse verbose flag
    let verbose = args.contains(["-v", "--verbose"]);

    // Get the subcommand
    let subcommand: String = match args.free_from_str() {
        Ok(cmd) => cmd,
        Err(_) => {
            return Err(anyhow!(
                "No command specified. Run 'hookmaster --help' for usage information."
            ));
        }
    };

    let command = match subcommand.as_str() {
        "add" => {
            let path: String = args.free_from_str().map_err(|_| {
                anyhow!("Missing required argument: PATH\n\nFor more information try --help")
            })?;
            // Check for unexpected arguments for add command
            let remaining = args.finish();
            if !remaining.is_empty() {
                let unexpected: Vec<String> = remaining
                    .into_iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect();
                return Err(anyhow!(
                    "Unexpected argument(s): {}\n\nFor more information try --help",
                    unexpected.join(", ")
                ));
            }
            Command::Add {
                path: PathBuf::from(path),
            }
        }
        "init" => {
            // Check for unexpected arguments for init command
            let remaining = args.finish();
            if !remaining.is_empty() {
                let unexpected: Vec<String> = remaining
                    .into_iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect();
                return Err(anyhow!(
                    "Unexpected argument(s): {}\n\nFor more information try --help",
                    unexpected.join(", ")
                ));
            }
            Command::Init
        }
        "run" => {
            let hook_name: String = args.free_from_str().map_err(|_| {
                anyhow!("Missing required argument: HOOK_NAME\n\nFor more information try --help")
            })?;
            // For run command, remaining args are passed to the hook
            let remaining_args: Vec<String> = args
                .finish()
                .into_iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();
            Command::Run {
                hook_name,
                args: remaining_args,
            }
        }
        "prepare-commit-msg" => {
            let commit_msg_file: String = args.free_from_str().map_err(|_| {
                anyhow!(
                    "Missing required argument: COMMIT_MSG_FILE\n\nFor more information try --help"
                )
            })?;
            let commit_source: Option<String> = args.free_from_str().ok();
            let commit_sha: Option<String> = args.free_from_str().ok();
            // Check for unexpected arguments for prepare-commit-msg command
            let remaining = args.finish();
            if !remaining.is_empty() {
                let unexpected: Vec<String> = remaining
                    .into_iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect();
                return Err(anyhow!(
                    "Unexpected argument(s): {}\n\nFor more information try --help",
                    unexpected.join(", ")
                ));
            }
            Command::PrepareCommitMsg {
                commit_msg_file: PathBuf::from(commit_msg_file),
                commit_source,
                commit_sha,
            }
        }
        _ => {
            return Err(anyhow!(
                "Unknown command: '{}'\n\nFor more information try --help",
                subcommand
            ));
        }
    };

    Ok((verbose, command))
}

fn main() -> Result<()> {
    let (verbose, command) = parse_args()?;

    match command {
        Command::Add { path } => {
            if verbose {
                println!(
                    "Adding hookmaster hooks to repositories under: {}",
                    path.display()
                );
            }
            let hook_manager = HookManager::new();
            hook_manager.add_hooks_to_path(&path)?;
        }
        Command::Init => {
            if verbose {
                println!("Initializing repository with sample githooks.toml");
            }
            let hook_manager = HookManager::new();
            hook_manager.init_repository()?;
        }
        Command::Run { hook_name, args } => {
            if verbose {
                println!("Running hook: {hook_name}");
            }
            let hook_manager = HookManager::new();
            hook_manager.run_hook(&hook_name, &args)?;
        }
        Command::PrepareCommitMsg {
            commit_msg_file,
            commit_source,
            commit_sha,
        } => {
            if verbose {
                println!("Processing prepare-commit-msg hook");
            }
            let hook_manager = HookManager::new();
            hook_manager.prepare_commit_msg(
                &commit_msg_file,
                commit_source.as_deref(),
                commit_sha.as_deref(),
            )?;
        }
    }

    Ok(())
}
