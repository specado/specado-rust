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
#[cfg(not(test))]
#[napi]
pub async fn hello_world_async() -> Result<String> {
    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    Ok(specado_core::hello_world())
}

/// Test version of async function
#[cfg(test)]
pub async fn hello_world_async() -> Result<String> {
    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
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

    #[tokio::test]
    async fn test_hello_world_async() {
        let result = hello_world_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello from Specado Core!");
    }
}

