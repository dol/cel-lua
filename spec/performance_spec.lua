local test_helper = require("spec.test_helper")
test_helper.setup_lua_path()

describe("CEL Performance Tests", function()
  local cel

  before_each(function()
    cel = require("cel")
  end)

  describe("Compilation Performance", function()
    it("should compile simple expressions quickly", function()
      local start_time = os.clock()

      for i = 1, 100 do
        local prog = cel.program.new()
        local ok = prog:compile("x + " .. i)
        assert.is_true(ok)
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      -- Should complete within reasonable time (adjust threshold as needed)
      assert.is_true(duration < 1.0, "Compilation took too long: " .. duration .. " seconds")
    end)

    it("should handle repeated compilation efficiently", function()
      local prog = cel.program.new()
      local start_time = os.clock()

      for i = 1, 50 do
        local ok = prog:compile("result + " .. i)
        assert.is_true(ok)
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 0.5, "Repeated compilation took too long: " .. duration .. " seconds")
    end)
  end)

  describe("Execution Performance", function()
    it("should execute simple expressions quickly", function()
      local prog = cel.program.new()
      local ctx = cel.context.new()

      prog:compile("x + y")
      ctx:add_variable("x", 10)
      ctx:add_variable("y", 20)

      local start_time = os.clock()

      for _ = 1, 1000 do
        local result, err = prog:execute(ctx)
        assert.is_nil(err)
        assert.equals(30, result)
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 1.0, "Execution took too long: " .. duration .. " seconds")
    end)

    it("should handle variable context efficiently", function()
      local prog = cel.program.new()
      prog:compile("sum")

      local start_time = os.clock()

      for i = 1, 500 do
        local ctx = cel.context.new()
        ctx:add_variable("sum", i)
        local result, err = prog:execute(ctx)
        assert.is_nil(err)
        assert.equals(i, result)
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 1.0, "Context creation and execution took too long: " .. duration .. " seconds")
    end)
  end)

  describe("Memory Efficiency", function()
    it("should handle many concurrent contexts", function()
      local prog = cel.program.new()
      prog:compile("value * 2")

      local contexts = {}

      -- Create many contexts
      for i = 1, 100 do
        local ctx = cel.context.new()
        ctx:add_variable("value", i)
        contexts[i] = ctx
      end

      -- Execute with all contexts
      for i = 1, 100 do
        local result, err = prog:execute(contexts[i])
        assert.is_nil(err)
        assert.equals(i * 2, result)
      end
    end)

    it("should handle many concurrent programs", function()
      local ctx = cel.context.new()
      ctx:add_variable("base", 100)

      local programs = {}

      -- Create many programs
      for i = 1, 50 do
        local prog = cel.program.new()
        prog:compile("base + " .. i)
        programs[i] = prog
      end

      -- Execute all programs
      for i = 1, 50 do
        local result, err = programs[i]:execute(ctx)
        assert.is_nil(err)
        assert.equals(100 + i, result)
      end
    end)
  end)

  describe("Complex Expression Performance", function()
    it("should handle complex arithmetic efficiently", function()
      local prog = cel.program.new()
      local ctx = cel.context.new()

      prog:compile("(a + b) * (c - d) / (e + f) > threshold")
      ctx:add_variable("a", 10)
      ctx:add_variable("b", 20)
      ctx:add_variable("c", 50)
      ctx:add_variable("d", 30)
      ctx:add_variable("e", 5)
      ctx:add_variable("f", 15)
      ctx:add_variable("threshold", 25)

      local start_time = os.clock()

      for _ = 1, 100 do
        local result, err = prog:execute(ctx)
        assert.is_nil(err)
        assert.is_boolean(result)
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 0.5, "Complex expression execution took too long: " .. duration .. " seconds")
    end)

    it("should handle string operations efficiently", function()
      local prog = cel.program.new()
      local ctx = cel.context.new()

      prog:compile('prefix + " " + suffix')
      ctx:add_variable("prefix", "Hello")
      ctx:add_variable("suffix", "World")

      local start_time = os.clock()

      for _ = 1, 200 do
        local result, err = prog:execute(ctx)
        assert.is_nil(err)
        assert.equals("Hello World", result)
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 0.5, "String operations took too long: " .. duration .. " seconds")
    end)
  end)

  describe("Validation Performance", function()
    it("should validate expressions quickly", function()
      local expressions = {
        "1 + 2",
        "x * y + z",
        "(a + b) > c",
        "name != '' && age >= 18",
        "price * quantity * (1 - discount)",
      }

      local start_time = os.clock()

      for _ = 1, 20 do
        for _, expr in ipairs(expressions) do
          local result, err = cel.program.validate(expr)
          -- Should either succeed or fail, but not hang
          assert.is_not_nil(result or err)
        end
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 0.5, "Validation took too long: " .. duration .. " seconds")
    end)
  end)

  describe("Stress Tests", function()
    it("should handle rapid create/destroy cycles", function()
      local start_time = os.clock()

      for i = 1, 200 do
        local ctx = cel.context.new()
        local prog = cel.program.new()

        prog:compile("value")
        ctx:add_variable("value", i)

        local result, err = prog:execute(ctx)
        assert.is_nil(err)
        assert.equals(i, result)

        -- Objects should be cleaned up automatically
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 2.0, "Stress test took too long: " .. duration .. " seconds")
    end)

    it("should handle long-running operations", function()
      local prog = cel.program.new()
      prog:compile("counter")

      local start_time = os.clock()

      for i = 1, 1000 do
        local ctx = cel.context.new()
        ctx:add_variable("counter", i)
        local result, err = prog:execute(ctx)
        assert.is_nil(err)
        assert.equals(i, result)
      end

      local end_time = os.clock()
      local duration = end_time - start_time

      assert.is_true(duration < 2.0, "Long-running test took too long: " .. duration .. " seconds")
    end)
  end)
end)
