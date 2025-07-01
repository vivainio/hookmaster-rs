# Hookmaster-rs

A Rust implementation of hookmaster - some nice git hooks for your pleasure.

## Problem

* You have a policy where every commit message should mention the JIRA ticket it applies to. You have them in the branch name, and can't be arsed to type them manually to each commit message. Hookmaster provides you with a nice "default" commit message formatter in prepare-commit-msg hook.
* You want to specify commands to run as different git hooks, in a file you share with your team in git.

## Features

- **Automatic commit message formatting**: Converts branch names like `feature/JIRA-123-add-new-feature` to commit messages like `JIRA-123: Add New Feature`
- **Configurable git hooks**: Define hook commands in a `githooks.toml` file
- **Easy installation**: Install hooks to multiple repositories at once
- **Cross-platform**: Works on Windows, macOS, and Linux

## Installation

### From Source

```bash
git clone https://github.com/yourusername/hookmaster-rs.git
cd hookmaster-rs
cargo install --path .
```

### Using Cargo

```bash
cargo install hookmaster-rs
```

## Usage

### Add hooks to repositories

To add hookmaster hooks to all projects under `/my/path`:

```bash
hookmaster add /my/path
```

This will recursively search for git repositories and install hookmaster hooks to each one.

### Initialize a repository

To initialize the current repository with a sample `githooks.toml`:

```bash
hookmaster init
```

This creates a sample configuration file and installs hooks to the current repository.

### Commit Message Formatting

Once installed, hookmaster automatically formats your commit messages based on branch names.

Branch name `feature/JIRA-123-add-new-feature` becomes commit message:
```
JIRA-123: Add New Feature
```

Branch name `bugfix/TICKET-456-fix-important-bug` becomes:
```
TICKET-456: Fix Important Bug
```

The hooks themselves delegate calls to the globally installed hookmaster application:

```bash
#!/bin/sh
hookmaster prepare-commit-msg "$@"
```

This means fixes and updates to hookmaster benefit all your repositories at once.

## Configuration with githooks.toml

For custom hook commands, create a `githooks.toml` file in your repository root:

```toml
pre-commit = "cargo fmt --check && cargo clippy -- -D warnings"
pre-push = "cargo test"
commit-msg = ""  # empty string does nothing
```

The format is straightforward: `hook-name = "command"`. Commands are always run in the repository root.

### Hook Types Supported

- `pre-commit`
- `prepare-commit-msg` (handled specially for commit message formatting)
- `commit-msg`
- `post-commit`
- `pre-push`
- `post-receive`
- `pre-receive`
- `update`

### Running hooks manually

You can test hooks without triggering git operations:

```bash
hookmaster run pre-commit
```

## How it works

1. **Hook Installation**: Creates shell scripts in `.git/hooks/` that delegate to `hookmaster`
2. **Commit Message Processing**: Extracts ticket numbers from branch names using regex patterns
3. **Command Execution**: Runs configured commands from `githooks.toml`
4. **Cross-platform**: Handles Windows (cmd) and Unix (sh) command execution

## Branch Name Patterns

Hookmaster recognizes these branch naming patterns:

- `feature/TICKET-123-description` → `TICKET-123: Description`
- `bugfix/ABC-456-fix-something` → `ABC-456: Fix Something`  
- `hotfix/XYZ-789-urgent_fix` → `XYZ-789: Urgent Fix`
- `JIRA-123-standalone` → `JIRA-123: Standalone`

Ticket patterns must match: `[A-Z][A-Z0-9]+-\d+` (e.g., JIRA-123, ABC-456, TICKET-789)

## Examples

### Sample githooks.toml for Rust projects

```toml
pre-commit = "cargo fmt --check && cargo clippy -- -D warnings"
pre-push = "cargo test --all"
commit-msg = ""
```

### Sample githooks.toml for Node.js projects

```toml
pre-commit = "npm run lint && npm run test"
pre-push = "npm run build"
commit-msg = ""
```

### Sample githooks.toml for Python projects

```toml
pre-commit = "black --check . && flake8"
pre-push = "pytest"
commit-msg = ""
```

## Development

### Building

```bash
cargo build
```

### Running tests

```bash
cargo test
```

### Running with debug output

```bash
hookmaster --verbose <command>
```

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 