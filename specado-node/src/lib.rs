//! Node.js bindings for Specado
//!
//! This crate provides Node.js bindings for the Specado core library using N-API.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Returns a hello world message from the Specado core library.
#[napi]
pub fn hello_world() -> Result<String> {
    Ok(specado_core::hello_world())
}

/// Returns the version of the Specado library.
#[napi]
pub fn version() -> Result<String> {
    Ok(specado_core::version().to_string())
}

/// A more complex example showing async support
#[napi]
pub async fn hello_world_async() -> Result<String> {
    // For Sprint 0, just return the result without actual async work
    Ok(specado_core::hello_world())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let result = hello_world();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello from Specado Core!");
    }

    #[test]
    fn test_version() {
        let result = version();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_hello_world_async() {
        // For Sprint 0, we'll test the async function synchronously
        // since it doesn't actually do async work yet
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(hello_world_async());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello from Specado Core!");
    }
}
