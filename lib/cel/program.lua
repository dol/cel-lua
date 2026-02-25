local _M = {}
local _MT = { __index = _M }

local ffi = require("ffi")
local cdefs = require("cel.cdefs")

local ffi_new = ffi.new
local ffi_gc = ffi.gc
local ffi_string = ffi.string
local tonumber = tonumber
local setmetatable = setmetatable

local ERR_BUF_MAX_LEN = cdefs.ERR_BUF_MAX_LEN
local clib = cdefs.clib
local program_free = cdefs.program_free
local get_string_buf = cdefs.get_string_buf
local get_size_ptr = cdefs.get_size_ptr
local use_resty_core = cdefs.use_resty_core

-- Helper function to convert CelValue to Lua value
local function cel_value_to_lua_value(cel_val)
  if cel_val.value_type == cdefs.Null then
    return nil
  elseif cel_val.value_type == cdefs.Bool then
    return cel_val.data.bool_val
  elseif cel_val.value_type == cdefs.Int then
    return tonumber(cel_val.data.int_val)
  elseif cel_val.value_type == cdefs.Uint then
    return tonumber(cel_val.data.uint_val)
  elseif cel_val.value_type == cdefs.Double then
    return cel_val.data.double_val
  elseif cel_val.value_type == cdefs.String then
    local string_val = cel_val.data.string_val
    local result = ffi_string(string_val.ptr, string_val.len)
    -- Clean up the string memory
    if not use_resty_core and clib.cel_string_free then
      clib.cel_string_free(string_val.ptr)
    end
    return result
  elseif cel_val.value_type == cdefs.List then
    return nil, "List values not yet supported"
  elseif cel_val.value_type == cdefs.Map then
    return nil, "Map values not yet supported"
  else
    return nil, "Unsupported value type"
  end
end

function _M.new()
  local program = clib.program_new()
  local p = setmetatable({
    program = ffi_gc(program, program_free),
    compiled = false,
  }, _MT)

  return p
end

function _M:compile(expression)
  local errbuf = get_string_buf(ERR_BUF_MAX_LEN)
  local errbuf_len = get_size_ptr()
  errbuf_len[0] = ERR_BUF_MAX_LEN

  local ok = clib.program_compile(self.program, expression, errbuf, errbuf_len)

  if not ok then
    return false, ffi_string(errbuf, errbuf_len[0])
  end

  self.compiled = true
  return true
end

function _M:execute(context)
  if not self.compiled then
    return nil, "Program not compiled"
  end

  local errbuf = get_string_buf(ERR_BUF_MAX_LEN)
  local errbuf_len = get_size_ptr()
  errbuf_len[0] = ERR_BUF_MAX_LEN

  local result = ffi_new("CelValue[1]")
  local ok = clib.program_execute(self.program, context.context, result, errbuf, errbuf_len)

  if not ok then
    return nil, ffi_string(errbuf, errbuf_len[0])
  end

  return cel_value_to_lua_value(result[0])
end

function _M.validate(expression)
  local errbuf = get_string_buf(ERR_BUF_MAX_LEN)
  local errbuf_len = get_size_ptr()
  errbuf_len[0] = ERR_BUF_MAX_LEN

  local variables_len = ffi_new("uintptr_t[1]")
  local ok = clib.program_validate(expression, nil, variables_len, errbuf, errbuf_len)

  if not ok then
    return nil, ffi_string(errbuf, errbuf_len[0])
  end

  -- For now, just return the count of variables
  return {
    variable_count = tonumber(variables_len[0]),
  }
end

-- Clean up string pool
if not use_resty_core and clib.cel_string_pool_clear then
  function _M.cleanup()
    clib.cel_string_pool_clear()
  end
end

return _M
