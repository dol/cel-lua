# Contributing to CEL Lua

Thank you for your interest in contributing to CEL Lua! This document provides guidelines for contributing to the project.

## Development Setup

1. **Prerequisites**

   - Rust 1.70+
   - OpenResty (for testing)
   - Perl with Test::Nginx module
   - cbindgen (for generating C headers)

2. **Quick Setup**

   ```bash
   git clone <your-repo>
   cd cel-lua
   ./setup.sh
   ```

3. **Manual Setup**

   ```bash
   # Install Rust dependencies
   cargo build

   # Install cbindgen
   cargo install cbindgen

   # Install Test::Nginx
   cpan -i Test::Nginx
   ```

## Development Workflow

1. **Building**

   ```bash
   make build        # Build release version
   cargo build       # Build debug version
   ```

2. **Testing**

   ```bash
   make test         # Run all tests
   make valgrind     # Run tests with Valgrind
   ```

3. **Code Formatting**
   ```bash
   cargo fmt         # Format Rust code
   cargo clippy      # Run linter
   ```

## Project Structure

```
cel-lua/
├── src/           # Rust source code
│   ├── lib.rs     # Main library entry point
│   └── ffi/       # FFI bindings
├── lib/           # Lua source code
│   └── resty/cel/ # Lua modules
├── t/             # Test files (Test::Nginx)
├── examples/      # Example scripts
└── Cargo.toml     # Rust dependencies
```

## Guidelines

### Rust Code

- Follow standard Rust conventions
- Use `cargo fmt` and `cargo clippy`
- Add documentation for public APIs
- Write unit tests for new functionality

### Lua Code

- Follow OpenResty Lua conventions
- Use local variables where possible
- Add error handling for FFI calls
- Write integration tests

### FFI Interface

- Keep C interface minimal and stable
- Use proper error handling with error buffers
- Document memory management responsibilities
- Test memory safety with Valgrind

### Testing

- Write tests for new features
- Use Test::Nginx for integration tests
- Test error conditions
- Verify memory safety

## Adding New Features

1. **CEL Value Types**

   - Update `CelValueType` enum in `ffi/mod.rs`
   - Add conversion functions in `context.rs` and `program.rs`
   - Update Lua bindings in `cdefs.lua`
   - Add tests

2. **New APIs**
   - Add Rust implementation in appropriate module
   - Export via FFI in relevant `.rs` file
   - Update C definitions in `cdefs.lua`
   - Create Lua wrapper in appropriate module
   - Add tests and documentation

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`make test`)
6. Run code formatting (`cargo fmt`)
7. Run linter (`cargo clippy`)
8. Commit your changes (`git commit -am 'Add new feature'`)
9. Push to the branch (`git push origin feature/new-feature`)
10. Create a Pull Request

## Code Review

- All submissions require review
- Tests must pass
- Code should be well-documented
- Follow existing patterns and conventions
- Consider backward compatibility

## Issues

When reporting issues:

- Use the issue template if available
- Provide minimal reproduction case
- Include relevant version information
- Describe expected vs actual behavior

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
