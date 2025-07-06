use super::Context;
use cel_interpreter::{Context as CelContext, Program as CelProgram, Value as CelRustValue};
use std::ffi::{c_char, CStr};
use std::mem::ManuallyDrop;

/// CEL Program for compiling and executing expressions
#[derive(Debug)]
pub struct Program {
    program: Option<CelProgram>,
    variables: Vec<String>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            program: None,
            variables: Vec::new(),
        }
    }

    pub fn compile(&mut self, expression: &str) -> Result<(), String> {
        match CelProgram::compile(expression) {
            Ok(program) => {
                // Extract variables from the expression
                self.variables = extract_variables(expression);
                self.program = Some(program);
                Ok(())
            }
            Err(e) => Err(format!("Compilation error: {}", e)),
        }
    }

    pub fn execute(&self, context: &Context) -> Result<CelRustValue, String> {
        let program = self.program.as_ref().ok_or("No expression compiled")?;

        // Create CEL context from our context
        let mut cel_ctx = CelContext::default();

        for (name, value) in context.get_variables() {
            let cel_value = json_to_cel_value(value)
                .map_err(|e| format!("Error converting variable '{}': {}", name, e))?;
            cel_ctx.add_variable_from_value(name, cel_value);
        }

        program
            .execute(&cel_ctx)
            .map_err(|e| format!("Execution error: {}", e))
    }

    pub fn get_variables(&self) -> &[String] {
        &self.variables
    }
}

/// Create a new program instance
#[no_mangle]
pub extern "C" fn program_new() -> *mut Program {
    Box::into_raw(Box::new(Program::new()))
}

/// Free a program instance
#[no_mangle]
pub unsafe extern "C" fn program_free(program: *mut Program) {
    if !program.is_null() {
        drop(Box::from_raw(program));
    }
}

/// Compile a CEL expression
#[no_mangle]
pub unsafe extern "C" fn program_compile(
    program: &mut Program,
    expression: *const c_char,
    errbuf: *mut u8,
    errbuf_len: &mut usize,
) -> bool {
    let expr_str = match CStr::from_ptr(expression).to_str() {
        Ok(s) => s,
        Err(e) => {
            let error_msg = format!("Invalid expression string: {}", e);
            copy_error_to_buffer(&error_msg, errbuf, errbuf_len);
            return false;
        }
    };

    match program.compile(expr_str) {
        Ok(()) => true,
        Err(e) => {
            copy_error_to_buffer(&e, errbuf, errbuf_len);
            false
        }
    }
}

/// Execute the compiled expression
#[no_mangle]
pub unsafe extern "C" fn program_execute(
    program: &Program,
    context: &Context,
    result: *mut super::CelValue,
    errbuf: *mut u8,
    errbuf_len: &mut usize,
) -> bool {
    match program.execute(context) {
        Ok(cel_value) => match cel_value_to_c_value(&cel_value, result) {
            Ok(()) => true,
            Err(e) => {
                copy_error_to_buffer(&e, errbuf, errbuf_len);
                false
            }
        },
        Err(e) => {
            copy_error_to_buffer(&e, errbuf, errbuf_len);
            false
        }
    }
}

/// Validate a CEL expression and return variables
#[no_mangle]
pub unsafe extern "C" fn program_validate(
    expression: *const c_char,
    _variables: *mut *const u8,
    variables_len: *mut usize,
    errbuf: *mut u8,
    errbuf_len: &mut usize,
) -> bool {
    let expr_str = match CStr::from_ptr(expression).to_str() {
        Ok(s) => s,
        Err(e) => {
            let error_msg = format!("Invalid expression string: {}", e);
            copy_error_to_buffer(&error_msg, errbuf, errbuf_len);
            return false;
        }
    };

    match CelProgram::compile(expr_str) {
        Ok(_) => {
            let vars = extract_variables(expr_str);
            // For simplicity, we'll just return the count for now
            // In a full implementation, you'd need to handle the variable names
            if !variables_len.is_null() {
                *variables_len = vars.len();
            }
            true
        }
        Err(e) => {
            let error_msg = format!("Validation error: {}", e);
            copy_error_to_buffer(&error_msg, errbuf, errbuf_len);
            false
        }
    }
}

fn extract_variables(expression: &str) -> Vec<String> {
    use std::collections::HashSet;
    let mut variables = HashSet::new();

    // Remove strings and comments to avoid false positives
    let cleaned = remove_strings_and_literals(expression);

    // Extract identifiers that could be variables
    let mut chars = cleaned.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_alphabetic() || ch == '_' {
            // Start of an identifier
            let mut current_word = String::new();
            current_word.push(ch);

            // Continue reading the identifier (including dots for field access)
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    current_word.push(chars.next().unwrap());
                } else if next_ch == '.' {
                    // Check if this is field access
                    let mut peek_ahead = chars.clone();
                    peek_ahead.next(); // consume the dot
                    if let Some(&after_dot) = peek_ahead.peek() {
                        if after_dot.is_alphabetic() || after_dot == '_' {
                            // This is field access, only keep the root variable
                            break;
                        }
                    }
                    break;
                } else {
                    break;
                }
            }

            // Check if this is a variable (not a keyword or function)
            if !is_keyword(&current_word) && !is_builtin_function(&current_word) {
                // Look ahead to see if this is a function call
                let mut peek_chars = chars.clone();
                skip_whitespace(&mut peek_chars);

                if let Some(&'(') = peek_chars.peek() {
                    // This is a function call, not a variable
                    continue;
                }

                variables.insert(current_word.clone());
            }

            // Skip any remaining field access
            while let Some(&'.') = chars.peek() {
                chars.next(); // consume dot
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        } else if ch.is_numeric() {
            // Skip numbers
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_numeric() || next_ch == '.' {
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }

    variables.into_iter().collect()
}

fn remove_strings_and_literals(expression: &str) -> String {
    let mut result = String::new();
    let mut chars = expression.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Skip double-quoted string
                result.push(' '); // Replace with space to maintain word boundaries
                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '"' {
                        break;
                    }
                    if inner_ch == '\\' {
                        chars.next(); // Skip escaped character
                    }
                }
            }
            '\'' => {
                // Skip single-quoted string
                result.push(' ');
                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '\'' {
                        break;
                    }
                    if inner_ch == '\\' {
                        chars.next(); // Skip escaped character
                    }
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "true"
            | "false"
            | "null"
            | "in"
            | "and"
            | "or"
            | "not"
            | "has"
            | "all"
            | "exists"
            | "exists_one"
            | "map"
            | "filter"
            | "if"
            | "else"
            | "endif"
    )
}

fn is_builtin_function(word: &str) -> bool {
    matches!(
        word,
        "size"
            | "int"
            | "uint"
            | "double"
            | "string"
            | "bytes"
            | "duration"
            | "timestamp"
            | "type"
            | "dyn"
            | "has"
            | "startsWith"
            | "endsWith"
            | "contains"
            | "matches"
    )
}

fn json_to_cel_value(value: &serde_json::Value) -> Result<CelRustValue, String> {
    match value {
        serde_json::Value::Null => Ok(CelRustValue::Null),
        serde_json::Value::Bool(b) => Ok(CelRustValue::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(CelRustValue::Int(i))
            } else if let Some(u) = n.as_u64() {
                Ok(CelRustValue::UInt(u))
            } else if let Some(f) = n.as_f64() {
                Ok(CelRustValue::Float(f))
            } else {
                Err("Invalid number format".to_string())
            }
        }
        serde_json::Value::String(s) => Ok(CelRustValue::String(s.clone().into())),
        serde_json::Value::Array(arr) => {
            let cel_list: Result<Vec<_>, _> = arr.iter().map(json_to_cel_value).collect();
            Ok(CelRustValue::List(cel_list?.into()))
        }
        serde_json::Value::Object(obj) => {
            let mut cel_map = std::collections::HashMap::new();
            for (k, v) in obj {
                cel_map.insert(
                    cel_interpreter::objects::Key::String(k.clone().into()),
                    json_to_cel_value(v)?,
                );
            }
            Ok(CelRustValue::Map(cel_interpreter::objects::Map {
                map: cel_map.into(),
            }))
        }
    }
}

fn cel_value_to_c_value(value: &CelRustValue, result: *mut super::CelValue) -> Result<(), String> {
    if result.is_null() {
        return Err("Result pointer is null".to_string());
    }

    unsafe {
        match value {
            CelRustValue::Null => {
                (*result).value_type = super::CelValueType::Null;
            }
            CelRustValue::Bool(b) => {
                (*result).value_type = super::CelValueType::Bool;
                (*result).data.bool_val = *b;
            }
            CelRustValue::Int(i) => {
                (*result).value_type = super::CelValueType::Int;
                (*result).data.int_val = *i;
            }
            CelRustValue::UInt(u) => {
                (*result).value_type = super::CelValueType::Uint;
                (*result).data.uint_val = *u;
            }
            CelRustValue::Float(f) => {
                (*result).value_type = super::CelValueType::Double;
                (*result).data.double_val = *f;
            }
            CelRustValue::String(s) => {
                (*result).value_type = super::CelValueType::String;
                // Use the string pool for proper memory management
                let ptr = super::store_string_in_pool(s);
                let len = s.len();

                (*result).data.string_val = ManuallyDrop::new(super::CelStringValue { ptr, len });
            }
            CelRustValue::List(_) => {
                return Err("List return values not yet supported".to_string());
            }
            CelRustValue::Map(_) => {
                return Err("Map return values not yet supported".to_string());
            }
            _ => return Err("Unsupported return value type".to_string()),
        }
    }

    Ok(())
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
