use super::{CelValue, CelValueType};
use std::collections::HashMap;
use std::ffi::{c_char, CStr};

/// Context for storing variables for CEL evaluation
#[derive(Debug, Default)]
pub struct Context {
    variables: HashMap<String, serde_json::Value>,
}

impl Context {
    #[must_use] pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, name: String, value: serde_json::Value) {
        self.variables.insert(name, value);
    }

    #[must_use] pub fn get_variables(&self) -> &HashMap<String, serde_json::Value> {
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
///
/// # Safety
/// The caller must ensure that:
/// - `context` is either null or a valid pointer returned by `context_new`
/// - `context` has not been previously freed
/// - No other references to the context exist
#[no_mangle]
pub unsafe extern "C" fn context_free(context: *mut Context) {
    if !context.is_null() {
        drop(Box::from_raw(context));
    }
}

/// Add a variable to the context
///
/// # Safety
/// The caller must ensure that:
/// - `context` is a valid mutable reference to a Context
/// - `name` is a valid null-terminated C string
/// - `value` is a valid reference to a `CelValue`
/// - `errbuf` is either null or points to a valid buffer of at least `*errbuf_len` bytes
/// - `errbuf_len` is a valid mutable reference to the buffer size
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
            let error_msg = format!("Invalid variable name: {e}");
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
///
/// # Safety
/// The caller must ensure that `context` is a valid mutable reference to a Context
#[no_mangle]
pub unsafe extern "C" fn context_reset(context: &mut Context) {
    context.reset();
}

fn cel_value_to_json(value: &CelValue) -> Result<serde_json::Value, String> {
    match value.value_type {
        CelValueType::Null => Ok(serde_json::Value::Null),
        CelValueType::Bool => Ok(serde_json::Value::Bool(unsafe { value.data.bool_val })),
        CelValueType::Int => Ok(serde_json::Value::Number(serde_json::Number::from(unsafe {
            value.data.int_val
        }))),
        CelValueType::Uint => Ok(serde_json::Value::Number(serde_json::Number::from(unsafe {
            value.data.uint_val
        }))),
        CelValueType::Double => {
            let float_val = unsafe { value.data.double_val };
            match serde_json::Number::from_f64(float_val) {
                Some(num) => Ok(serde_json::Value::Number(num)),
                None => Err(format!("Invalid float value: {float_val}")),
            }
        }
        CelValueType::String => {
            let string_val = unsafe { &*value.data.string_val };
            let slice = unsafe { std::slice::from_raw_parts(string_val.ptr, string_val.len) };
            match std::str::from_utf8(slice) {
                Ok(s) => Ok(serde_json::Value::String(s.to_string())),
                Err(e) => Err(format!("Invalid UTF-8 string: {e}")),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CelStringValue, CelValueData};
    use std::mem::ManuallyDrop;

    #[test]
    fn test_context_new_and_default() {
        let ctx1 = Context::new();
        let ctx2 = Context::default();

        assert_eq!(ctx1.variables.len(), 0);
        assert_eq!(ctx2.variables.len(), 0);
    }

    #[test]
    fn test_context_add_variable_direct() {
        let mut context = Context::new();
        assert_eq!(context.variables.len(), 0);

        context.add_variable("test".to_string(), serde_json::Value::Bool(true));
        assert_eq!(context.variables.len(), 1);
        assert_eq!(context.variables.get("test"), Some(&serde_json::Value::Bool(true)));
    }

    #[test]
    fn test_context_reset_direct() {
        let mut context = Context::new();
        context.add_variable("test1".to_string(), serde_json::Value::Bool(true));
        context.add_variable("test2".to_string(), serde_json::Value::String("hello".to_string()));
        assert_eq!(context.variables.len(), 2);

        context.reset();
        assert_eq!(context.variables.len(), 0);
    }

    #[test]
    fn test_context_get_variables() {
        let mut context = Context::new();
        context
            .add_variable("x".to_string(), serde_json::Value::Number(serde_json::Number::from(42)));

        let vars = context.get_variables();
        assert_eq!(vars.len(), 1);
        assert!(vars.contains_key("x"));
    }

    #[test]
    fn test_cel_value_to_json_bool() {
        let cel_value = CelValue {
            value_type: CelValueType::Bool,
            data: CelValueData { bool_val: true },
        };

        let result = cel_value_to_json(&cel_value).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_cel_value_to_json_int() {
        let cel_value = CelValue {
            value_type: CelValueType::Int,
            data: CelValueData { int_val: -123 },
        };

        let result = cel_value_to_json(&cel_value).unwrap();
        assert_eq!(result, serde_json::Value::Number(serde_json::Number::from(-123)));
    }

    #[test]
    fn test_cel_value_to_json_uint() {
        let cel_value = CelValue {
            value_type: CelValueType::Uint,
            data: CelValueData { uint_val: 456 },
        };

        let result = cel_value_to_json(&cel_value).unwrap();
        assert_eq!(result, serde_json::Value::Number(serde_json::Number::from(456)));
    }

    #[test]
    fn test_cel_value_to_json_double() {
        let cel_value = CelValue {
            value_type: CelValueType::Double,
            data: CelValueData {
                double_val: std::f64::consts::PI,
            },
        };

        let result = cel_value_to_json(&cel_value).unwrap();
        if let serde_json::Value::Number(n) = result {
            assert!((n.as_f64().unwrap() - std::f64::consts::PI).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_cel_value_to_json_null() {
        let cel_value = CelValue {
            value_type: CelValueType::Null,
            data: CelValueData { int_val: 0 }, // Value doesn't matter for null
        };

        let result = cel_value_to_json(&cel_value).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_cel_value_to_json_invalid_double() {
        let cel_value = CelValue {
            value_type: CelValueType::Double,
            data: CelValueData {
                double_val: f64::NAN,
            },
        };

        let result = cel_value_to_json(&cel_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid float value"));
    }

    #[test]
    fn test_copy_error_to_buffer() {
        let error_msg = "Test error message";
        let mut buffer = [0u8; 20];
        let mut buffer_len = buffer.len();

        copy_error_to_buffer(error_msg, buffer.as_mut_ptr(), &mut buffer_len);

        assert_eq!(buffer_len, error_msg.len());
        assert_eq!(buffer[buffer_len], 0); // null terminator

        let result_str = std::str::from_utf8(&buffer[..buffer_len]).unwrap();
        assert_eq!(result_str, error_msg);
    }

    #[test]
    fn test_copy_error_to_buffer_truncation() {
        let error_msg = "This is a very long error message that should be truncated";
        let mut buffer = [0u8; 10];
        let mut buffer_len = buffer.len();

        copy_error_to_buffer(error_msg, buffer.as_mut_ptr(), &mut buffer_len);

        assert_eq!(buffer_len, 9); // 10 - 1 for null terminator
        assert_eq!(buffer[buffer_len], 0); // null terminator

        let result_str = std::str::from_utf8(&buffer[..buffer_len]).unwrap();
        assert_eq!(result_str, "This is a");
    }

    #[test]
    fn test_copy_error_to_buffer_null_ptr() {
        let error_msg = "Test error";
        let mut buffer_len = 100usize;

        // Should not crash with null pointer
        copy_error_to_buffer(error_msg, std::ptr::null_mut(), &mut buffer_len);

        // buffer_len should remain unchanged
        assert_eq!(buffer_len, 100);
    }

    #[test]
    fn test_context_add_variable_string() {
        let mut context = Context::new();
        let test_string = "Hello, CEL!";

        // Test adding string variable
        let json_value = serde_json::Value::String(test_string.to_string());
        context.add_variable("greeting".to_string(), json_value);

        let variables = context.get_variables();
        assert_eq!(variables.len(), 1);
        assert!(variables.contains_key("greeting"));

        if let Some(value) = variables.get("greeting") {
            assert_eq!(value.as_str().unwrap(), test_string);
        }
    }

    #[test]
    fn test_context_add_variable_array() {
        let mut context = Context::new();
        let array_value = serde_json::json!([1, 2, 3, "four", true]);

        context.add_variable("items".to_string(), array_value);

        let variables = context.get_variables();
        assert_eq!(variables.len(), 1);
        assert!(variables.contains_key("items"));

        if let Some(value) = variables.get("items") {
            assert!(value.is_array());
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], 1);
            assert_eq!(arr[3], "four");
        }
    }

    #[test]
    fn test_context_add_variable_object() {
        let mut context = Context::new();
        let object_value = serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "active": true,
            "score": 95.5
        });

        context.add_variable("user".to_string(), object_value);

        let variables = context.get_variables();
        assert_eq!(variables.len(), 1);
        assert!(variables.contains_key("user"));

        if let Some(value) = variables.get("user") {
            assert!(value.is_object());
            let obj = value.as_object().unwrap();
            assert_eq!(obj["name"], "John Doe");
            assert_eq!(obj["age"], 30);
            assert_eq!(obj["active"], true);
            assert_eq!(obj["score"], 95.5);
        }
    }

    #[test]
    fn test_context_add_variable_nested_object() {
        let mut context = Context::new();
        let nested_object = serde_json::json!({
            "user": {
                "profile": {
                    "name": "Alice",
                    "preferences": {
                        "theme": "dark",
                        "notifications": true
                    }
                },
                "permissions": ["read", "write"]
            }
        });

        context.add_variable("data".to_string(), nested_object);

        let variables = context.get_variables();
        assert_eq!(variables.len(), 1);

        if let Some(value) = variables.get("data") {
            assert!(value["user"]["profile"]["name"] == "Alice");
            assert!(value["user"]["profile"]["preferences"]["theme"] == "dark");
            assert!(value["user"]["permissions"].is_array());
        }
    }

    #[test]
    fn test_context_variable_types_edge_cases() {
        let mut context = Context::new(); // Test various numeric edge cases
        context.add_variable(
            "max_i64".to_string(),
            serde_json::Value::Number(serde_json::Number::from(i64::MAX)),
        );
        context.add_variable(
            "min_i64".to_string(),
            serde_json::Value::Number(serde_json::Number::from(i64::MIN)),
        );
        context.add_variable(
            "zero".to_string(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );
        context.add_variable("float_val".to_string(), serde_json::json!(f64::MAX)); // Close to f64::MAX
        context.add_variable("small_float".to_string(), serde_json::json!(2.2250738585072014e-308)); // Close to f64::MIN_POSITIVE

        // Test empty collections
        context.add_variable("empty_array".to_string(), serde_json::json!([]));
        context.add_variable("empty_object".to_string(), serde_json::json!({}));

        // Test null value
        context.add_variable("null_value".to_string(), serde_json::Value::Null);

        let variables = context.get_variables();
        assert_eq!(variables.len(), 8);

        assert_eq!(variables["max_i64"], i64::MAX);
        assert_eq!(variables["min_i64"], i64::MIN);
        assert_eq!(variables["zero"], 0);
        assert!(variables["empty_array"].as_array().unwrap().is_empty());
        assert!(variables["empty_object"].as_object().unwrap().is_empty());
        assert!(variables["null_value"].is_null());
    }

    #[test]
    fn test_cel_value_to_json_string() {
        let test_string = "Test string with special chars: áéíóú, 你好, 🚀";
        let string_ptr = crate::store_string_in_pool(test_string);

        let cel_value = super::CelValue {
            value_type: super::CelValueType::String,
            data: CelValueData {
                string_val: ManuallyDrop::new(CelStringValue {
                    ptr: string_ptr,
                    len: test_string.len(),
                }),
            },
        };

        let result = cel_value_to_json(&cel_value);
        assert!(result.is_ok());

        let json_value = result.unwrap();
        assert!(json_value.is_string());
        assert_eq!(json_value.as_str().unwrap(), test_string);

        crate::release_string_from_pool(string_ptr);
    }

    #[test]
    fn test_cel_value_to_json_edge_cases() {
        // Test special floating point values
        let nan_value = super::CelValue {
            value_type: super::CelValueType::Double,
            data: CelValueData {
                double_val: f64::NAN,
            },
        };

        let infinity_value = super::CelValue {
            value_type: super::CelValueType::Double,
            data: CelValueData {
                double_val: f64::INFINITY,
            },
        };

        let neg_infinity_value = super::CelValue {
            value_type: super::CelValueType::Double,
            data: CelValueData {
                double_val: f64::NEG_INFINITY,
            },
        };

        // NaN should fail conversion
        assert!(cel_value_to_json(&nan_value).is_err());

        // Infinity values should also fail for JSON compatibility
        assert!(cel_value_to_json(&infinity_value).is_err());
        assert!(cel_value_to_json(&neg_infinity_value).is_err());
    }

    #[test]
    fn test_copy_error_to_buffer_exact_fit() {
        let error_msg = "exact"; // 5 characters
        let mut buffer = [0u8; 6]; // Exactly enough space including null terminator
        let mut buffer_len = buffer.len();

        copy_error_to_buffer(error_msg, buffer.as_mut_ptr(), &mut buffer_len);

        assert_eq!(buffer_len, 5);
        assert_eq!(buffer[5], 0); // null terminator

        let result_str = std::str::from_utf8(&buffer[..buffer_len]).unwrap();
        assert_eq!(result_str, error_msg);
    }

    #[test]
    fn test_copy_error_to_buffer_single_char() {
        let error_msg = "x";
        let mut buffer = [0u8; 2];
        let mut buffer_len = buffer.len();

        copy_error_to_buffer(error_msg, buffer.as_mut_ptr(), &mut buffer_len);

        assert_eq!(buffer_len, 1);
        assert_eq!(buffer[1], 0);

        let result_str = std::str::from_utf8(&buffer[..buffer_len]).unwrap();
        assert_eq!(result_str, "x");
    }

    #[test]
    fn test_copy_error_to_buffer_empty_message() {
        let error_msg = "";
        let mut buffer = [0u8; 10];
        let mut buffer_len = buffer.len();

        copy_error_to_buffer(error_msg, buffer.as_mut_ptr(), &mut buffer_len);

        assert_eq!(buffer_len, 0);
        assert_eq!(buffer[0], 0); // null terminator at start
    }

    #[test]
    fn test_copy_error_to_buffer_minimal_space() {
        let error_msg = "Error";
        let mut buffer = [0u8; 1]; // Only space for null terminator
        let mut buffer_len = buffer.len();

        copy_error_to_buffer(error_msg, buffer.as_mut_ptr(), &mut buffer_len);

        assert_eq!(buffer_len, 0); // No space for characters
        assert_eq!(buffer[0], 0); // null terminator
    }

    #[test]
    fn test_context_unicode_variable_names() {
        let mut context = Context::new();

        // Test Unicode variable names
        context.add_variable("变量".to_string(), serde_json::json!(42));
        context.add_variable("переменная".to_string(), serde_json::json!("hello"));
        context.add_variable("変数".to_string(), serde_json::json!(true));
        context.add_variable("متغير".to_string(), serde_json::json!(std::f64::consts::PI));

        let variables = context.get_variables();
        assert_eq!(variables.len(), 4);

        assert_eq!(variables["变量"], 42);
        assert_eq!(variables["переменная"], "hello");
        assert_eq!(variables["変数"], true);
        assert_eq!(variables["متغير"], std::f64::consts::PI);
    }

    #[test]
    fn test_context_large_number_of_variables() {
        let mut context = Context::new();

        // Add many variables
        for i in 0..1000 {
            let name = format!("var_{i}");
            let value = serde_json::json!({
                "index": i,
                "even": i % 2 == 0,
                "name": format!("Variable number {}", i)
            });
            context.add_variable(name, value);
        }

        let variables = context.get_variables();
        assert_eq!(variables.len(), 1000);

        // Verify some random variables
        assert_eq!(variables["var_0"]["index"], 0);
        assert_eq!(variables["var_0"]["even"], true);
        assert_eq!(variables["var_500"]["index"], 500);
        assert_eq!(variables["var_500"]["even"], true);
        assert_eq!(variables["var_999"]["index"], 999);
        assert_eq!(variables["var_999"]["even"], false);
    }
}
