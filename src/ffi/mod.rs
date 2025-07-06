pub mod context;
pub mod program;

use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};

// Global string pool for memory management
lazy_static::lazy_static! {
    static ref STRING_POOL: Arc<Mutex<HashMap<usize, Arc<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// Re-export the main types
pub use context::*;
pub use program::*;

// Common value types for CEL
#[repr(C)]
#[derive(Debug, Clone)]
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
    Timestamp,
    Duration,
}

#[repr(C)]
pub struct CelValue {
    pub value_type: CelValueType,
    pub data: CelValueData,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CelStringValue {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CelBytesValue {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CelListValue {
    pub ptr: *const CelValue,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CelMapValue {
    pub ptr: *const CelMapEntry,
    pub len: usize,
}

#[repr(C)]
pub struct CelMapEntry {
    pub key: CelValue,
    pub value: CelValue,
}

#[repr(C)]
pub union CelValueData {
    pub bool_val: bool,
    pub int_val: i64,
    pub uint_val: u64,
    pub double_val: f64,
    pub string_val: ManuallyDrop<CelStringValue>,
    pub bytes_val: ManuallyDrop<CelBytesValue>,
}

// Helper function to convert C string to Rust string
/// # Safety
/// The caller must ensure that `ptr` is a valid null-terminated C string pointer
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Result<String, std::str::Utf8Error> {
    let c_str = CStr::from_ptr(ptr);
    Ok(c_str.to_str()?.to_string())
}

// String memory management functions
pub fn store_string_in_pool(s: &str) -> *const u8 {
    let arc_string = Arc::new(s.to_string());
    let ptr = arc_string.as_ptr();
    let addr = ptr as usize;

    if let Ok(mut pool) = STRING_POOL.lock() {
        pool.insert(addr, arc_string);
    }

    ptr
}

pub fn release_string_from_pool(ptr: *const u8) {
    let addr = ptr as usize;
    if let Ok(mut pool) = STRING_POOL.lock() {
        pool.remove(&addr);
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

/// Clear all strings from the pool (useful for cleanup)
#[no_mangle]
pub extern "C" fn cel_string_pool_clear() {
    if let Ok(mut pool) = STRING_POOL.lock() {
        pool.clear();
    }
}
