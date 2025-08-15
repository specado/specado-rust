//! Specado Core Library
//!
//! This crate provides the core functionality for spec-driven LLM integration.

pub mod config;
pub mod protocol;
pub mod providers;

use std::ffi::CString;
use std::os::raw::c_char;

/// Returns a simple hello world message.
/// This function is used to test FFI bindings.
pub fn hello_world() -> String {
    "Hello from Specado Core!".to_string()
}

/// FFI-safe version of hello_world that returns a C string.
/// The caller is responsible for freeing the returned string using `free_string`.
#[no_mangle]
pub extern "C" fn specado_hello_world() -> *mut c_char {
    let message = hello_world();
    match CString::new(message) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a string that was allocated by Rust.
/// This must be called on strings returned by `specado_hello_world`.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The pointer must be valid and must have been allocated by `specado_hello_world`.
#[no_mangle]
pub unsafe extern "C" fn specado_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = CString::from_raw(s);
}

/// Returns the version of the Specado Core library.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// FFI-safe version of the version function.
#[no_mangle]
pub extern "C" fn specado_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello from Specado Core!");
    }

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_ffi_hello_world() {
        let ptr = specado_hello_world();
        assert!(!ptr.is_null());

        unsafe {
            let c_str = CStr::from_ptr(ptr);
            let rust_str = c_str.to_str().unwrap();
            assert_eq!(rust_str, "Hello from Specado Core!");
        }

        unsafe {
            specado_free_string(ptr);
        }
    }
}
