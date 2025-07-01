# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-20

### Added
- Initial release of hookmaster-rs, a Rust implementation of hookmaster
- Automatic commit message formatting based on branch names
- Support for JIRA/ticket-style branch names (e.g., `JIRA-123-description`)
- Configurable git hooks via `githooks.toml` file
- Cross-platform support (Windows, macOS, Linux)
- CLI commands:
  - `hookmaster add <path>` - Install hooks to repositories under path
  - `hookmaster init` - Initialize current repository with sample config
  - `hookmaster run <hook-name>` - Run a specific hook command
  - `hookmaster prepare-commit-msg` - Process commit message formatting
- Comprehensive test suite
- Full documentation and examples

### Features
- Converts branch names like `feature/JIRA-123-add-new-feature` to `JIRA-123: Add New Feature`
- Supports common branch prefixes: `feature/`, `bugfix/`, `hotfix/`, `fix/`
- Handles underscores and hyphens in branch descriptions
- Configurable hook commands for all standard Git hooks
- Recursive repository discovery and hook installation
- Logging with configurable verbosity levels 