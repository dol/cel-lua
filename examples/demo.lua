#!/usr/bin/env luajit

-- CEL Lua Demo - Working examples demonstrating current capabilities
local cel = require("cel")

print("=== CEL Lua Library Demo ===")

-- Example 1: Basic arithmetic
print("1. Basic Arithmetic: 1 + 2 * 3")
local ctx = cel.context.new()
local prog = cel.program.new()
prog:compile("1 + 2 * 3")
local result = prog:execute(ctx)
print("   Result: " .. result)

-- Example 2: String operations
print("2. String Operations: 'Hello, ' + name + '!'")
ctx = cel.context.new()
prog = cel.program.new()
ctx:add_variable("name", "World")
prog:compile("'Hello, ' + name + '!'")
result = prog:execute(ctx)
print("   Result: " .. result)

-- Example 3: Boolean logic
print("3. Boolean Logic: age >= 18 && status == 'active'")
ctx = cel.context.new()
prog = cel.program.new()
ctx:add_variable("age", 25)
ctx:add_variable("status", "active")
prog:compile("age >= 18 && status == 'active'")
result = prog:execute(ctx)
print("   Result: " .. tostring(result))

-- Example 4: Complex expression with ternary operator
print("4. Ternary Operation: score > 90 ? 'Excellent' : 'Good'")
ctx = cel.context.new()
prog = cel.program.new()
ctx:add_variable("score", 95)
prog:compile("score > 90 ? 'Excellent' : 'Good'")
result = prog:execute(ctx)
print("   Result: " .. result)

print("🎉 All examples working! CEL Lua is ready to use.")
