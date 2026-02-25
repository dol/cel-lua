# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-07-03

### Added

- Initial release of CEL Lua library
- FFI bindings for Lua integration with Rust backend
- CEL expression compilation and execution support
- Basic context management for variables (null, bool, int, uint, double, string)
- Program validation functionality with variable extraction
- Memory management with string pool cleanup
- Dual mode support: OpenResty and standalone LuaJIT
- Comprehensive test suite
- Build system with Makefile
- CI/CD pipeline with GitHub Actions
- Examples and documentation

### Features

- **Core Operations**: Arithmetic, logical, comparison operators
- **String Operations**: Concatenation and literals
- **Variable Binding**: Dynamic context with named variables
- **Expression Validation**: Compile-time checking and variable detection
- **Memory Safety**: Proper string cleanup and memory management

### Dependencies

- cel-interpreter 0.9.1 from clarkmcc/cel-rust
- serde and serde_json for data serialization
- lazy_static for global state management

### Known Limitations

- Limited to basic CEL value types (null, bool, int, uint, double, string)
- Complex data types (lists, maps, timestamps, durations) not yet implemented
- Field access syntax (object.field) planned for future release
