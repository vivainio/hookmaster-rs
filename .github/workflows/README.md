# CI/CD Workflows

This repository uses GitHub Actions for continuous integration and deployment.

## Workflows

### 1. CI (`ci.yml`)
Runs on every push to `master`/`main` and on pull requests.

**What it does:**
- **Test Job**: Runs unit tests, checks code formatting with `rustfmt`, and runs `clippy` linting
- **Build Job**: Builds release binaries for multiple platforms (Linux, Windows, macOS Intel, macOS ARM)
- **Release Job**: Creates GitHub releases with binaries when tags are pushed

**Platforms supported:**
- Linux (x86_64-unknown-linux-gnu)
- Windows (x86_64-pc-windows-msvc)
- macOS Intel (x86_64-apple-darwin)
- macOS ARM (aarch64-apple-darwin)

### 2. Release (`release.yml`)
Runs only when version tags are pushed (e.g., `v1.0.0`).

**What it does:**
- Creates a GitHub release
- Builds optimized binaries for all supported platforms
- Uploads binaries as release assets

## Creating a Release

To create a new release:

1. **Update version in `Cargo.toml`**:
   ```toml
   [package]
   version = "1.0.0"  # Update this
   ```

2. **Commit the version change**:
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 1.0.0"
   ```

3. **Create and push a git tag**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

4. **The release workflow will automatically**:
   - Create a GitHub release
   - Build binaries for all platforms
   - Upload the binaries to the release

## Binary Names

The released binaries will be named:
- `hookmaster-linux.tar.gz` (Linux x86_64)
- `hookmaster-windows.zip` (Windows x86_64)
- `hookmaster-macos.tar.gz` (macOS Intel)
- `hookmaster-macos-arm.tar.gz` (macOS ARM)

## Development

- All code must pass `cargo fmt --check`
- All code must pass `clippy` with no warnings
- All tests must pass

## Configuration Files

- `rustfmt.toml` - Code formatting rules
- `clippy.toml` - Linting configuration 