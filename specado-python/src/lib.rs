//! Python bindings for Specado
//!
//! This crate provides Python bindings for the Specado core library using PyO3 v0.25.1.

use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError, PyTypeError};
use pyo3::types::{PyDict, PyList};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specado_core::protocol::types::{ChatRequest as CoreChatRequest, Message as CoreMessage};
use specado_core::providers::{
    OpenAIProvider, AnthropicProvider, Provider, RoutingBuilder, RoutingStrategy,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Structured error types for better error reporting
#[derive(Debug)]
enum SpecadoError {
    ProviderError(String),
    ConfigurationError(String),
    RuntimeError(String),
    MessageFormatError(String),
}

impl From<SpecadoError> for PyErr {
    fn from(err: SpecadoError) -> PyErr {
        match err {
            SpecadoError::ProviderError(msg) => PyRuntimeError::new_err(format!("Provider error: {}", msg)),
            SpecadoError::ConfigurationError(msg) => PyValueError::new_err(format!("Configuration error: {}", msg)),
            SpecadoError::RuntimeError(msg) => PyRuntimeError::new_err(format!("Runtime error: {}", msg)),
            SpecadoError::MessageFormatError(msg) => PyTypeError::new_err(format!("Message format error: {}", msg)),
        }
    }
}

/// Python-compatible message structure
#[pyclass(module = "specado")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    #[pyo3(get, set)]
    pub role: String,
    #[pyo3(get, set)]
    pub content: String,
}

#[pymethods]
impl Message {
    #[new]
    fn new(role: String, content: String) -> Self {
        Self { role, content }
    }
    
    fn __repr__(&self) -> String {
        format!("Message(role='{}', content='{}')", self.role, self.content)
    }
    
    fn __str__(&self) -> String {
        format!("{}: {}", self.role, self.content)
    }
}

/// Response extensions containing routing metadata
#[pyclass(module = "specado")]
#[derive(Clone, Debug)]
pub struct Extensions {
    #[pyo3(get)]
    pub provider_used: String,
    #[pyo3(get)]
    pub fallback_triggered: bool,
    #[pyo3(get)]
    pub attempts: usize,
    metadata_internal: HashMap<String, Value>,
}

#[pymethods]
impl Extensions {
    #[getter]
    fn metadata(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (key, value) in &self.metadata_internal {
            match value {
                Value::String(s) => dict.set_item(key, s.clone())?,
                Value::Number(n) => {
                    if let Some(i) = n.as_u64() {
                        dict.set_item(key, i)?;
                    } else if let Some(f) = n.as_f64() {
                        dict.set_item(key, f)?;
                    } else {
                        dict.set_item(key, n.to_string())?;
                    }
                },
                Value::Bool(b) => dict.set_item(key, *b)?,
                Value::Array(arr) => {
                    let py_list = PyList::empty(py);
                    for item in arr {
                        py_list.append(item.to_string())?;
                    }
                    dict.set_item(key, py_list)?;
                },
                Value::Object(_) => dict.set_item(key, value.to_string())?,
                Value::Null => dict.set_item(key, py.None())?,
            }
        }
        Ok(dict.unbind())
    }
    
    fn __repr__(&self) -> String {
        format!(
            "Extensions(provider_used='{}', fallback_triggered={}, attempts={})",
            self.provider_used, self.fallback_triggered, self.attempts
        )
    }
}

/// Response choice containing the message
#[pyclass(module = "specado")]
#[derive(Debug)]
pub struct Choice {
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub message: Py<Message>,
    #[pyo3(get)]
    pub finish_reason: Option<String>,
}

#[pymethods]
impl Choice {
    fn __repr__(&self) -> String {
        format!(
            "Choice(index={}, finish_reason={:?})",
            self.index, self.finish_reason
        )
    }
}

/// Chat completion response
#[pyclass(module = "specado")]
#[derive(Debug)]
pub struct ChatCompletionResponse {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub object: String,
    #[pyo3(get)]
    pub created: u64,
    #[pyo3(get)]
    pub model: String,
    #[pyo3(get)]
    pub choices: Py<PyList>,
    #[pyo3(get)]
    pub extensions: Py<Extensions>,
}

#[pymethods]
impl ChatCompletionResponse {
    fn __repr__(&self) -> String {
        format!(
            "ChatCompletionResponse(id='{}', model='{}', provider_used='{}')",
            self.id,
            self.model,
            Python::with_gil(|py| {
                self.extensions
                    .bind(py)
                    .getattr("provider_used")
                    .unwrap()
                    .extract::<String>()
                    .unwrap_or_else(|_| "unknown".to_string())
            })
        )
    }
    
    fn __str__(&self) -> String {
        format!("ChatCompletion[{}] - Model: {}", self.id, self.model)
    }
}

/// Shared router state
type SharedRouter = Arc<Mutex<Box<dyn RoutingStrategy>>>;

/// Chat completions API interface
#[pyclass(module = "specado")]
struct ChatCompletions {
    router: SharedRouter,
}

#[pymethods]
impl ChatCompletions {
    /// Create a chat completion (sync wrapper for async operation)
    #[pyo3(signature = (model, messages, temperature=None, max_tokens=None))]
    fn create(
        &self,
        py: Python,
        model: String,
        messages: Vec<Py<Message>>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> PyResult<ChatCompletionResponse> {
        let router = self.router.clone();
        
        // Convert Python messages to core messages with proper error handling
        let core_messages: PyResult<Vec<CoreMessage>> = messages
            .iter()
            .map(|msg| {
                let msg_ref = msg.bind(py);
                let role = msg_ref.getattr("role")?.extract::<String>()?;
                let content = msg_ref.getattr("content")?.extract::<String>()?;
                
                Ok(match role.as_str() {
                    "system" => CoreMessage::system(&content),
                    "user" => CoreMessage::user(&content),
                    "assistant" => CoreMessage::assistant(&content),
                    _ => CoreMessage::user(&content), // Default to user
                })
            })
            .collect();
        let core_messages = core_messages?;
        
        // Create chat request
        let mut request = CoreChatRequest::new(&model, core_messages);
        if let Some(temp) = temperature {
            request.temperature = Some(temp);
        }
        if let Some(max_tok) = max_tokens {
            request.max_tokens = Some(max_tok as usize);
        }
        
        // Execute async operation in blocking manner
        // In production, we'd use pyo3-asyncio for proper async support
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| SpecadoError::RuntimeError(format!("Failed to create runtime: {}", e)))?;
        
        let request_model = request.model.clone();
        let result = runtime.block_on(async move {
            let router_guard = router.lock().await;
            router_guard.route(request).await
        }).map_err(|e| SpecadoError::ProviderError(format!("Routing failed: {}", e)))?;
        
        // Create extensions
        let extensions = Py::new(py, Extensions {
            provider_used: result.provider_used.clone(),
            fallback_triggered: result.used_fallback,
            attempts: result.attempts,
            metadata_internal: result.metadata.clone(),
        })?;
        
        // Extract actual response from routing result
        let response = result.response
            .ok_or_else(|| SpecadoError::RuntimeError("No response received from provider".to_string()))?;
        
        // Convert the first choice to Python format
        let first_choice = response.choices.first()
            .ok_or_else(|| SpecadoError::RuntimeError("No choices in response".to_string()))?;
        
        // Extract message content
        let message_content = match &first_choice.message.content {
            specado_core::protocol::types::MessageContent::Text(text) => text.clone(),
            specado_core::protocol::types::MessageContent::Parts(_) => {
                // For multimodal, just use a placeholder for now
                "[Multimodal content]".to_string()
            }
        };
        
        let message = Py::new(py, Message {
            role: "assistant".to_string(),
            content: message_content,
        })?;
        
        // Create choice
        let choice = Choice {
            index: first_choice.index,
            message,
            finish_reason: first_choice.finish_reason.clone(),
        };
        
        // Create choices list 
        let choice_py = Py::new(py, choice)?;
        let choices_list = PyList::new(py, vec![choice_py])?;
        
        // Create response using actual response data
        let response_obj = ChatCompletionResponse {
            id: response.id.clone(),
            object: response.object.clone(),
            created: response.created as u64,
            model: response.model.clone(),
            choices: choices_list.into(),
            extensions,
        };
        
        Ok(response_obj)
    }
    
    /// Create a chat completion (async version for Python asyncio)
    /// This returns a coroutine that can be awaited in Python
    #[pyo3(signature = (model, messages, temperature=None, max_tokens=None))]
    fn create_async<'py>(
        &self,
        py: Python<'py>,
        model: String,
        messages: Vec<Py<Message>>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> PyResult<Py<PyAny>> {
        // For MVP, wrap the sync version
        // In production, we'd use pyo3-asyncio for proper async support
        let result = self.create(py, model, messages, temperature, max_tokens)?;
        Ok(Py::new(py, result)?.into_any())
    }
}

/// Chat API namespace
#[pyclass(module = "specado")]
struct Chat {
    #[pyo3(get)]
    completions: Py<ChatCompletions>,
}

#[pymethods]
impl Chat {
    fn __repr__(&self) -> String {
        "Chat(completions=<ChatCompletions>)".to_string()
    }
}

/// Main Specado client
#[pyclass(module = "specado")]
pub struct Client {
    #[pyo3(get)]
    chat: Py<Chat>,
    config: HashMap<String, Value>,
}

#[pymethods]
impl Client {
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(py: Python, config: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        // Parse configuration with proper type handling
        let config_map = if let Some(cfg) = config {
            cfg.iter()
                .map(|(k, v)| {
                    let key = k.extract::<String>()?;
                    let value = if let Ok(s) = v.extract::<String>() {
                        Value::String(s)
                    } else if let Ok(b) = v.extract::<bool>() {
                        Value::Bool(b)
                    } else if let Ok(i) = v.extract::<i64>() {
                        Value::Number(serde_json::Number::from(i))
                    } else if let Ok(f) = v.extract::<f64>() {
                        Value::Number(serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)))
                    } else {
                        Value::String(v.to_string())
                    };
                    Ok((key, value))
                })
                .collect::<PyResult<HashMap<_, _>>>()?
        } else {
            HashMap::new()
        };
        
        // Create router based on configuration
        let primary_provider = config_map
            .get("primary_provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openai");
        
        let fallback_provider = config_map
            .get("fallback_provider")
            .and_then(|v| v.as_str())
            .unwrap_or("anthropic");
        
        // Build router
        let router = create_router(primary_provider, fallback_provider)?;
        let router_arc = Arc::new(Mutex::new(router));
        
        // Create completions API
        let completions = Py::new(py, ChatCompletions {
            router: router_arc.clone(),
        })?;
        
        // Create chat namespace
        let chat = Py::new(py, Chat { completions })?;
        
        Ok(Self {
            chat,
            config: config_map,
        })
    }
    
    fn __repr__(&self) -> String {
        format!("Client(providers={})", 
            self.config.len()
        )
    }
    
    fn __str__(&self) -> String {
        format!("Specado Client with {} configured providers", self.config.len())
    }
    
    /// Get configuration value
    fn get_config(&self, key: &str) -> Option<String> {
        self.config.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }
    
    /// Get all configuration keys
    fn config_keys(&self) -> Vec<String> {
        self.config.keys().cloned().collect()
    }
}

/// Helper function to create router
fn create_router(primary: &str, fallback: &str) -> PyResult<Box<dyn RoutingStrategy>> {
    let primary_provider: Box<dyn Provider> = match primary {
        "openai" => Box::new(OpenAIProvider::new()),
        "anthropic" => Box::new(AnthropicProvider::new()),
        _ => return Err(PyRuntimeError::new_err(format!("Unknown provider: {}", primary))),
    };
    
    let fallback_provider: Box<dyn Provider> = match fallback {
        "openai" => Box::new(OpenAIProvider::new()),
        "anthropic" => Box::new(AnthropicProvider::new()),
        _ => return Err(PyRuntimeError::new_err(format!("Unknown provider: {}", fallback))),
    };
    
    RoutingBuilder::new()
        .primary(primary_provider)
        .fallback(fallback_provider)
        .build()
        .map(|router| Box::new(router) as Box<dyn RoutingStrategy>)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to build router: {}", e)))
}

/// Generate a simple UUID-like string
fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

/// Returns the version of the Specado library.
#[pyfunction]
fn version() -> PyResult<&'static str> {
    Ok(specado_core::version())
}

mod capabilities;

/// Main module initialization for Python bindings.
#[pymodule]
fn specado(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add version info
    m.add("__version__", specado_core::version())?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    
    // Add classes
    m.add_class::<Client>()?;
    m.add_class::<Message>()?;
    m.add_class::<ChatCompletionResponse>()?;
    m.add_class::<Extensions>()?;
    m.add_class::<Choice>()?;
    m.add_class::<Chat>()?;
    m.add_class::<ChatCompletions>()?;
    
    // Add capability functions
    capabilities::register_capabilities(m)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        Python::with_gil(|py| {
            let msg = Message::new("user".to_string(), "Hello".to_string());
            assert_eq!(msg.role, "user");
            assert_eq!(msg.content, "Hello");
        });
    }
    
    #[test] 
    fn test_uuid_generation() {
        let id1 = uuid();
        let id2 = uuid();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
    }
    
    #[test]
    fn test_message_repr() {
        let msg = Message::new("assistant".to_string(), "Hi there!".to_string());
        let repr = msg.__repr__();
        assert!(repr.contains("assistant"));
        assert!(repr.contains("Hi there!"));
    }
    
    #[test]
    fn test_extensions_repr() {
        let ext = Extensions {
            provider_used: "openai".to_string(),
            fallback_triggered: false,
            attempts: 1,
            metadata_internal: HashMap::new(),
        };
        let repr = ext.__repr__();
        assert!(repr.contains("openai"));
        assert!(repr.contains("false"));
        assert!(repr.contains("1"));
    }
}