//! Python bindings for capability taxonomy

use pyo3::prelude::*;
use pyo3::types::PyDict;
use specado_core::capabilities::{ProviderManifest, Capability};

/// Get OpenAI provider manifest
#[pyfunction]
fn get_openai_manifest(py: Python) -> PyResult<PyObject> {
    let manifest = ProviderManifest::openai();
    let json_str = serde_json::to_string(&manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    // Parse JSON string into Python dict
    let json_module = py.import("json")?;
    let loads = json_module.getattr("loads")?;
    let result = loads.call1((json_str,))?;
    Ok(result.into())
}

/// Get Anthropic provider manifest
#[pyfunction]
fn get_anthropic_manifest(py: Python) -> PyResult<PyObject> {
    let manifest = ProviderManifest::anthropic();
    let json_str = serde_json::to_string(&manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    // Parse JSON string into Python dict
    let json_module = py.import("json")?;
    let loads = json_module.getattr("loads")?;
    let result = loads.call1((json_str,))?;
    Ok(result.into())
}

/// Compare two capabilities and return lossiness report
#[pyfunction]
fn compare_capabilities(py: Python, source: Bound<'_, PyDict>, target: Bound<'_, PyDict>) -> PyResult<PyObject> {
    // Convert Python dicts to JSON strings
    let json_module = py.import("json")?;
    let dumps = json_module.getattr("dumps")?;
    
    let source_json = dumps.call1((&source,))?;
    let source_str = source_json.extract::<String>()?;
    
    let target_json = dumps.call1((&target,))?;
    let target_str = target_json.extract::<String>()?;
    
    // Parse into Rust capabilities
    let source_cap: Capability = serde_json::from_str(&source_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid source capability: {}", e)
        ))?;
    
    let target_cap: Capability = serde_json::from_str(&target_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid target capability: {}", e)
        ))?;
    
    // Compare capabilities
    let comparison = source_cap.compare(&target_cap);
    
    // Convert back to Python dict
    let comparison_json = serde_json::to_string(&comparison)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    let loads = json_module.getattr("loads")?;
    let result = loads.call1((comparison_json,))?;
    Ok(result.into())
}

/// Get model capabilities for a specific provider and model
#[pyfunction]
fn get_model_capabilities(py: Python, provider: &str, model_id: &str) -> PyResult<Option<PyObject>> {
    let manifest = match provider.to_lowercase().as_str() {
        "openai" => ProviderManifest::openai(),
        "anthropic" => ProviderManifest::anthropic(),
        _ => return Ok(None),
    };
    
    match manifest.get_model_capabilities(model_id) {
        Some(capabilities) => {
            let json_str = serde_json::to_string(capabilities)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            let json_module = py.import("json")?;
            let loads = json_module.getattr("loads")?;
            let result = loads.call1((json_str,))?;
            Ok(Some(result.into()))
        }
        None => Ok(None),
    }
}

/// Register capability functions with the Python module
pub fn register_capabilities(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_openai_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(get_anthropic_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(compare_capabilities, m)?)?;
    m.add_function(wrap_pyfunction!(get_model_capabilities, m)?)?;
    Ok(())
}