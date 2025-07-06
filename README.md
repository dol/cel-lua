# CEL Lua

CEL (Common Expression Language) library for Lua with FFI bindings.

[![CI](https://github.com/cel-lua/cel-lua/workflows/CI/badge.svg)](https://github.com/cel-lua/cel-lua/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast and efficient CEL expression evaluator for Lua environments, built with Rust and FFI bindings. Automatically detects and works with both OpenResty and standalone LuaJIT environments.

## Features

- Fast CEL expression evaluation with Rust backend
- FFI bindings for seamless Lua integration
- Support for basic CEL data types (null, bool, int, uint, double, string)
- Arithmetic, logical, and comparison operations
- String concatenation and manipulation
- Variable binding and context management
- Expression validation and variable extraction
- Memory-safe string management
- Works with OpenResty and standalone LuaJIT (auto-detection)

## Quick Start

```bash
# Build the project
make build

# Run tests
make test

# Run example
make example
```

## Requirements

- Rust 1.70+
- LuaJIT (for FFI support)
- OpenResty (optional, for full integration tests)

## Installation

1. Clone the repository
2. Run `make build` to compile the Rust library
3. The Lua modules will be available in `lib/resty/cel/`

For development setup, run `./setup.sh` (requires OpenResty for full testing).

## CEL Language Support

At the core of the library, CEL Lua is a wrapper around the [CEL (Common Expression Language)](https://github.com/google/cel-spec) that provides simple expression evaluation with type safety and performance.

### Supported Data Types

- `null` - null/nil value
- `bool` - boolean true/false  
- `int` - 64-bit signed integer
- `uint` - 64-bit unsigned integer
- `double` - 64-bit floating point
- `string` - UTF-8 string value

### Supported Operations

- **Arithmetic**: `+`, `-`, `*`, `/`, `%`
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical**: `&&`, `||`, `!`
- **String**: concatenation with `+`
- **Ternary**: `condition ? true_value : false_value`

Please refer to the [CEL specification](https://github.com/google/cel-spec/blob/master/doc/langdef.md) for detailed language documentation.

## Usage Examples

### Basic Usage

```lua
local cel = require("cel")

-- Create program and context
local program = cel.program.new()
local context = cel.context.new()

-- Simple arithmetic
program:compile("1 + 2 * 3")
local result = program:execute(context)
print(result) -- 7

-- String operations with variables
context:add_variable("name", "World")
program:compile("'Hello, ' + name + '!'")
result = program:execute(context)
print(result) -- "Hello, World!"

-- Boolean logic
context:add_variable("age", 25)
context:add_variable("status", "active")
program:compile("age >= 18 && status == 'active'")
result = program:execute(context)
print(result) -- true
```

### OpenResty Usage

```lua
lua_package_path '/path/to/cel-lua/lib/?.lua;;';
lua_package_cpath '/path/to/cel-lua/target/release/?.so;;';

location = /cel_example {
    content_by_lua_block {
        local cel = require("cel")
        
        local program = cel.program.new()
        local context = cel.context.new()
        local compiled, err = program:compile("name == 'world' && age > 18")

        if not compiled then
            ngx.log(ngx.ERR, "Compilation error: ", err)
            return
        end

        context:add_variable("name", "world")
        context:add_variable("age", 25)

        local result = program:execute(context)
        ngx.say(result) -- prints: true
    }
}
```

## API Reference

### cel.context

#### context.new()

Create a new context instance for storing variables.

```lua
local context = cel.context.new()
```

#### context:add_variable(name, value)

Add a variable to the context.

```lua
context:add_variable("name", "value")
context:add_variable("age", 25)
context:add_variable("active", true)
```

#### context:reset()

Clear all variables from the context.

```lua
context:reset()
```

### cel.program

#### program.new()

Create a new program instance for compiling and executing expressions.

```lua
local program = cel.program.new()
```

#### program:compile(expression)

Compile a CEL expression. Returns `true` on success, or `nil, error` on failure.

```lua
local success, err = program:compile("age > 18 && status == 'active'")
if not success then
    print("Error:", err)
end
```

#### program:execute(context)

Execute the compiled expression with the given context. Returns the result value, or `nil, error` on failure.

```lua
local result, err = program:execute(context)
if err then
    print("Execution error:", err)
else
    print("Result:", result)
end
```

#### program.validate(expression)

Validate an expression and extract variable information. Returns validation info or `nil, error`.

```lua
local info, err = cel.program.validate("user.name == 'admin' && user.age > 21")
if info then
    print("Variables detected:", info.variable_count)
end
```

## Current Limitations

- Limited to basic CEL value types (null, bool, int, uint, double, string)
- Complex data types (lists, maps, timestamps, durations) not yet implemented
- Field access syntax (object.field) planned for future release
- Built-in functions (size, string conversions) not yet supported

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and contribution guidelines.

## License

Copyright © 2025 CEL Lua Contributors.

Licensed under the [MIT License](https://opensource.org/licenses/MIT).
