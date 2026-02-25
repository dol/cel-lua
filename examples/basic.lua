#!/usr/bin/env luajit

-- Simple example showing how to use cel-lua

package.path = "../lib/?.lua;../lib/?/init.lua;" .. package.path
package.cpath = "../target/debug/?.so;" .. package.cpath

-- Use the unified CEL implementation
local cel = require("cel")

print("CEL Lua Example")
print("===============")

-- Create program and context
local program = cel.program.new()
local context = cel.context.new()

-- Example 1: Simple boolean expression
print("\nExample 1: Simple boolean expression")
print("Expression: name == 'world'")

local success, err = program:compile("name == 'world'")
if not success then
  print("Compilation error:", err)
  return
end

context:add_variable("name", "world")
local result, err = program:execute(context)
if err then
  print("Execution error:", err)
else
  print("Result:", result)
end

-- Example 2: Numeric comparison
print("\nExample 2: Numeric comparison")
print("Expression: age > 18 && age < 65")

local program2 = cel.program.new()
local success, err = program2:compile("age > 18 && age < 65")
if not success then
  print("Compilation error:", err)
  return
end

local context2 = cel.context.new()
context2:add_variable("age", 25)
local result, err = program2:execute(context2)
if err then
  print("Execution error:", err)
else
  print("Result:", result)
end

-- Example 3: String operations
print("\nExample 3: String operations")
print("Expression: 'Hello, ' + name + '!'")

local program3 = cel.program.new()
local success, err = program3:compile("'Hello, ' + name + '!'")
if not success then
  print("Compilation error:", err)
  return
end

local context3 = cel.context.new()
context3:add_variable("name", "CEL")
local result, err = program3:execute(context3)
if err then
  print("Execution error:", err)
else
  print("Result:", result)
end

print("\nExample completed!")
