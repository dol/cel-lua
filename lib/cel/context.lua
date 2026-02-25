local _M = {}
local _MT = { __index = _M }

local ffi = require("ffi")
local cdefs = require("cel.cdefs")

local ffi_new = ffi.new
local ffi_gc = ffi.gc
local ffi_string = ffi.string
local setmetatable = setmetatable

local ERR_BUF_MAX_LEN = cdefs.ERR_BUF_MAX_LEN
local clib = cdefs.clib
local context_free = cdefs.context_free
local get_string_buf = cdefs.get_string_buf
local get_size_ptr = cdefs.get_size_ptr

-- Helper function to convert Lua values to CelValue
local function lua_value_to_cel_value(lua_val, cel_val)
  local val_type = type(lua_val)

  if lua_val == nil then
    cel_val.value_type = cdefs.Null
    return true
  elseif val_type == "boolean" then
    cel_val.value_type = cdefs.Bool
    cel_val.data.bool_val = lua_val
    return true
  elseif val_type == "number" then
    if lua_val == math.floor(lua_val) then
      cel_val.value_type = cdefs.Int
      cel_val.data.int_val = lua_val
    else
      cel_val.value_type = cdefs.Double
      cel_val.data.double_val = lua_val
    end
    return true
  elseif val_type == "string" then
    cel_val.value_type = cdefs.String
    cel_val.data.string_val.ptr = lua_val
    cel_val.data.string_val.len = #lua_val
    return true
  else
    return false
  end
end

function _M.new()
  local context = clib.context_new()
  local c = setmetatable({
    context = ffi_gc(context, context_free),
  }, _MT)

  return c
end

function _M:add_variable(name, value)
  local errbuf = get_string_buf(ERR_BUF_MAX_LEN)
  local errbuf_len = get_size_ptr()
  errbuf_len[0] = ERR_BUF_MAX_LEN

  -- Convert Lua value to CelValue
  local cel_value = ffi_new("CelValue[1]")
  local success = lua_value_to_cel_value(value, cel_value[0])

  if not success then
    return nil, "Failed to convert value"
  end

  local ok = clib.context_add_variable(self.context, name, cel_value, errbuf, errbuf_len)

  if not ok then
    return nil, ffi_string(errbuf, errbuf_len[0])
  end

  return true
end

function _M:reset()
  clib.context_reset(self.context)
end

return _M
