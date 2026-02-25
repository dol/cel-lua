# CEL Lua Test Suite

This directory contains comprehensive tests for the CEL Lua library.

## Test Files

- **`test_helper.lua`** - Common test utilities and helper functions
- **`basic_spec.lua`** - Basic functionality tests (arithmetic, boolean, comparison operations)
- **`variables_spec.lua`** - Variable handling and context management tests
- **`program_spec.lua`** - Program compilation, execution, and validation tests
- **`environment_spec.lua`** - Environment detection and compatibility tests
- **`performance_spec.lua`** - Performance and stress tests
- **`run_tests.lua`** - Simple test runner (works without Busted)

## Running Tests

### With Busted (Recommended)

If you have Busted installed:

```bash
cd /path/to/cel-lua
busted
```

### Without Busted (Simple Runner)

Use the included simple test runner:

```bash
cd /path/to/cel-lua
luajit spec/run_tests.lua
```

### Individual Test Files

Run specific test categories:

```bash
cd /path/to/cel-lua
LUA_PATH="./lib/?.lua;./lib/?/init.lua;;" luajit -e "
dofile('spec/test_helper.lua')
-- Run specific tests here
"
```

## Test Coverage

The test suite covers:

### Basic Functionality

- ✅ Module loading
- ✅ Arithmetic operations (+, -, \*, /, %)
- ✅ Boolean operations (&&, ||, !)
- ✅ Comparison operations (==, !=, <, >, <=, >=)
- ✅ Parentheses and operator precedence

### Variables and Context

- ✅ Variable creation and management
- ✅ Different data types (string, number, boolean, nil)
- ✅ Variable usage in expressions
- ✅ Context reset functionality
- ✅ Error handling for invalid variables

### Program Management

- ✅ Program compilation
- ✅ Program execution
- ✅ Expression validation
- ✅ Recompilation
- ✅ Error handling for invalid expressions
- ✅ Memory management

### Environment Compatibility

- ✅ Automatic OpenResty detection
- ✅ Polyfill fallback for non-OpenResty environments
- ✅ Unified implementation consistency
- ✅ Buffer management across environments
- ✅ String cleanup behavior

### Performance

- ✅ Compilation speed
- ✅ Execution speed
- ✅ Memory efficiency
- ✅ Concurrent operations
- ✅ Stress testing
- ✅ Resource cleanup

## Test Requirements

### Runtime Requirements

- LuaJIT 2.1+
- The CEL Lua library files in `lib/`
- The compiled CEL library (`libcel_lua.so` or `libcel_lua.dylib`)

### Optional Dependencies

- Busted testing framework (for advanced test running)
- OpenResty (for testing OpenResty-specific functionality)

## Test Configuration

The `.busted` file configures Busted to:

- Look for tests in the `spec/` directory
- Use the `_spec` pattern for test files
- Run with LuaJIT
- Provide verbose output

## Writing New Tests

When adding new tests:

1. Follow the existing naming convention (`*_spec.lua`)
2. Use the test helper for common operations
3. Include both positive and negative test cases
4. Test error conditions and edge cases
5. Add performance considerations for complex operations

Example test structure:

```lua
local test_helper = require("spec.test_helper")
test_helper.setup_lua_path()

describe("Feature Name", function()
    local cel

    before_each(function()
        cel = require("cel")
    end)

    describe("Specific Functionality", function()
        it("should do something", function()
            local result, err = test_helper.eval_expression(cel, "1 + 1")
            assert.is_nil(err)
            assert.equals(2, result)
        end)
    end)
end)
```

## Continuous Integration

These tests are designed to work in CI environments and can be run in:

- Pure Lua/LuaJIT environments
- OpenResty environments
- Docker containers
- GitHub Actions
- Any environment with the CEL library compiled
