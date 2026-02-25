local test_helper = require("spec.test_helper")
test_helper.setup_lua_path()

describe("CEL Environment Detection and Compatibility", function()
  local cel

  before_each(function()
    cel = require("cel")
  end)

  describe("Environment Detection", function()
    it("should load successfully in any environment", function()
      assert.is_not_nil(cel)
      assert.is_not_nil(cel.context)
      assert.is_not_nil(cel.program)
    end)

    it("should provide the same API regardless of environment", function()
      -- Test that the main entry point works
      local cel_entry = require("cel")

      assert.is_not_nil(cel_entry.context)
      assert.is_not_nil(cel_entry.program)
    end)

    it("should handle OpenResty detection gracefully", function()
      -- This test verifies that the module loads without crashing
      -- regardless of whether resty.core.base is available
      local is_openresty = test_helper.is_openresty()
      -- Should be either true or false, not nil
      assert.is_not_nil(is_openresty)
      assert.is_boolean(is_openresty)
    end)
  end)

  describe("Unified Implementation", function()
    it("should work consistently across environments", function()
      local cel_module = require("cel")

      -- Test simple arithmetic
      local result1, err1 = test_helper.eval_expression(cel_module, "2 + 3")
      assert.is_nil(err1)
      assert.equals(5, result1)
    end)

    it("should handle variables correctly", function()
      local cel_module = require("cel")

      local vars = { x = 10, y = 20 }
      local expr = "x * y + 5"

      local result, err = test_helper.eval_expression(cel_module, expr, vars)
      assert.is_nil(err)
      assert.equals(205, result)
    end)

    it("should handle complex expressions", function()
      local cel_module = require("cel")

      local vars = {
        price = 29.99,
        quantity = 3,
        discount = 0.1,
        tax_rate = 0.08,
      }
      local expr = "(price * quantity * (1 - discount)) * (1 + tax_rate)"

      local result, err = test_helper.eval_expression(cel_module, expr, vars)
      assert.is_nil(err)
      assert.is_number(result)
      -- Should be approximately 87.45084
      assert.is_true(math.abs(result - 87.45084) < 0.001)
    end)
  end)

  describe("Buffer Management", function()
    it("should handle string operations correctly", function()
      local result, err = test_helper.eval_expression(cel, "msg", { msg = "Hello, World!" })
      assert.is_nil(err)
      assert.equals("Hello, World!", result)
    end)

    it("should handle long strings", function()
      local long_string = string.rep("A", 1000)
      local result, err = test_helper.eval_expression(cel, "data", { data = long_string })
      assert.is_nil(err)
      assert.equals(long_string, result)
    end)

    it("should handle many variables", function()
      local vars = {}
      for i = 1, 50 do
        vars["var" .. i] = i
      end

      local ctx = test_helper.create_test_context(cel, vars)
      assert.is_not_nil(ctx)

      -- Test accessing some variables
      local result, err = test_helper.eval_expression(cel, "var1 + var50", vars)
      assert.is_nil(err)
      assert.equals(51, result)
    end)
  end)

  describe("Error Handling Consistency", function()
    it("should handle compilation errors gracefully", function()
      local cel_module = require("cel")

      local invalid_expr = "1 + + 2"

      local prog = cel_module.program.new()

      local compiled, err = prog:compile(invalid_expr)

      assert.is_false(compiled)
      assert.is_not_nil(err)
      assert.is_string(err)
    end)

    it("should handle runtime errors gracefully", function()
      local cel_module = require("cel")

      -- This may or may not cause a runtime error depending on implementation
      local expr = "undefined_variable"

      local ctx = cel_module.context.new()
      local prog = cel_module.program.new()

      local compile_ok, compile_err = prog:compile(expr)

      if compile_ok then
        local result, err = prog:execute(ctx)
        -- Should either return nil or an error
        assert.is_true(result == nil or err ~= nil)
      else
        -- Compile error is also acceptable
        assert.is_not_nil(compile_err)
      end
    end)
  end)

  describe("Memory and Resource Management", function()
    it("should clean up resources properly", function()
      -- Test that we can create and destroy many objects without issues
      for i = 1, 100 do
        local ctx = cel.context.new()
        local prog = cel.program.new()

        ctx:add_variable("test", i)
        prog:compile("test * 2")
        local result, err = prog:execute(ctx)

        assert.is_nil(err)
        assert.are.equal(i * 2, result)
      end
    end)

    it("should handle string cleanup appropriately", function()
      -- Test string handling across multiple operations
      for i = 1, 50 do
        local test_string = "Test string " .. i
        local result, err = test_helper.eval_expression(cel, "str", { str = test_string })
        assert.is_nil(err)
        assert.are.equal(test_string, result)
      end
    end)
  end)
end)
