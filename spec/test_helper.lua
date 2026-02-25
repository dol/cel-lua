-- Test configuration for CEL Lua tests
local _M = {}

-- Setup Lua path to find the CEL modules
function _M.setup_lua_path()
  local current_path = package.path
  local lib_path = "./lib/?.lua;./lib/?/init.lua"
  package.path = lib_path .. ";" .. current_path
end

-- Helper function to check if we're in OpenResty environment
function _M.is_openresty()
  local ok, _ = pcall(require, "resty.core.base")
  return ok
end

-- Special marker for null values (since Lua tables don't store nil values)
_M.NULL = {}

-- Helper function to create a test context with variables
function _M.create_test_context(cel, variables)
  local ctx = cel.context.new()
  if variables then
    for name, value in pairs(variables) do
      -- Convert our special NULL marker to actual nil
      if value == _M.NULL then
        value = nil
      end
      local ok, err = ctx:add_variable(name, value)
      if not ok then
        error("Failed to add variable " .. name .. ": " .. err)
      end
    end
  end
  return ctx
end

-- Helper function to compile and execute a CEL expression
function _M.eval_expression(cel, expression, variables)
  local ctx = _M.create_test_context(cel, variables)
  local prog = cel.program.new()

  local ok, err = prog:compile(expression)
  if not ok then
    return nil, "Compile error: " .. err
  end

  return prog:execute(ctx)
end

return _M
