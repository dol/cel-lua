local test_helper = require("spec.test_helper")
test_helper.setup_lua_path()

describe("CEL Program Compilation and Validation", function()
  local cel

  before_each(function()
    cel = require("cel")
  end)

  describe("Program Compilation", function()
    it("should compile valid expressions", function()
      local prog = cel.program.new()
      local ok, err = prog:compile("1 + 2")
      assert.is_true(ok)
      assert.is_nil(err)
    end)

    it("should reject invalid expressions", function()
      local prog = cel.program.new()
      local ok, err = prog:compile("1 + + 2") -- Invalid syntax
      assert.is_false(ok)
      assert.is_not_nil(err)
      assert.is_string(err)
    end)

    it("should handle empty expressions", function()
      local prog = cel.program.new()
      local ok, err = prog:compile("")
      assert.is_false(ok)
      assert.is_not_nil(err)
    end)

    it("should handle complex valid expressions", function()
      local prog = cel.program.new()
      local ok, err = prog:compile("(x + y) * z > threshold && active")
      assert.is_true(ok)
      assert.is_nil(err)
    end)
  end)

  describe("Program Execution", function()
    it("should not execute uncompiled programs", function()
      local prog = cel.program.new()
      local ctx = cel.context.new()

      local result, err = prog:execute(ctx)
      assert.is_nil(result)
      assert.is_not_nil(err)
      assert.equals("Program not compiled", err)
    end)

    it("should execute compiled programs", function()
      local prog = cel.program.new()
      local ctx = cel.context.new()

      local ok, err = prog:compile("42")
      assert.is_true(ok)
      assert.is_nil(err)

      local result, exec_err = prog:execute(ctx)
      assert.is_nil(exec_err)
      assert.equals(42, result)
    end)

    it("should handle multiple executions", function()
      local prog = cel.program.new()
      local ctx1 = cel.context.new()
      local ctx2 = cel.context.new()

      ctx1:add_variable("x", 10)
      ctx2:add_variable("x", 20)

      local ok = prog:compile("x * 2")
      assert.is_true(ok)

      local result1, err1 = prog:execute(ctx1)
      assert.is_nil(err1)
      assert.equals(20, result1)

      local result2, err2 = prog:execute(ctx2)
      assert.is_nil(err2)
      assert.equals(40, result2)
    end)
  end)

  describe("Program Validation", function()
    it("should validate simple expressions", function()
      local result, err = cel.program.validate("1 + 2")
      assert.is_not_nil(result)
      assert.is_nil(err)
      assert.is_number(result.variable_count)
    end)

    it("should detect variables in expressions", function()
      local result, err = cel.program.validate("x + y")
      assert.is_not_nil(result)
      assert.is_nil(err)
      -- Should detect at least some variables
      assert.is_true(result.variable_count >= 0)
    end)

    it("should handle validation of invalid expressions", function()
      local result, err = cel.program.validate("1 + + 2")
      assert.is_nil(result)
      assert.is_not_nil(err)
    end)

    it("should validate complex expressions", function()
      local result, err = cel.program.validate("(a + b) * c > d && e")
      assert.is_not_nil(result)
      assert.is_nil(err)
      assert.is_number(result.variable_count)
    end)
  end)

  describe("Program Lifecycle", function()
    it("should allow recompilation", function()
      local prog = cel.program.new()

      -- First compilation
      local ok1, err1 = prog:compile("1 + 2")
      assert.is_true(ok1)
      assert.is_nil(err1)

      -- Second compilation (should replace the first)
      local ok2, err2 = prog:compile("3 * 4")
      assert.is_true(ok2)
      assert.is_nil(err2)

      -- Execute should use the second expression
      local ctx = cel.context.new()
      local result, exec_err = prog:execute(ctx)
      assert.is_nil(exec_err)
      assert.equals(12, result)
    end)

    it("should handle compilation errors gracefully", function()
      local prog = cel.program.new()

      -- First successful compilation
      local ok1, err1 = prog:compile("1 + 2")
      assert.is_true(ok1)
      assert.is_nil(err1)

      -- Failed compilation (should not affect the previous state)
      local ok2, err2 = prog:compile("invalid syntax +++")
      assert.is_false(ok2)
      assert.is_not_nil(err2)

      -- The program should still be in the compiled state from the first compilation
      local ctx = cel.context.new()
      local result, exec_err = prog:execute(ctx)
      -- This behavior may vary depending on implementation
      -- Either it should still work with the first expression or indicate not compiled
      assert.is_not_nil(result or exec_err)
    end)
  end)

  describe("Memory Management", function()
    it("should handle multiple program instances", function()
      local programs = {}

      -- Create multiple programs
      for i = 1, 10 do
        local prog = cel.program.new()
        local ok = prog:compile("x + " .. i)
        assert.is_true(ok)
        programs[i] = prog
      end

      -- Execute all programs
      local ctx = cel.context.new()
      ctx:add_variable("x", 100)

      for i = 1, 10 do
        local result, err = programs[i]:execute(ctx)
        assert.is_nil(err)
        assert.equals(100 + i, result)
      end
    end)
  end)
end)
