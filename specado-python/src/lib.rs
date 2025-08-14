//! Python bindings for Specado
//!
//! This crate provides Python bindings for the Specado core library using PyO3.

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// Returns a hello world message from the Specado core library.
#[pyfunction]
fn hello_world() -> PyResult<String> {
    Ok(specado_core::hello_world())
}

/// Returns the version of the Specado library.
#[pyfunction]
fn version() -> PyResult<&'static str> {
    Ok(specado_core::version())
}

/// Main module initialization for Python bindings.
#[pymodule]
fn specado(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_world, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("__version__", specado_core::version())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "extension-module")]
    #[ignore] // PyO3 tests need to be run with maturin, not cargo test
    fn test_hello_world_binding() {
        Python::with_gil(|_py| {
            let result = hello_world();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "Hello from Specado Core!");
        });
    }

    #[test]
    #[cfg(feature = "extension-module")]
    #[ignore] // PyO3 tests need to be run with maturin, not cargo test
    fn test_version_binding() {
        Python::with_gil(|_py| {
            let result = version();
            assert!(result.is_ok());
            assert!(!result.unwrap().is_empty());
        });
    }
}

