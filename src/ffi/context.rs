use super::{CelValue, CelValueType};
use std::collections::HashMap;
use std::ffi::{c_char, CStr};

/// Context for storing variables for CEL evaluation
#[derive(Debug, Default)]
pub struct Context {
    variables: HashMap<String, serde_json::Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, name: String, value: serde_json::Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variables(&self) -> &HashMap<String, serde_json::Value> {
        &self.variables
    }

    pub fn reset(&mut self) {
        self.variables.clear();
    }
}

/// Create a new context instance
#[no_mangle]
pub extern "C" fn context_new() -> *mut Context {
    Box::into_raw(Box::new(Context::new()))
}

/// Free a context instance
#[no_mangle]
pub unsafe extern "C" fn context_free(context: *mut Context) {
    if !context.is_null() {
        drop(Box::from_raw(context));
    }
}

/// Add a variable to the context
#[no_mangle]
pub unsafe extern "C" fn context_add_variable(
    context: &mut Context,
    name: *const c_char,
    value: &CelValue,
    errbuf: *mut u8,
    errbuf_len: &mut usize,
) -> bool {
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(e) => {
            let error_msg = format!("Invalid variable name: {}", e);
            copy_error_to_buffer(&error_msg, errbuf, errbuf_len);
            return false;
        }
    };

    let json_value = match cel_value_to_json(value) {
        Ok(v) => v,
        Err(e) => {
            copy_error_to_buffer(&e, errbuf, errbuf_len);
            return false;
        }
    };

    context.add_variable(name_str, json_value);
    true
}

/// Reset the context, clearing all variables
#[no_mangle]
pub unsafe extern "C" fn context_reset(context: &mut Context) {
    context.reset();
}

fn cel_value_to_json(value: &CelValue) -> Result<serde_json::Value, String> {
    match value.value_type {
        CelValueType::Null => Ok(serde_json::Value::Null),
        CelValueType::Bool => Ok(serde_json::Value::Bool(unsafe { value.data.bool_val })),
        CelValueType::Int => Ok(serde_json::Value::Number(serde_json::Number::from(
            unsafe { value.data.int_val },
        ))),
        CelValueType::Uint => Ok(serde_json::Value::Number(serde_json::Number::from(
            unsafe { value.data.uint_val },
        ))),
        CelValueType::Double => {
            let float_val = unsafe { value.data.double_val };
            match serde_json::Number::from_f64(float_val) {
                Some(num) => Ok(serde_json::Value::Number(num)),
                None => Err(format!("Invalid float value: {}", float_val)),
            }
        }
        CelValueType::String => {
            let string_val = unsafe { &*value.data.string_val };
            let slice = unsafe { std::slice::from_raw_parts(string_val.ptr, string_val.len) };
            match std::str::from_utf8(slice) {
                Ok(s) => Ok(serde_json::Value::String(s.to_string())),
                Err(e) => Err(format!("Invalid UTF-8 string: {}", e)),
            }
        }
        _ => Err("Unsupported value type".to_string()),
    }
}

fn copy_error_to_buffer(error: &str, errbuf: *mut u8, errbuf_len: &mut usize) {
    if errbuf.is_null() {
        return;
    }

    let error_bytes = error.as_bytes();
    let copy_len = std::cmp::min(error_bytes.len(), *errbuf_len - 1);

    unsafe {
        std::ptr::copy_nonoverlapping(error_bytes.as_ptr(), errbuf, copy_len);
        *errbuf.add(copy_len) = 0; // null terminator
    }

    *errbuf_len = copy_len;
}
