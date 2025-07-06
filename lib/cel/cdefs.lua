local ffi = require("ffi")

-- Try to load resty.core.base, fall back to polyfill if not available
local use_resty_core = false
local get_string_buf, get_size_ptr

-- Attempt to load OpenResty's resty.core.base
local ok, base = pcall(require, "resty.core.base")
if ok and base and base.get_string_buf and base.get_size_ptr then
  use_resty_core = true
  get_string_buf = base.get_string_buf
  get_size_ptr = base.get_size_ptr
else
  -- Polyfill for non-OpenResty environments
  get_string_buf = function(size)
    return ffi.new("uint8_t[?]", size)
  end

  get_size_ptr = function()
    return ffi.new("uintptr_t[1]")
  end
end

-- Common FFI definitions - generated from "cbindgen -l c", do not edit manually
ffi.cdef([[
typedef enum CelValueType {
  Null,
  Bool,
  Int,
  Uint,
  Double,
  String,
  Bytes,
  List,
  Map,
  Timestamp,
  Duration,
} CelValueType;

typedef struct CelStringValue {
  const uint8_t *ptr;
  uintptr_t len;
} CelStringValue;

typedef struct CelBytesValue {
  const uint8_t *ptr;
  uintptr_t len;
} CelBytesValue;

typedef union CelValueData {
  bool bool_val;
  int64_t int_val;
  uint64_t uint_val;
  double double_val;
  CelStringValue string_val;
  CelBytesValue bytes_val;
} CelValueData;

typedef struct CelValue {
  CelValueType value_type;
  CelValueData data;
} CelValue;

typedef struct Context Context;

typedef struct Program Program;

struct Context *context_new(void);

void context_free(struct Context *context);

bool context_add_variable(struct Context *context,
                         const char *name,
                         const struct CelValue *value,
                         uint8_t *errbuf,
                         uintptr_t *errbuf_len);

void context_reset(struct Context *context);

struct Program *program_new(void);

void program_free(struct Program *program);

bool program_compile(struct Program *program,
                    const char *expression,
                    uint8_t *errbuf,
                    uintptr_t *errbuf_len);

bool program_execute(const struct Program *program,
                    const struct Context *context,
                    struct CelValue *result,
                    uint8_t *errbuf,
                    uintptr_t *errbuf_len);

bool program_validate(const char *expression,
                     const uint8_t **variables,
                     uintptr_t *variables_len,
                     uint8_t *errbuf,
                     uintptr_t *errbuf_len);

// String memory management functions
void cel_string_free(const uint8_t *ptr);
void cel_string_pool_clear(void);
]])

local ERR_BUF_MAX_LEN = 4096

-- Library loading with fallback strategy
local function load_library()
  local lib_name = ffi.os == "OSX" and "libcel_lua.dylib" or "libcel_lua.so"

  if use_resty_core then
    -- OpenResty environment: use sophisticated path search
    local load_shared_lib
    do
      local tostring = tostring
      local string_gmatch = string.gmatch
      local string_match = string.match
      local io_open = io.open
      local io_close = io.close
      local table_new = require("table.new")

      local cpath = package.cpath

      function load_shared_lib(so_name)
        local tried_paths = table_new(32, 0)
        local i = 1

        for k, _ in string_gmatch(cpath, "[^;]+") do
          local fpath = tostring(string_match(k, "(.*/)"))
          if fpath then
            fpath = fpath .. so_name
            -- Don't get me wrong, the only way to know if a file exist is
            -- trying to open it.
            local f = io_open(fpath)
            if f ~= nil then
              io_close(f)
              return ffi.load(fpath)
            end

            tried_paths[i] = fpath
            i = i + 1
          end
        end

        return nil, tried_paths
      end
    end

    local clib, tried_paths = load_shared_lib(lib_name)
    if not clib then
      error(
        ("could not load %s shared library from the following paths:\n"):format(lib_name)
          .. table.concat(tried_paths, "\n"),
        2
      )
    end
    return clib
  else
    -- Non-OpenResty environment: direct path loading
    return ffi.load("./target/debug/" .. lib_name)
  end
end

-- Load the library
local clib = load_library()

-- Create unified module
local module = {
  clib = clib,
  ERR_BUF_MAX_LEN = ERR_BUF_MAX_LEN,
  get_string_buf = get_string_buf,
  get_size_ptr = get_size_ptr,
  use_resty_core = use_resty_core,
}

-- Add cleanup functions
module.context_free = function(c)
  clib.context_free(c)
end

module.program_free = function(p)
  clib.program_free(p)
end

-- Add CEL value type constants
if use_resty_core then
  -- OpenResty environment: use dynamic constants from the loaded library
  module.Null = clib.Null
  module.Bool = clib.Bool
  module.Int = clib.Int
  module.Uint = clib.Uint
  module.Double = clib.Double
  module.String = clib.String
  module.Bytes = clib.Bytes
  module.List = clib.List
  module.Map = clib.Map
  module.Timestamp = clib.Timestamp
  module.Duration = clib.Duration
else
  -- Non-OpenResty environment: use hardcoded constants
  module.Null = 0
  module.Bool = 1
  module.Int = 2
  module.Uint = 3
  module.Double = 4
  module.String = 5
  module.Bytes = 6
  module.List = 7
  module.Map = 8
  module.Timestamp = 9
  module.Duration = 10
end

return module
