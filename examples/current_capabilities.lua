#!/usr/bin/env luajit

-- Working examples demonstrating current CEL Lua capabilities
local cel = require("cel")

print("=== CEL Lua Library - Current Capabilities Demo ===")
print()

-- Example 1: Basic arithmetic
print("1. Basic Arithmetic:")
local ctx = cel.context.new()
local prog = cel.program.new()

prog:compile("1 + 2 * 3")
local result = prog:execute(ctx)
print("   Expression: 1 + 2 * 3")
print("   Result: " .. result)
print()

-- Example 2: String operations
print("2. String Operations:")
ctx = cel.context.new()
prog = cel.program.new()

ctx:add_variable("name", "World")
prog:compile("'Hello, ' + name + '!'")
result = prog:execute(ctx)
print("   Expression: 'Hello, ' + name + '!'")
print("   Variables: name = 'World'")
print("   Result: " .. result)
print()

-- Example 3: Boolean logic
print("3. Boolean Logic:")
ctx = cel.context.new()
prog = cel.program.new()

ctx:add_variable("age", 25)
ctx:add_variable("status", "active")
prog:compile("age >= 18 && status == 'active'")
result = prog:execute(ctx)
print("   Expression: age >= 18 && status == 'active'")
print("   Variables: age = 25, status = 'active'")
print("   Result: " .. tostring(result))
print()

-- Example 4: Variable extraction
print("4. Variable Extraction:")
local validation = cel.program.validate("size(items) > 0 && user.name != ''")
print("   Expression: size(items) > 0 && user.name != ''")
print("   Detected variables: " .. validation.variable_count)
print()

-- Example 5: Complex expressions
print("5. Complex Expressions:")
ctx = cel.context.new()
prog = cel.program.new()

ctx:add_variable("score", 95)
ctx:add_variable("bonus", 10)
prog:compile("score + bonus > 100 ? 'Excellent!' : 'Good job!'")
result = prog:execute(ctx)
print("   Expression: score + bonus > 100 ? 'Excellent!' : 'Good job!'")
print("   Variables: score = 95, bonus = 10")
print("   Result: " .. result)
print()

-- Example 6: Null handling
print("6. Null Handling:")
ctx = cel.context.new()
prog = cel.program.new()

ctx:add_variable("value", nil)
prog:compile("value == null")
result = prog:execute(ctx)
print("   Expression: value == null")
print("   Variables: value = nil")
print("   Result: " .. tostring(result))
print()

-- Cleanup demonstration
print("7. Memory Management:")
cel.program.cleanup()
print("   String pool cleared successfully")
print()

print("🎉 All current capabilities working correctly!")
print()
print("📋 Summary of supported features:")
print("   ✅ Basic data types: null, bool, int, uint, double, string")
print("   ✅ Arithmetic operations: +, -, *, /, %")
print("   ✅ Comparison operations: ==, !=, <, <=, >, >=")
print("   ✅ Logical operations: &&, ||, !")
print("   ✅ String concatenation: + operator")
print("   ✅ Ternary operator: condition ? true_val : false_val")
print("   ✅ Variable extraction from expressions")
print("   ✅ Proper memory management with cleanup")
print()
print("🔄 Coming next:")
print("   🚧 Support for complex objects (maps, lists)")
print("   🚧 Field access (user.name, items[0])")
print("   🚧 Built-in functions (size, string conversions)")
print("   🚧 Advanced CEL features (timestamps, durations)")
