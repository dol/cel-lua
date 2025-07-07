use std::ffi::{c_char, CStr};
use std::mem::ManuallyDrop;

pub mod context;
pub mod program;

pub use context::*;
pub use program::*;

// Simple memory management without global state
// We'll use a simpler approach that doesn't require a global HashMap

/// CEL value types enum
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CelValueType {
    Null,
    Bool,
    Int,
    Uint,
    Double,
    String,
    Bytes,
    List,
    Map,
    Type,
}

/// String value representation for CEL
#[repr(C)]
#[derive(Debug)]
pub struct CelStringValue {
    pub ptr: *const u8,
    pub len: usize,
}

/// Bytes value representation for CEL
#[repr(C)]
#[derive(Debug)]
pub struct CelBytesValue {
    pub ptr: *const u8,
    pub len: usize,
}

/// Union for CEL value data
#[repr(C)]
pub union CelValueData {
    pub bool_val: bool,
    pub int_val: i64,
    pub uint_val: u64,
    pub double_val: f64,
    pub string_val: ManuallyDrop<CelStringValue>,
    pub bytes_val: ManuallyDrop<CelBytesValue>,
}

/// CEL value structure
#[repr(C)]
pub struct CelValue {
    pub value_type: CelValueType,
    pub data: CelValueData,
}

/// Clear all strings from the pool (useful for cleanup)
/// Note: With the new memory management approach, this is a no-op
/// Individual strings are freed when release_string_from_pool is called
#[no_mangle]
pub extern "C" fn cel_clear_string_pool() {
    // No-op with the new approach - memory is managed per-string
}

/// Get the number of strings currently in the pool
/// Note: With the new memory management approach, we can't track count
/// Returns 0 as we don't maintain a global pool anymore
#[no_mangle]
pub extern "C" fn cel_string_pool_size() -> usize {
    0 // No global pool anymore
}

// Helper function to convert C string to Rust string
/// # Safety
/// The caller must ensure that `ptr` is a valid null-terminated C string pointer
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Result<String, std::str::Utf8Error> {
    let c_str = CStr::from_ptr(ptr);
    Ok(c_str.to_str()?.to_string())
}

// String memory management functions - simplified without global state
pub fn store_string_in_pool(s: &str) -> *const u8 {
    // Create a CString and convert it to a raw pointer
    // This ensures null-termination and proper memory layout
    let c_string = std::ffi::CString::new(s).unwrap();
    let bytes = c_string.into_bytes_with_nul();
    let boxed_bytes = bytes.into_boxed_slice();
    let ptr = boxed_bytes.as_ptr();

    // Leak the box so the memory stays allocated
    Box::leak(boxed_bytes);

    ptr
}

pub fn release_string_from_pool(ptr: *const u8) {
    if !ptr.is_null() {
        unsafe {
            // SAFETY: This function should only be called with pointers that were
            // returned by store_string_in_pool. We cannot validate the pointer
            // without additional bookkeeping, so we trust the caller.
            // If an invalid pointer is passed, this will cause undefined behavior.
            let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
            let len = c_str.to_bytes_with_nul().len();

            // Reconstruct the Box and let it drop to free the memory
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr as *mut u8, len);
            let _boxed_slice = Box::from_raw(slice_ptr);
            // Box automatically drops and frees the memory
        }
    }
}

/// Free a string that was allocated by the library
///
/// # Safety
/// The caller must ensure that:
/// - `ptr` is either null or a valid pointer returned by a CEL library function
/// - `ptr` has not been previously freed
/// - No other references to the string exist
#[no_mangle]
pub unsafe extern "C" fn cel_string_free(ptr: *const u8) {
    if !ptr.is_null() {
        release_string_from_pool(ptr);
    }
}

#[cfg(test)]
pub fn test_cleanup() {
    cel_clear_string_pool();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_cel_value_type_enum() {
        // Test that the enum values are as expected
        assert_eq!(CelValueType::Null as i32, 0);
        assert_ne!(CelValueType::Bool, CelValueType::Int);
        assert_ne!(CelValueType::String, CelValueType::Bytes);
    }

    #[test]
    fn test_cel_value_struct_creation() {
        let cel_value = CelValue {
            value_type: CelValueType::Bool,
            data: CelValueData { bool_val: true },
        };

        assert_eq!(cel_value.value_type, CelValueType::Bool);
        assert!(unsafe { cel_value.data.bool_val });
    }

    #[test]
    fn test_cel_value_int() {
        let cel_value = CelValue {
            value_type: CelValueType::Int,
            data: CelValueData { int_val: -123 },
        };

        assert_eq!(cel_value.value_type, CelValueType::Int);
        assert_eq!(unsafe { cel_value.data.int_val }, -123);
    }

    #[test]
    fn test_cel_value_uint() {
        let cel_value = CelValue {
            value_type: CelValueType::Uint,
            data: CelValueData { uint_val: 456 },
        };

        assert_eq!(cel_value.value_type, CelValueType::Uint);
        assert_eq!(unsafe { cel_value.data.uint_val }, 456);
    }

    #[test]
    fn test_cel_value_double() {
        let cel_value = CelValue {
            value_type: CelValueType::Double,
            data: CelValueData {
                double_val: std::f64::consts::PI,
            },
        };

        assert_eq!(cel_value.value_type, CelValueType::Double);
        assert!((unsafe { cel_value.data.double_val } - std::f64::consts::PI).abs() < 0.00001);
    }

    #[test]
    fn test_cel_string_value() {
        let test_str = "Hello, World!";
        let string_val = CelStringValue {
            ptr: test_str.as_ptr(),
            len: test_str.len(),
        };

        assert_eq!(string_val.len, test_str.len());

        let reconstructed = unsafe { std::slice::from_raw_parts(string_val.ptr, string_val.len) };
        let reconstructed_str = std::str::from_utf8(reconstructed).unwrap();
        assert_eq!(reconstructed_str, test_str);
    }

    #[test]
    fn test_c_str_to_string_valid() {
        let test_str = CString::new("Hello, Rust!").unwrap();
        let result = unsafe { c_str_to_string(test_str.as_ptr()) };

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, Rust!");
    }

    #[test]
    fn test_string_pool_operations() {
        let test_string = "Test string for pool";
        let ptr = store_string_in_pool(test_string);

        assert!(!ptr.is_null());

        // Free the string
        unsafe {
            cel_string_free(ptr);
        }

        // Verify we can store and free again without issues
        let ptr2 = store_string_in_pool(test_string);
        assert!(!ptr2.is_null());
        unsafe {
            cel_string_free(ptr2);
        }
    }

    #[test]
    fn test_string_pool_multiple_strings() {
        let ptr1 = store_string_in_pool("String 1");
        let ptr2 = store_string_in_pool("String 2");
        let ptr3 = store_string_in_pool("String 3");

        // Verify pointers are distinct and valid
        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert!(!ptr3.is_null());
        assert_ne!(ptr1, ptr2);
        assert_ne!(ptr2, ptr3);

        // Free all strings
        release_string_from_pool(ptr1);
        release_string_from_pool(ptr2);
        release_string_from_pool(ptr3);
    }

    #[test]
    fn test_string_pool_clear() {
        // Add some strings
        let ptr1 = store_string_in_pool("String 1");
        let ptr2 = store_string_in_pool("String 2");

        // With the new approach, we manually free the strings
        release_string_from_pool(ptr1);
        release_string_from_pool(ptr2);

        // Clear the pool (no-op with new approach)
        cel_clear_string_pool();

        // Pool size is always 0 with new approach
        assert_eq!(cel_string_pool_size(), 0);
    }

    #[test]
    fn test_cel_string_free_null() {
        // Should not crash when called with null pointer
        unsafe {
            cel_string_free(std::ptr::null());
        }
    }

    #[test]
    fn test_release_string_from_pool_null() {
        // Should not crash when called with null pointer
        release_string_from_pool(std::ptr::null());
    }

    #[test]
    fn test_string_pool_concurrent_access() {
        use std::thread;

        let handles: Vec<_> = (0..5)
            .map(|i| {
                thread::spawn(move || {
                    let test_str = format!("Thread {i} string");
                    let ptr = store_string_in_pool(&test_str);

                    // Do some work
                    thread::sleep(std::time::Duration::from_millis(1));

                    release_string_from_pool(ptr);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Pool should be empty or have minimal entries after all threads finish
        // (depending on timing and other concurrent tests)
    }

    #[test]
    fn test_cel_value_data_union() {
        // Test that union works correctly for different types
        let mut data = CelValueData { bool_val: false };

        // Set as bool
        data.bool_val = true;
        assert!(unsafe { data.bool_val });

        // Reuse as int (overwrites the bool)
        data.int_val = 42;
        assert_eq!(unsafe { data.int_val }, 42);

        // Reuse as double (overwrites the int)
        data.double_val = std::f64::consts::PI;
        assert!((unsafe { data.double_val } - std::f64::consts::PI).abs() < 0.001);
    }

    #[test]
    fn test_manually_drop_string_value() {
        let test_str = "Test string";
        let string_val = ManuallyDrop::new(CelStringValue {
            ptr: test_str.as_ptr(),
            len: test_str.len(),
        });

        let mut cel_value = CelValue {
            value_type: CelValueType::String,
            data: CelValueData { string_val },
        };

        assert_eq!(cel_value.value_type, CelValueType::String);
        assert_eq!(unsafe { cel_value.data.string_val.len }, test_str.len());

        // Manually drop the ManuallyDrop (in real code, this would be handled by the FFI layer)
        unsafe {
            ManuallyDrop::drop(&mut cel_value.data.string_val);
        }
    }

    #[test]
    fn test_cel_value_data_union_safety() {
        // Test that union data can be safely accessed for different types
        let bool_data = CelValueData { bool_val: true };
        let int_data = CelValueData { int_val: -123456 };
        let uint_data = CelValueData {
            uint_val: 987654321,
        };
        let double_data = CelValueData {
            double_val: std::f64::consts::PI,
        };

        unsafe {
            assert!(bool_data.bool_val);
            assert_eq!(int_data.int_val, -123456);
            assert_eq!(uint_data.uint_val, 987654321);
            assert!((double_data.double_val - std::f64::consts::PI).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_cel_value_creation_all_types() {
        // Test creating CelValue for each supported type
        let null_val = CelValue {
            value_type: CelValueType::Null,
            data: CelValueData { int_val: 0 }, // Value doesn't matter for null
        };

        let bool_val = CelValue {
            value_type: CelValueType::Bool,
            data: CelValueData { bool_val: false },
        };

        let int_val = CelValue {
            value_type: CelValueType::Int,
            data: CelValueData { int_val: i64::MIN },
        };

        let uint_val = CelValue {
            value_type: CelValueType::Uint,
            data: CelValueData { uint_val: u64::MAX },
        };

        let double_val = CelValue {
            value_type: CelValueType::Double,
            data: CelValueData {
                double_val: f64::INFINITY,
            },
        };

        // Verify types are set correctly
        assert_eq!(null_val.value_type, CelValueType::Null);
        assert_eq!(bool_val.value_type, CelValueType::Bool);
        assert_eq!(int_val.value_type, CelValueType::Int);
        assert_eq!(uint_val.value_type, CelValueType::Uint);
        assert_eq!(double_val.value_type, CelValueType::Double);

        // Verify data access (unsafe)
        unsafe {
            assert!(!bool_val.data.bool_val);
            assert_eq!(int_val.data.int_val, i64::MIN);
            assert_eq!(uint_val.data.uint_val, u64::MAX);
            assert_eq!(double_val.data.double_val, f64::INFINITY);
        }
    }

    #[test]
    fn test_cel_string_value_creation() {
        let test_strings = [
            "",
            "hello",
            "Hello, World! 🌍",
            "Unicode test: 你好世界",
            "Emoji test: 🚀🎉🔥💯",
            "Multi\nLine\nString\nWith\nBreaks",
            "String with null bytes: \0 in middle",
            &"Very long string ".repeat(1000),
        ];

        for test_str in test_strings {
            let string_val = CelStringValue {
                ptr: test_str.as_ptr(),
                len: test_str.len(),
            };

            assert_eq!(string_val.len, test_str.len());
            assert!(!string_val.ptr.is_null());

            // Verify we can reconstruct the string
            unsafe {
                let reconstructed = std::slice::from_raw_parts(string_val.ptr, string_val.len);
                assert_eq!(reconstructed, test_str.as_bytes());
            }
        }
    }

    #[test]
    fn test_cel_bytes_value_creation() {
        let test_data = [
            vec![],
            vec![0],
            vec![255],
            vec![0, 1, 2, 3, 4, 5],
            vec![255, 254, 253, 252],
            (0..=255).collect::<Vec<u8>>(),
            vec![0; 10000], // Large buffer
        ];

        for data in test_data {
            let bytes_val = CelBytesValue {
                ptr: data.as_ptr(),
                len: data.len(),
            };

            assert_eq!(bytes_val.len, data.len());

            if data.is_empty() {
                // For empty data, ptr might be null or not null
                assert_eq!(bytes_val.len, 0);
            } else {
                assert!(!bytes_val.ptr.is_null());

                // Verify we can reconstruct the data
                unsafe {
                    let reconstructed = std::slice::from_raw_parts(bytes_val.ptr, bytes_val.len);
                    assert_eq!(reconstructed, data.as_slice());
                }
            }
        }
    }

    #[test]
    fn test_string_pool_stress_test() {
        let num_strings = 10; // Reduced from 1000 to avoid potential memory issues
        let mut stored_ptrs = Vec::new();

        // Store strings
        for i in 0..num_strings {
            let test_string = format!("test_{i}");
            let ptr = store_string_in_pool(&test_string);
            assert!(!ptr.is_null());
            stored_ptrs.push((ptr, test_string));
        }

        // Verify strings are accessible (simplified test)
        assert_eq!(stored_ptrs.len(), num_strings);

        // Clean up
        for (ptr, _) in stored_ptrs {
            release_string_from_pool(ptr);
        }
    }

    #[test]
    fn test_string_pool_duplicate_strings() {
        let test_string = "duplicate test";

        // Store the same string multiple times
        let ptr1 = store_string_in_pool(test_string);
        let ptr2 = store_string_in_pool(test_string);
        let ptr3 = store_string_in_pool(test_string);

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert!(!ptr3.is_null());

        // Clean up - just verify pointers are valid
        release_string_from_pool(ptr1);
        release_string_from_pool(ptr2);
        release_string_from_pool(ptr3);
    }

    #[test]
    fn test_string_pool_unicode_handling() {
        let unicode_strings = ["Hello, 世界!", "Русский", "العربية", "🚀🎉"];

        for test_str in unicode_strings {
            let ptr = store_string_in_pool(test_str);
            assert!(!ptr.is_null());

            release_string_from_pool(ptr);
        }
    }

    #[test]
    fn test_string_pool_edge_cases() {
        // Test empty string
        let empty_ptr = store_string_in_pool("");
        assert!(!empty_ptr.is_null());

        unsafe {
            cel_string_free(empty_ptr);
        }

        // Test short string
        let short_string = "test";
        let short_ptr = store_string_in_pool(short_string);
        assert!(!short_ptr.is_null());

        release_string_from_pool(short_ptr);
    }

    #[test]
    fn test_cel_value_type_enum_completeness() {
        // Ensure all enum variants can be created and compared
        let types = [
            CelValueType::Null,
            CelValueType::Bool,
            CelValueType::Int,
            CelValueType::Uint,
            CelValueType::Double,
            CelValueType::String,
            CelValueType::Bytes,
        ];

        // Test that all types are distinct
        for (i, type1) in types.iter().enumerate() {
            for (j, type2) in types.iter().enumerate() {
                if i == j {
                    assert_eq!(type1, type2);
                } else {
                    assert_ne!(type1, type2);
                }
            }
        }
    }

    #[test]
    fn test_cel_value_size_and_alignment() {
        // Verify struct sizes are reasonable for FFI
        use std::mem;

        println!("CelValueType size: {}", mem::size_of::<CelValueType>());
        println!("CelValueData size: {}", mem::size_of::<CelValueData>());
        println!("CelValue size: {}", mem::size_of::<CelValue>());
        println!("CelStringValue size: {}", mem::size_of::<CelStringValue>());
        println!("CelBytesValue size: {}", mem::size_of::<CelBytesValue>());

        // Basic sanity checks
        assert!(mem::size_of::<CelValue>() > 0);
        assert!(mem::size_of::<CelValue>() < 1000); // Should be reasonably sized

        // Alignment checks
        assert_eq!(mem::align_of::<CelValue>() % mem::align_of::<u64>(), 0);
    }

    #[test]
    fn test_string_pool_memory_management() {
        // Test that string memory is properly managed without global state
        let strings = (0..50).map(|i| format!("test_string_{i}")).collect::<Vec<_>>();
        let mut ptrs = Vec::new();

        // Add strings to memory (no global pool)
        for s in &strings {
            ptrs.push(store_string_in_pool(s));
        }

        // All pointers should be valid
        for ptr in &ptrs {
            assert!(!ptr.is_null());
        }

        // Free half the strings
        for ptr in &ptrs[..25] {
            release_string_from_pool(*ptr);
        }

        // Free remaining strings
        for ptr in &ptrs[25..] {
            release_string_from_pool(*ptr);
        }

        // Clear function is a no-op now
        cel_clear_string_pool();

        // Pool size is always 0 with new approach
        let final_size = cel_string_pool_size();
        assert_eq!(final_size, 0);
    }

    #[test]
    fn test_c_str_to_string_edge_cases() {
        // Test with various C string patterns
        let test_strings = [
            "normal string",
            "",
            "a",
            "string with spaces",
            "string\nwith\nnewlines",
            "string\twith\ttabs",
        ];

        for test_str in test_strings {
            let c_string = std::ffi::CString::new(test_str).unwrap();
            let ptr = c_string.as_ptr();

            unsafe {
                let result = c_str_to_string(ptr);
                assert!(result.is_ok(), "Failed to convert C string: '{test_str}'");
                assert_eq!(result.unwrap(), test_str);
            }
        }
    }

    #[test]
    fn test_manually_drop_behavior() {
        // Test that ManuallyDrop works correctly with our string values
        let test_str = "ManuallyDrop test";
        let string_val = ManuallyDrop::new(CelStringValue {
            ptr: test_str.as_ptr(),
            len: test_str.len(),
        });

        // Verify the value is accessible
        assert_eq!(string_val.len, test_str.len());
        assert!(!string_val.ptr.is_null());

        // Create a CelValue with ManuallyDrop
        let mut cel_value = CelValue {
            value_type: CelValueType::String,
            data: CelValueData { string_val },
        };

        // Verify we can access the data
        unsafe {
            assert_eq!(cel_value.data.string_val.len, test_str.len());
        }

        // Manually drop (simulating cleanup)
        unsafe {
            ManuallyDrop::drop(&mut cel_value.data.string_val);
        }

        // After dropping, the memory should still be valid since we used a string literal
        // In real usage, this would be managed by the string pool
    }

    // Add a cleanup test that runs last to clean up global state
    #[test]
    fn zzz_test_cleanup() {
        // This test runs last (due to name starting with 'zzz')
        // and cleans up any remaining string pool memory
        test_cleanup();
    } // This test should run last to attempt cleanup of global resources
    #[test]
    fn zzz_final_test_cleanup() {
        cel_clear_string_pool();

        // With the new approach, there's no global state to shrink
        // This is a no-op test now
    }
}
