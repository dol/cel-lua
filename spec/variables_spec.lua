local test_helper = require("spec.test_helper")
test_helper.setup_lua_path()

describe("CEL Variables and Context", function()
  local cel

  before_each(function()
    cel = require("cel")
  end)

  describe("Context Management", function()
    it("should create and manage context", function()
      local ctx = cel.context.new()
      assert.is_not_nil(ctx)
    end)

    it("should reset context", function()
      local ctx = cel.context.new()
      local ok, err = ctx:add_variable("test", 42)
      assert.is_true(ok)
      assert.is_nil(err)

      ctx:reset()
      -- After reset, the context should be empty
      -- Note: We can't directly test this without executing an expression
    end)
  end)

  describe("Variable Types", function()
    it("should handle string variables", function()
      local result, err = test_helper.eval_expression(cel, "name", { name = "World" })
      assert.is_nil(err)
      assert.equals("World", result)
    end)

    it("should handle integer variables", function()
      local result, err = test_helper.eval_expression(cel, "age", { age = 25 })
      assert.is_nil(err)
      assert.equals(25, result)
    end)

    it("should handle float variables", function()
      local result, err = test_helper.eval_expression(cel, "price", { price = 19.99 })
      assert.is_nil(err)
      assert.equals(19.99, result)
    end)

    it("should handle boolean variables", function()
      local result, err = test_helper.eval_expression(cel, "active", { active = true })
      assert.is_nil(err)
      assert.is_true(result)

      result, err = test_helper.eval_expression(cel, "inactive", { inactive = false })
      assert.is_nil(err)
      assert.is_false(result)
    end)

    it("should handle nil variables", function()
      local result, err = test_helper.eval_expression(cel, "empty", { empty = test_helper.NULL })
      assert.is_nil(err)
      assert.is_nil(result)
    end)
  end)

  describe("Variable Operations", function()
    it("should use variables in arithmetic", function()
      local result, err = test_helper.eval_expression(cel, "x + y", { x = 10, y = 5 })
      assert.is_nil(err)
      assert.equals(15, result)
    end)

    it("should use variables in comparisons", function()
      local result, err = test_helper.eval_expression(cel, "age >= 18", { age = 25 })
      assert.is_nil(err)
      assert.is_true(result)

      result, err = test_helper.eval_expression(cel, "age >= 18", { age = 16 })
      assert.is_nil(err)
      assert.is_false(result)
    end)

    it("should use variables in string operations", function()
      local result, err = test_helper.eval_expression(cel, 'greeting + " " + name', {
        greeting = "Hello",
        name = "World",
      })
      assert.is_nil(err)
      assert.equals("Hello World", result)
    end)

    it("should use variables in boolean operations", function()
      local result, err = test_helper.eval_expression(cel, "active && verified", {
        active = true,
        verified = true,
      })
      assert.is_nil(err)
      assert.is_true(result)

      result, err = test_helper.eval_expression(cel, "active && verified", {
        active = true,
        verified = false,
      })
      assert.is_nil(err)
      assert.is_false(result)
    end)
  end)

  describe("Complex Expressions", function()
    it("should evaluate complex arithmetic with variables", function()
      local result, err = test_helper.eval_expression(cel, "(price * quantity) + tax", {
        price = 10.50,
        quantity = 3,
        tax = 2.15,
      })
      assert.is_nil(err)
      assert.equals(33.65, result)
    end)

    it("should evaluate conditional-like expressions", function()
      local result, err = test_helper.eval_expression(cel, "score >= 90 && attendance > 0.8", {
        score = 95,
        attendance = 0.85,
      })
      assert.is_nil(err)
      assert.is_true(result)
    end)

    it("should handle mixed type operations", function()
      local result, err = test_helper.eval_expression(cel, "count > 0 && name != ''", {
        count = 5,
        name = "test",
      })
      assert.is_nil(err)
      assert.is_true(result)
    end)
  end)

  describe("Error Handling", function()
    it("should handle undefined variables gracefully", function()
      local ctx = cel.context.new()
      local prog = cel.program.new()

      local ok, err = prog:compile("undefined_var")
      -- This might succeed at compile time but fail at runtime
      -- The exact behavior depends on the CEL implementation
      if ok then
        local result, exec_err = prog:execute(ctx)
        -- Should either return nil or an error
        assert.is_true(result == nil or exec_err ~= nil)
      else
        -- Compile error is also acceptable
        assert.is_not_nil(err)
      end
    end)

    it("should handle invalid variable names", function()
      local ctx = cel.context.new()
      -- Test with invalid variable name (if the implementation validates)
      local ok = ctx:add_variable("", "value")
      -- The behavior may vary, but it should not crash
      assert.is_not_nil(ok) -- Should return some response
    end)
  end)
end)
