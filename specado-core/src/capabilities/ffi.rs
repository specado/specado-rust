//! FFI bindings for capability taxonomy

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde_json;

use crate::capabilities::{ProviderManifest, Capability};

/// Get OpenAI provider manifest as JSON string
/// The caller must free the returned string using specado_free_string
#[no_mangle]
pub extern "C" fn specado_get_openai_manifest() -> *mut c_char {
    let manifest = ProviderManifest::openai();
    match serde_json::to_string(&manifest) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Get Anthropic provider manifest as JSON string
/// The caller must free the returned string using specado_free_string
#[no_mangle]
pub extern "C" fn specado_get_anthropic_manifest() -> *mut c_char {
    let manifest = ProviderManifest::anthropic();
    match serde_json::to_string(&manifest) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Compare two capabilities and return lossiness report as JSON
/// Both capability JSONs must be valid JSON strings
/// The caller must free the returned string using specado_free_string
#[no_mangle]
pub unsafe extern "C" fn specado_compare_capabilities(
    source_json: *const c_char,
    target_json: *const c_char,
) -> *mut c_char {
    if source_json.is_null() || target_json.is_null() {
        return std::ptr::null_mut();
    }
    
    let source_str = match CStr::from_ptr(source_json).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let target_str = match CStr::from_ptr(target_json).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let source: Capability = match serde_json::from_str(source_str) {
        Ok(cap) => cap,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let target: Capability = match serde_json::from_str(target_str) {
        Ok(cap) => cap,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let comparison = source.compare(&target);
    
    match serde_json::to_string(&comparison) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Get a specific model's capabilities from a provider manifest
/// Returns null if model not found
/// The caller must free the returned string using specado_free_string
#[no_mangle]
pub unsafe extern "C" fn specado_get_model_capabilities(
    provider: *const c_char,
    model_id: *const c_char,
) -> *mut c_char {
    if provider.is_null() || model_id.is_null() {
        return std::ptr::null_mut();
    }
    
    let provider_str = match CStr::from_ptr(provider).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let model_str = match CStr::from_ptr(model_id).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let manifest = match provider_str.to_lowercase().as_str() {
        "openai" => ProviderManifest::openai(),
        "anthropic" => ProviderManifest::anthropic(),
        _ => return std::ptr::null_mut(),
    };
    
    match manifest.get_model_capabilities(model_str) {
        Some(capabilities) => {
            match serde_json::to_string(capabilities) {
                Ok(json) => match CString::new(json) {
                    Ok(c_str) => c_str.into_raw(),
                    Err(_) => std::ptr::null_mut(),
                },
                Err(_) => std::ptr::null_mut(),
            }
        }
        None => std::ptr::null_mut(),
    }
}