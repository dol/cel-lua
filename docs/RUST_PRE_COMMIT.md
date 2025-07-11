# Rust Pre-commit Hooks Documentation

This document describes the Rust-specific pre-commit hooks configured for the CEL Lua project.

## Available Rust Hooks

### 1. `cargo fmt` (Rust Formatting)

- **Purpose**: Ensures consistent code formatting across all Rust source files
- **Command**: `make format-rust` (via pre-commit)
- **Files**: `*.rs`
- **Behavior**: Automatically formats Rust code during pre-commit
- **Manual run**: Run `make format-rust` or `cargo fmt` to format the code

### 2. `cargo clippy` (Rust Linting)

- **Purpose**: Catches common mistakes and suggests idiomatic Rust code improvements
- **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Files**: `*.rs`
- **Behavior**: Fails if any clippy warnings are found (treats warnings as errors)
- **Fix**: Address the specific clippy warnings or add `#[allow(...)]` attributes if appropriate

### 3. `cargo check` (Rust Compilation)

- **Purpose**: Verifies that the Rust code compiles successfully
- **Command**: `cargo check --all-targets --all-features`
- **Files**: `*.rs`
- **Behavior**: Fails if the code doesn't compile
- **Fix**: Fix compilation errors in the Rust source code

### 4. `cargo test` (Rust Testing)

- **Purpose**: Runs all Rust unit tests and integration tests
- **Command**: `cargo test --all-features`
- **Files**: `*.rs`
- **Behavior**: Fails if any tests fail
- **Test Coverage**: FFI function lifecycle, compilation, validation, variable management, and memory safety
- **Fix**: Fix failing tests or update tests to match new behavior

### 5. `cargo audit` (Rust Security Audit)

- **Purpose**: Checks for known security vulnerabilities in dependencies
- **Command**: `cargo audit` (with graceful fallback if not installed)
- **Files**: `*.rs`, `Cargo.toml`, `Cargo.lock`
- **Behavior**: Fails if security vulnerabilities are found
- **Installation**: `cargo install cargo-audit`
- **Fix**: Update vulnerable dependencies or acknowledge known issues

## Configuration Details

### Hook Execution Order

The hooks run in the following order:

1. `cargo fmt` - Fast formatting check
2. `cargo clippy` - Linting and code quality
3. `cargo check` - Basic compilation check
4. `cargo test` - Run tests
5. `cargo audit` - Security audit

### Performance Considerations

- **`cargo fmt`**: Very fast, checks formatting without modifying files
- **`cargo clippy`**: Moderate speed, performs analysis and linting
- **`cargo check`**: Fast, only checks compilation without generating binaries
- **`cargo test`**: Variable speed, depends on test suite size
- **`cargo audit`**: Fast, checks against vulnerability database

### Skipping Hooks

To skip specific Rust hooks during commit:

```bash
# Skip all Rust hooks
SKIP=cargo-fmt,cargo-clippy,cargo-check,cargo-test,cargo-audit git commit -m "message"

# Skip only clippy
SKIP=cargo-clippy git commit -m "message"
```

### Local Development Workflow

#### Format and lint before committing

```bash
# Format both Lua and Rust code
make format

# Or format individually
make format-lua
make format-rust

# Fix clippy issues
cargo clippy --fix --allow-dirty --all-targets --all-features
```

#### Run all checks locally

```bash
# Format Rust code
make format-rust
cargo fmt

# Run linting
make lint-rust
cargo clippy --all-targets --all-features -- -D warnings

# Compile and test
cargo check --all-targets --all-features
cargo test --all-features
cargo audit  # if installed
```

## Common Issues and Solutions

### Missing Safety Documentation

**Issue**: `cargo clippy` fails with "unsafe function's docs are missing a `# Safety` section"

**Solution**: Add safety documentation to unsafe functions:

```rust
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `ptr` is valid and properly aligned.
pub unsafe extern "C" fn my_function(ptr: *mut MyType) {
    // function body
}
```

### Missing Default Implementation

**Issue**: `cargo clippy` suggests adding a `Default` implementation

**Solution**: Either implement `Default` or use `#[allow(clippy::new_without_default)]`:

```rust
impl Default for MyStruct {
    fn default() -> Self {
        Self::new()
    }
}
```

### Cargo Audit Not Installed

**Issue**: `cargo audit` hook fails because the tool isn't installed

**Solution**: Install cargo-audit:

```bash
cargo install cargo-audit
```

The hook is configured to gracefully handle missing cargo-audit installation.

## Integration with CI/CD

These hooks ensure that:

- All committed Rust code is properly formatted
- Code passes linting checks
- Code compiles successfully
- Tests pass
- No known security vulnerabilities are introduced

This provides a consistent development experience and prevents common issues from reaching the main branch.
