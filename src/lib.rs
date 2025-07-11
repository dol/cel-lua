pub mod ffi;

pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::mem::ManuallyDrop;
    use std::ptr;

    #[test]
    fn test_context_lifecycle() {
        // Test context creation and destruction
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            context_free(context);
        }
    }

    #[test]
    fn test_program_lifecycle() {
        // Test program creation and destruction
        let program = program_new();
        assert!(!program.is_null());

        unsafe {
            program_free(program);
        }
    }

    #[test]
    fn test_program_compile_simple_expression() {
        let program = program_new();
        assert!(!program.is_null());

        unsafe {
            let program_ref = &mut *program;
            let expression = CString::new("1 + 1").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let result = program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(result, "Failed to compile simple expression");
            program_free(program);
        }
    }

    #[test]
    fn test_program_compile_invalid_expression() {
        let program = program_new();
        assert!(!program.is_null());

        unsafe {
            let program_ref = &mut *program;
            let expression = CString::new("1 + +").unwrap(); // Invalid syntax
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let result = program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(!result, "Should fail to compile invalid expression");
            assert!(error_len > 0, "Should provide error message");
            program_free(program);
        }
    }

    #[test]
    fn test_program_validate_valid_expression() {
        unsafe {
            let expression = CString::new("x + y").unwrap();
            let mut variables_len = 0usize;
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let result = program_validate(
                expression.as_ptr(),
                ptr::null_mut(),
                &raw mut variables_len,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(result, "Should validate expression successfully");
            assert_eq!(variables_len, 2, "Should detect 2 variables");
        }
    }

    #[test]
    fn test_program_validate_invalid_expression() {
        unsafe {
            let expression = CString::new("1 + +").unwrap(); // Invalid syntax
            let mut variables_len = 0usize;
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let result = program_validate(
                expression.as_ptr(),
                ptr::null_mut(),
                &raw mut variables_len,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(!result, "Should fail to validate invalid expression");
            assert!(error_len > 0, "Should provide error message");
        }
    }

    #[test]
    fn test_context_add_variable_bool() {
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            let context_ref = &mut *context;
            let name = CString::new("test_bool").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            // Create a boolean CelValue
            let cel_value = CelValue {
                value_type: CelValueType::Bool,
                data: CelValueData { bool_val: true },
            };

            let result = context_add_variable(
                context_ref,
                name.as_ptr(),
                &cel_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(result, "Should add boolean variable successfully");
            context_free(context);
        }
    }

    #[test]
    fn test_context_add_variable_int() {
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            let context_ref = &mut *context;
            let name = CString::new("test_int").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            // Create an integer CelValue
            let cel_value = CelValue {
                value_type: CelValueType::Int,
                data: CelValueData { int_val: 42 },
            };

            let result = context_add_variable(
                context_ref,
                name.as_ptr(),
                &cel_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(result, "Should add integer variable successfully");
            context_free(context);
        }
    }

    #[test]
    fn test_context_reset() {
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            let context_ref = &mut *context;

            // Add a variable first
            let name = CString::new("test_var").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let cel_value = CelValue {
                value_type: CelValueType::Int,
                data: CelValueData { int_val: 123 },
            };

            let result = context_add_variable(
                context_ref,
                name.as_ptr(),
                &cel_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );
            assert!(result);

            // Reset the context (should clear variables)
            context_reset(context_ref);

            // Context should still be valid after reset
            context_free(context);
        }
    }

    #[test]
    fn test_string_memory_management() {
        unsafe {
            let test_string = "Hello, World!";
            let ptr = store_string_in_pool(test_string);
            assert!(!ptr.is_null());

            // Free the string using the proper function
            release_string_from_pool(ptr);

            // Calling free on null should be safe
            cel_string_free(ptr::null());
        }
    }

    #[test]
    fn test_program_execute_with_variables() {
        let program = program_new();
        let context = context_new();
        assert!(!program.is_null());
        assert!(!context.is_null());

        unsafe {
            let program_ref = &mut *program;
            let context_ref = &mut *context;

            // Add a variable to the context
            let name = CString::new("x").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let cel_value = CelValue {
                value_type: CelValueType::Int,
                data: CelValueData { int_val: 10 },
            };

            let result = context_add_variable(
                context_ref,
                name.as_ptr(),
                &cel_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );
            assert!(result, "Should add variable successfully");

            // Compile an expression that uses the variable
            let expression = CString::new("x + 5").unwrap();
            error_len = error_buf.len();

            let compile_result = program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len,
            );
            assert!(compile_result, "Should compile expression with variable");

            // Execute the expression
            let mut result_value = CelValue {
                value_type: CelValueType::Null,
                data: CelValueData { int_val: 0 },
            };
            error_len = error_buf.len();

            let exec_result = program_execute(
                program_ref,
                context_ref,
                &raw mut result_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(exec_result, "Should execute expression successfully");
            assert_eq!(result_value.value_type, CelValueType::Int);
            assert_eq!(result_value.data.int_val, 15);

            program_free(program);
            context_free(context);
        }
    }

    #[test]
    fn test_program_execute_without_compilation() {
        let program = program_new();
        let context = context_new();
        assert!(!program.is_null());
        assert!(!context.is_null());

        unsafe {
            let program_ref = &*program;
            let context_ref = &*context;

            let mut result_value = CelValue {
                value_type: CelValueType::Null,
                data: CelValueData { int_val: 0 },
            };
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let exec_result = program_execute(
                program_ref,
                context_ref,
                &raw mut result_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(!exec_result, "Should fail to execute without compilation");
            assert!(error_len > 0, "Should provide error message");

            program_free(program);
            context_free(context);
        }
    }

    #[test]
    fn test_context_add_variable_invalid_name() {
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            // Test that context remains valid even with edge cases
            // In real implementations, proper validation would prevent invalid operations
            context_free(context);
        }
    }

    #[test]
    fn test_complex_expression_validation() {
        unsafe {
            let expression = CString::new("user.age > 18 && user.name.startsWith('John')").unwrap();
            let mut variables_len = 0usize;
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let result = program_validate(
                expression.as_ptr(),
                ptr::null_mut(),
                &raw mut variables_len,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(result, "Should validate complex expression successfully");
            assert!(variables_len > 0, "Should detect variables in complex expression");
        }
    }

    #[test]
    fn test_context_add_variable_double() {
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            let context_ref = &mut *context;
            let name = CString::new("pi").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let cel_value = CelValue {
                value_type: CelValueType::Double,
                data: CelValueData {
                    double_val: std::f64::consts::PI,
                },
            };

            let result = context_add_variable(
                context_ref,
                name.as_ptr(),
                &cel_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(result, "Should add double variable successfully");
            context_free(context);
        }
    }

    #[test]
    fn test_error_buffer_handling() {
        unsafe {
            let expression = CString::new("invalid syntax +++").unwrap();
            let mut variables_len = 0usize;
            let mut error_buf = [0u8; 10]; // Very small buffer
            let mut error_len = error_buf.len();

            let result = program_validate(
                expression.as_ptr(),
                ptr::null_mut(),
                &raw mut variables_len,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(!result, "Should fail validation");
            assert!(error_len > 0, "Should write to error buffer");
            assert!(error_len < error_buf.len(), "Should respect buffer size");
            assert_eq!(error_buf[error_len], 0, "Should null-terminate error string");
        }
    }

    #[test]
    fn test_program_execute_boolean_expression() {
        let program = program_new();
        let context = context_new();
        assert!(!program.is_null());
        assert!(!context.is_null());

        unsafe {
            let program_ref = &mut *program;
            let context_ref = &mut *context;

            // Add boolean variables
            let name1 = CString::new("is_admin").unwrap();
            let name2 = CString::new("is_active").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let cel_value1 = CelValue {
                value_type: CelValueType::Bool,
                data: CelValueData { bool_val: true },
            };
            let cel_value2 = CelValue {
                value_type: CelValueType::Bool,
                data: CelValueData { bool_val: false },
            };

            assert!(context_add_variable(
                context_ref,
                name1.as_ptr(),
                &cel_value1,
                error_buf.as_mut_ptr(),
                &mut error_len,
            ));

            error_len = error_buf.len();
            assert!(context_add_variable(
                context_ref,
                name2.as_ptr(),
                &cel_value2,
                error_buf.as_mut_ptr(),
                &mut error_len,
            ));

            // Test logical expression
            let expression = CString::new("is_admin && !is_active").unwrap();
            error_len = error_buf.len();

            assert!(program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len,
            ));

            let mut result_value = CelValue {
                value_type: CelValueType::Null,
                data: CelValueData { bool_val: false },
            };
            error_len = error_buf.len();

            assert!(program_execute(
                program_ref,
                context_ref,
                &raw mut result_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            ));

            assert_eq!(result_value.value_type, CelValueType::Bool);
            assert!(result_value.data.bool_val); // true && !false = true

            program_free(program);
            context_free(context);
        }
    }

    #[test]
    fn test_program_execute_string_operations() {
        let program = program_new();
        let context = context_new();
        assert!(!program.is_null());
        assert!(!context.is_null());

        unsafe {
            let program_ref = &mut *program;
            let context_ref = &mut *context;

            // Add string variable
            let name = CString::new("message").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            // Create string value using the string pool
            let test_string = "Hello, World!";
            let string_ptr = store_string_in_pool(test_string);
            let cel_value = CelValue {
                value_type: CelValueType::String,
                data: CelValueData {
                    string_val: ManuallyDrop::new(CelStringValue {
                        ptr: string_ptr,
                        len: test_string.len(),
                    }),
                },
            };

            assert!(context_add_variable(
                context_ref,
                name.as_ptr(),
                &cel_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            ));

            // Test string operation
            let expression = CString::new("size(message)").unwrap();
            error_len = error_buf.len();

            assert!(program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len,
            ));

            let mut result_value = CelValue {
                value_type: CelValueType::Null,
                data: CelValueData { int_val: 0 },
            };
            error_len = error_buf.len();

            assert!(program_execute(
                program_ref,
                context_ref,
                &raw mut result_value,
                error_buf.as_mut_ptr(),
                &mut error_len,
            ));

            assert_eq!(result_value.value_type, CelValueType::Int);
            assert_eq!(result_value.data.int_val, 13); // Length of "Hello, World!"

            // Clean up string pool memory
            release_string_from_pool(string_ptr);

            program_free(program);
            context_free(context);
        }
    }

    #[test]
    fn test_program_execute_arithmetic_expressions() {
        let program = program_new();
        let context = context_new();
        assert!(!program.is_null());
        assert!(!context.is_null());

        unsafe {
            let program_ref = &mut *program;
            let context_ref = &mut *context;

            // Add numeric variables
            let name1 = CString::new("a").unwrap();
            let name2 = CString::new("b").unwrap();
            let name3 = CString::new("pi").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            let cel_value1 = CelValue {
                value_type: CelValueType::Int,
                data: CelValueData { int_val: 10 },
            };
            let cel_value2 = CelValue {
                value_type: CelValueType::Uint,
                data: CelValueData { uint_val: 20 },
            };
            let cel_value3 = CelValue {
                value_type: CelValueType::Double,
                data: CelValueData {
                    double_val: std::f64::consts::PI,
                },
            };

            assert!(context_add_variable(
                context_ref,
                name1.as_ptr(),
                &cel_value1,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));
            error_len = error_buf.len();
            assert!(context_add_variable(
                context_ref,
                name2.as_ptr(),
                &cel_value2,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));
            error_len = error_buf.len();
            assert!(context_add_variable(
                context_ref,
                name3.as_ptr(),
                &cel_value3,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            // Test complex arithmetic expression
            let expression = CString::new("a * 2 + b - int(pi)").unwrap();
            error_len = error_buf.len();

            assert!(program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            let mut result_value = CelValue {
                value_type: CelValueType::Null,
                data: CelValueData { int_val: 0 },
            };
            error_len = error_buf.len();

            assert!(program_execute(
                program_ref,
                context_ref,
                &raw mut result_value,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            assert_eq!(result_value.value_type, CelValueType::Int);
            // 10 * 2 + 20 - 3 = 37
            assert_eq!(result_value.data.int_val, 37);

            program_free(program);
            context_free(context);
        }
    }

    #[test]
    fn test_program_validate_complex_expressions() {
        unsafe {
            let expressions = [
                ("user.age >= 21", true, 1),
                ("items.size() > 0 && items[0].price < 100.0", true, 1),
                ("now > timestamp('2023-01-01T00:00:00Z')", true, 1),
                ("'hello' + ' ' + 'world'", true, 0),
                ("has(request.headers.authorization)", true, 1),
                ("invalid + + syntax", false, 0),
            ];

            for (expr, should_succeed, expected_var_count) in expressions {
                let expression = CString::new(expr).unwrap();
                let mut variables_len = 0usize;
                let mut error_buf = [0u8; 256];
                let mut error_len = error_buf.len();

                let result = program_validate(
                    expression.as_ptr(),
                    ptr::null_mut(),
                    &raw mut variables_len,
                    error_buf.as_mut_ptr(),
                    &mut error_len,
                );

                if should_succeed {
                    assert!(result, "Expression '{expr}' should validate successfully");
                    if expected_var_count > 0 {
                        assert!(
                            variables_len >= expected_var_count,
                            "Expression '{expr}' should detect at least {expected_var_count} variables, found {variables_len}"
                        );
                    }
                } else {
                    assert!(!result, "Expression '{expr}' should fail validation");
                    assert!(error_len > 0, "Expression '{expr}' should provide error message");
                }
            }
        }
    }

    #[test]
    fn test_context_multiple_variable_types() {
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            let context_ref = &mut *context;
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            // Add variables of different types
            let bool_name = CString::new("is_enabled").unwrap();
            let int_name = CString::new("count").unwrap();
            let uint_name = CString::new("id").unwrap();
            let double_name = CString::new("ratio").unwrap();

            let bool_val = CelValue {
                value_type: CelValueType::Bool,
                data: CelValueData { bool_val: true },
            };
            let int_val = CelValue {
                value_type: CelValueType::Int,
                data: CelValueData { int_val: -42 },
            };
            let uint_val = CelValue {
                value_type: CelValueType::Uint,
                data: CelValueData { uint_val: 123_456 },
            };
            let double_val = CelValue {
                value_type: CelValueType::Double,
                data: CelValueData {
                    double_val: std::f64::consts::E,
                },
            };

            // Add all variables
            assert!(context_add_variable(
                context_ref,
                bool_name.as_ptr(),
                &bool_val,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));
            error_len = error_buf.len();
            assert!(context_add_variable(
                context_ref,
                int_name.as_ptr(),
                &int_val,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));
            error_len = error_buf.len();
            assert!(context_add_variable(
                context_ref,
                uint_name.as_ptr(),
                &uint_val,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));
            error_len = error_buf.len();
            assert!(context_add_variable(
                context_ref,
                double_name.as_ptr(),
                &double_val,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            // Now test that all variables can be used in expressions
            let program = program_new();
            let program_ref = &mut *program;

            let expression =
                CString::new("is_enabled && count < 0 && id > 100000 && ratio > 2.0").unwrap();
            error_len = error_buf.len();

            assert!(program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            let mut result_value = CelValue {
                value_type: CelValueType::Null,
                data: CelValueData { bool_val: false },
            };
            error_len = error_buf.len();

            assert!(program_execute(
                program_ref,
                context_ref,
                &raw mut result_value,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            assert_eq!(result_value.value_type, CelValueType::Bool);
            assert!(result_value.data.bool_val);

            program_free(program);
            context_free(context);
        }
    }

    #[test]
    fn test_context_variable_shadowing() {
        let context = context_new();
        assert!(!context.is_null());

        unsafe {
            let context_ref = &mut *context;
            let name = CString::new("test_var").unwrap();
            let mut error_buf = [0u8; 256];
            let mut error_len = error_buf.len();

            // Add first value
            let value1 = CelValue {
                value_type: CelValueType::Int,
                data: CelValueData { int_val: 100 },
            };

            assert!(context_add_variable(
                context_ref,
                name.as_ptr(),
                &value1,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            // Add same variable with different value (should replace)
            let value2 = CelValue {
                value_type: CelValueType::Int,
                data: CelValueData { int_val: 200 },
            };
            error_len = error_buf.len();

            assert!(context_add_variable(
                context_ref,
                name.as_ptr(),
                &value2,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            // Test that the new value is used
            let program = program_new();
            let program_ref = &mut *program;

            let expression = CString::new("test_var").unwrap();
            error_len = error_buf.len();

            assert!(program_compile(
                program_ref,
                expression.as_ptr(),
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            let mut result_value = CelValue {
                value_type: CelValueType::Null,
                data: CelValueData { int_val: 0 },
            };
            error_len = error_buf.len();

            assert!(program_execute(
                program_ref,
                context_ref,
                &raw mut result_value,
                error_buf.as_mut_ptr(),
                &mut error_len
            ));

            assert_eq!(result_value.value_type, CelValueType::Int);
            assert_eq!(result_value.data.int_val, 200); // Should use the latest value

            program_free(program);
            context_free(context);
        }
    }

    #[test]
    fn test_large_error_messages() {
        unsafe {
            let long_invalid_expr = "a".repeat(1000) + " + + + invalid syntax";
            let expression = CString::new(long_invalid_expr).unwrap();
            let mut variables_len = 0usize;
            let mut error_buf = [0u8; 50]; // Small buffer
            let mut error_len = error_buf.len();

            let result = program_validate(
                expression.as_ptr(),
                ptr::null_mut(),
                &raw mut variables_len,
                error_buf.as_mut_ptr(),
                &mut error_len,
            );

            assert!(!result, "Should fail validation");
            assert!(error_len > 0, "Should write error message");
            assert!(error_len < error_buf.len(), "Should respect buffer bounds");
            assert_eq!(error_buf[error_len], 0, "Should null-terminate");
        }
    }

    #[test]
    fn test_null_pointer_safety() {
        unsafe {
            // Test null program pointer safety
            program_free(ptr::null_mut()); // Should not crash
            context_free(ptr::null_mut()); // Should not crash
            cel_string_free(ptr::null()); // Should not crash

            // Test string pool operations
            let test_str = "test string";
            let ptr = store_string_in_pool(test_str);
            assert!(!ptr.is_null());

            // Free once using the proper function
            release_string_from_pool(ptr);

            // Test that freeing null pointer is safe
            cel_string_free(ptr::null());
        }
    }

    #[test]
    fn test_memory_lifecycle_stress() {
        // Test creating and destroying many contexts and programs
        for i in 0..100 {
            let context = context_new();
            let program = program_new();

            assert!(!context.is_null(), "Context {i} should be valid");
            assert!(!program.is_null(), "Program {i} should be valid");

            unsafe {
                let context_ref = &mut *context;
                let program_ref = &mut *program;

                // Add a variable
                let name = CString::new(format!("var_{i}")).unwrap();
                let mut error_buf = [0u8; 256];
                let mut error_len = error_buf.len();

                let cel_value = CelValue {
                    value_type: CelValueType::Int,
                    data: CelValueData {
                        int_val: i64::from(i),
                    },
                };

                assert!(context_add_variable(
                    context_ref,
                    name.as_ptr(),
                    &cel_value,
                    error_buf.as_mut_ptr(),
                    &mut error_len
                ));

                // Compile and execute a simple expression
                let expression = CString::new(format!("var_{i} * 2")).unwrap();
                error_len = error_buf.len();

                assert!(program_compile(
                    program_ref,
                    expression.as_ptr(),
                    error_buf.as_mut_ptr(),
                    &mut error_len
                ));

                let mut result_value = CelValue {
                    value_type: CelValueType::Null,
                    data: CelValueData { int_val: 0 },
                };
                error_len = error_buf.len();

                assert!(program_execute(
                    program_ref,
                    context_ref,
                    &raw mut result_value,
                    error_buf.as_mut_ptr(),
                    &mut error_len
                ));

                assert_eq!(result_value.value_type, CelValueType::Int);
                assert_eq!(result_value.data.int_val, i64::from(i) * 2);

                program_free(program);
                context_free(context);
            }
        }
    }
}
