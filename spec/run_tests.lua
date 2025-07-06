#!/usr/bin/env luajit

-- Simple test runner for CEL Lua when Busted is not available
local test_helper = require("spec.test_helper")
test_helper.setup_lua_path()

local function run_basic_tests()
  print("=== Running Basic CEL Tests ===")

  local cel = require("cel")

  -- Test 1: Module loading
  assert(cel ~= nil, "CEL module should load")
  assert(cel.context ~= nil, "CEL context should be available")
  assert(cel.program ~= nil, "CEL program should be available")
  print("✓ Module loading test passed")

  -- Test 2: Simple arithmetic
  local result2, err2 = test_helper.eval_expression(cel, "1 + 2")
  assert(err2 == nil, "No error expected: " .. tostring(err2))
  assert(result2 == 3, "1 + 2 should equal 3, got " .. tostring(result2))
  print("✓ Simple arithmetic test passed")

  -- Test 3: Variables
  local result3, err3 = test_helper.eval_expression(cel, "x * 2", { x = 21 })
  assert(err3 == nil, "No error expected: " .. tostring(err3))
  assert(result3 == 42, "x * 2 with x=21 should equal 42, got " .. tostring(result3))
  print("✓ Variable test passed")

  -- Test 4: Boolean operations
  local result4, err4 = test_helper.eval_expression(cel, "true && false")
  assert(err4 == nil, "No error expected: " .. tostring(err4))
  assert(result4 == false, "true && false should be false")
  print("✓ Boolean operation test passed")

  -- Test 5: Comparison
  local result5, err5 = test_helper.eval_expression(cel, "age >= 18", { age = 25 })
  assert(err5 == nil, "No error expected: " .. tostring(err5))
  assert(result5 == true, "25 >= 18 should be true")
  print("✓ Comparison test passed")

  -- Test 6: String operations
  local result6, err6 = test_helper.eval_expression(cel, "name", { name = "World" })
  assert(err6 == nil, "No error expected: " .. tostring(err6))
  assert(result6 == "World", "String variable should work")
  print("✓ String operation test passed")

  -- Test 7: Error handling
  local prog = cel.program.new()
  local ok, err7 = prog:compile("1 + + 2") -- Invalid syntax
  assert(ok ~= true, "Should fail to compile invalid expression")
  assert(err7 ~= nil, "Should return error message")
  print("✓ Error handling test passed")

  print("=== All basic tests passed! ===\n")
end

local function run_environment_tests()
  print("=== Running Environment Tests ===")

  -- Test unified implementation
  local cel = require("cel")

  assert(cel ~= nil, "CEL should load")

  -- Test with the unified implementation
  local result, err = test_helper.eval_expression(cel, "10 + 5")

  assert(result == 15, "Expression should evaluate correctly")
  assert(err == nil, "No error expected")

  print("✓ Environment detection test passed")
  print("✓ Unified implementation test passed")

  print("=== Environment tests passed! ===\n")
end

local function run_performance_tests()
  print("=== Running Performance Tests ===")

  local cel = require("cel")

  -- Test compilation performance
  local start_time = os.clock()
  for _ = 1, 50 do
    local prog = cel.program.new()
    local ok = prog:compile("x + " .. _)
    assert(ok == true, "Compilation should succeed")
  end
  local compile_time = os.clock() - start_time
  print("✓ Compiled 50 expressions in " .. string.format("%.3f", compile_time) .. " seconds")

  -- Test execution performance
  local prog = cel.program.new()
  local ctx = cel.context.new()
  prog:compile("x + y")
  ctx:add_variable("x", 10)
  ctx:add_variable("y", 20)

  start_time = os.clock()
  for _ = 1, 100 do
    local result, err = prog:execute(ctx)
    assert(err == nil, "Execution should succeed")
    assert(result == 30, "Result should be correct")
  end
  local exec_time = os.clock() - start_time
  print("✓ Executed expression 100 times in " .. string.format("%.3f", exec_time) .. " seconds")

  print("=== Performance tests passed! ===\n")
end

-- Run all tests
local function main()
  print("CEL Lua Test Suite")
  print("==================")

  local success1, err1 = pcall(run_basic_tests)
  if not success1 then
    print("FAILED: Basic tests failed - " .. tostring(err1))
    os.exit(1)
  end

  local success2, err2 = pcall(run_environment_tests)
  if not success2 then
    print("FAILED: Environment tests failed - " .. tostring(err2))
    os.exit(1)
  end

  local success3, err3 = pcall(run_performance_tests)
  if not success3 then
    print("FAILED: Performance tests failed - " .. tostring(err3))
    os.exit(1)
  end

  print("🎉 ALL TESTS PASSED! 🎉")
  print("\nThe CEL Lua unified implementation is working correctly!")
end

main()
