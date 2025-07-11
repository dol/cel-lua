# Pre-commit Setup

This project uses [pre-commit](https://pre-commit.com/) to ensure code quality and consistency.

## Installation

1. Install pre-commit:

   ```bash
   pip install pre-commit
   ```

   Or using other package managers:

   ```bash
   # macOS with Homebrew
   brew install pre-commit

   # Ubuntu/Debian
   sudo apt-get install pre-commit
   ```

2. Install the git hooks:

   ```bash
   pre-commit install
   ```

## Usage

Pre-commit will automatically run when you commit. You can also run it manually:

```bash
# Run on all files
pre-commit run --all-files

# Run specific hooks
pre-commit run luacheck
pre-commit run stylua
pre-commit run cargo-clippy
pre-commit run cargo-fmt
pre-commit run markdownlint

# Run on staged files only
pre-commit run
```

## Hooks Configured

### Lua Development

- **luacheck**: Lua linting using the project's `.luacheckrc` configuration
- **stylua**: Lua code formatting (automatically fixes formatting issues)

### Documentation

- **markdownlint**: Markdown linting using `.markdownlint.yaml` configuration

### General

- **trailing-whitespace**: Removes trailing whitespace
- **end-of-file-fixer**: Ensures files end with a newline
- **check-yaml**: Validates YAML syntax
- **check-added-large-files**: Prevents committing large files (>1MB)
- **check-merge-conflict**: Detects merge conflict markers
- **mixed-line-ending**: Ensures consistent line endings (LF)

## Rust Hooks

The following Rust-specific hooks are configured:

- **`cargo fmt`**: Ensures consistent Rust code formatting
- **`cargo clippy`**: Catches common mistakes and suggests improvements
- **`cargo check`**: Verifies Rust code compiles successfully
- **`cargo test`**: Runs all Rust unit and integration tests
- **`cargo audit`**: Checks for security vulnerabilities in dependencies

For detailed information about Rust hooks, see [docs/RUST_PRE_COMMIT.md](docs/RUST_PRE_COMMIT.md).

## Manual Commands

You can also run the underlying tools directly:

```bash
# Lua linting
make lint-lua

# Lua formatting
make format-lua

# Rust linting
make lint-rust

# Rust formatting
make format-rust

# Lint all (Lua + Rust)
make lint

# Format all code (Lua + Rust)
make format

# Markdown linting
markdownlint --config .markdownlint.yaml *.md
```

## Available Make Targets

The following make targets are available for code formatting and quality:

```bash
# Format Lua code
make format-lua

# Format Rust code
make format-rust

# Format all code (Lua + Rust)
make format

# Lint Lua code
make lint-lua

# Lint Rust code
make lint-rust

# Lint all code (Lua + Rust)
make lint
```

The pre-commit hooks use these specific targets to ensure consistency between local development and CI.

## Configuration Files

- `.pre-commit-config.yaml` - Pre-commit hook configuration
- `.luacheckrc` - Luacheck configuration for Lua linting
- `.markdownlint.yaml` - Markdownlint configuration for Markdown files
