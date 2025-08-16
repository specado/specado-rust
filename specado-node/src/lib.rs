//! Node.js bindings for Specado
//!
//! This crate provides Node.js bindings for the Specado core library using NAPI-RS.
//! Implementation follows the Python bindings pattern with native async support.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
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

impl From<SpecadoError> for napi::Error {
    fn from(err: SpecadoError) -> napi::Error {
        match err {
            SpecadoError::ProviderError(msg) => {
                napi::Error::new(napi::Status::GenericFailure, format!("Provider error: {}", msg))
            }
            SpecadoError::ConfigurationError(msg) => {
                napi::Error::new(napi::Status::InvalidArg, format!("Configuration error: {}", msg))
            }
            SpecadoError::RuntimeError(msg) => {
                napi::Error::new(napi::Status::GenericFailure, format!("Runtime error: {}", msg))
            }
            SpecadoError::MessageFormatError(msg) => {
                napi::Error::new(napi::Status::InvalidArg, format!("Message format error: {}", msg))
            }
        }
    }
}

/// JavaScript-compatible message structure
#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Create a new message
#[napi(js_name = "createMessage")]
pub fn create_message(role: String, content: String) -> Message {
    Message { role, content }
}

/// Response extensions containing routing metadata
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Extensions {
    pub provider_used: String,
    pub fallback_triggered: bool,
    pub attempts: i32,
    pub metadata: HashMap<String, String>,
}

/// Response choice containing the message
#[napi(object)]
#[derive(Debug)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

/// Chat completion response
#[napi(object)]
#[derive(Debug)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub extensions: Extensions,
}

/// Shared router state
type SharedRouter = Arc<Mutex<Box<dyn RoutingStrategy>>>;

/// Chat completions API interface
#[napi]
pub struct ChatCompletions {
    router: SharedRouter,
}

#[napi]
impl ChatCompletions {
    /// Create a chat completion (async native to Node.js)
    #[napi]
    pub async fn create(
        &self,
        model: String,
        messages: Vec<Message>,
        temperature: Option<f64>,
        max_tokens: Option<i32>,
    ) -> Result<ChatCompletionResponse> {
        let router = self.router.clone();
        
        // Convert JS messages to core messages
        let core_messages: Result<Vec<CoreMessage>> = messages
            .iter()
            .map(|msg| {
                Ok(match msg.role.as_str() {
                    "system" => CoreMessage::system(&msg.content),
                    "user" => CoreMessage::user(&msg.content),
                    "assistant" => CoreMessage::assistant(&msg.content),
                    _ => CoreMessage::user(&msg.content), // Default to user
                })
            })
            .collect();
        let core_messages = core_messages?;
        
        // Create chat request
        let mut request = CoreChatRequest::new(&model, core_messages);
        if let Some(temp) = temperature {
            request.temperature = Some(temp as f32);
        }
        if let Some(max_tok) = max_tokens {
            request.max_tokens = Some(max_tok as usize);
        }
        
        // Execute routing operation
        let request_model = request.model.clone();
        let result = {
            let router_guard = router.lock().await;
            router_guard.route(request).await
        }.map_err(|e| SpecadoError::ProviderError(format!("Routing failed: {}", e)))?;
        
        // Create extensions with proper type conversion
        let mut metadata = HashMap::new();
        for (key, value) in result.metadata.iter() {
            metadata.insert(key.clone(), value.to_string());
        }
        
        let extensions = Extensions {
            provider_used: result.provider_used.clone(),
            fallback_triggered: result.used_fallback,
            attempts: result.attempts as i32,
            metadata,
        };
        
        // Extract actual response from routing result
        let response = result.response
            .ok_or_else(|| SpecadoError::RuntimeError("No response received from provider".to_string()))?;
        
        // Convert the first choice to Node.js format
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
        
        let message = Message {
            role: "assistant".to_string(),
            content: message_content,
        };
        
        // Create choice
        let choice = Choice {
            index: first_choice.index as i32,
            message,
            finish_reason: first_choice.finish_reason.clone(),
        };
        
        // Create response using actual response data
        let response_obj = ChatCompletionResponse {
            id: response.id.clone(),
            object: response.object.clone(),
            created: response.created,
            model: response.model.clone(),
            choices: vec![choice],
            extensions,
        };
        
        Ok(response_obj)
    }
}

/// Chat API namespace
#[napi]
pub struct Chat {
    completions: ChatCompletions,
}

#[napi]
impl Chat {
    /// Access to completions API
    #[napi]
    pub fn get_completions(&self) -> ChatCompletions {
        ChatCompletions {
            router: self.completions.router.clone(),
        }
    }
}

/// Main Specado client
#[napi]
pub struct Client {
    chat: Chat,
    config: HashMap<String, String>,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new(config: Option<Object>) -> Result<Self> {
        // Parse configuration
        let config_map = if let Some(cfg) = config {
            // Extract configuration from JS object
            let primary_provider = cfg.get("primary_provider")
                .ok()
                .flatten()
                .unwrap_or_else(|| "openai".to_string());
            let fallback_provider = cfg.get("fallback_provider")
                .ok()
                .flatten()
                .unwrap_or_else(|| "anthropic".to_string());
            
            let mut map = HashMap::new();
            map.insert("primary_provider".to_string(), primary_provider);
            map.insert("fallback_provider".to_string(), fallback_provider);
            map
        } else {
            let mut map = HashMap::new();
            map.insert("primary_provider".to_string(), "openai".to_string());
            map.insert("fallback_provider".to_string(), "anthropic".to_string());
            map
        };
        
        // Create router based on configuration
        let primary_provider = config_map
            .get("primary_provider")
            .map(|s| s.as_str())
            .unwrap_or("openai");
        let fallback_provider = config_map
            .get("fallback_provider")
            .map(|s| s.as_str())
            .unwrap_or("anthropic");
        
        // Build router
        let router = create_router(primary_provider, fallback_provider)?;
        let router_arc = Arc::new(Mutex::new(router));
        
        // Create completions API
        let completions = ChatCompletions {
            router: router_arc.clone(),
        };
        
        // Create chat namespace
        let chat = Chat { completions };
        
        Ok(Self {
            chat,
            config: config_map,
        })
    }
    
    /// Access to chat namespace
    #[napi]
    pub fn get_chat(&self) -> Chat {
        Chat {
            completions: ChatCompletions {
                router: self.chat.completions.router.clone(),
            }
        }
    }
    
    /// Get configuration value
    #[napi]
    pub fn get_config(&self, key: String) -> Option<String> {
        self.config.get(&key).cloned()
    }
    
    /// Get all configuration keys
    #[napi]
    pub fn config_keys(&self) -> Vec<String> {
        self.config.keys().cloned().collect()
    }
}

/// Helper function to create router
fn create_router(primary: &str, fallback: &str) -> Result<Box<dyn RoutingStrategy>> {
    let primary_provider: Box<dyn Provider> = match primary {
        "openai" => Box::new(OpenAIProvider::new()),
        "anthropic" => Box::new(AnthropicProvider::new()),
        _ => return Err(SpecadoError::ConfigurationError(format!("Unknown provider: {}", primary)).into()),
    };
    
    let fallback_provider: Box<dyn Provider> = match fallback {
        "openai" => Box::new(OpenAIProvider::new()),
        "anthropic" => Box::new(AnthropicProvider::new()),
        _ => return Err(SpecadoError::ConfigurationError(format!("Unknown provider: {}", fallback)).into()),
    };
    
    RoutingBuilder::new()
        .primary(primary_provider)
        .fallback(fallback_provider)
        .build()
        .map(|router| Box::new(router) as Box<dyn RoutingStrategy>)
        .map_err(|e| SpecadoError::RuntimeError(format!("Failed to build router: {}", e)).into())
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
#[napi]
pub fn version() -> Result<String> {
    Ok(specado_core::version().to_string())
}

/// Returns a hello world message from the Specado core library.
#[napi(js_name = "helloWorld")]
pub fn hello_world() -> Result<String> {
    Ok(specado_core::hello_world())
}

/// Get OpenAI provider manifest as JSON object
#[napi(js_name = "getOpenaiManifest")]
pub fn get_openai_manifest() -> Result<String> {
    let manifest = specado_core::capabilities::ProviderManifest::openai();
    serde_json::to_string(&manifest)
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Failed to serialize manifest: {}", e)))
}

/// Get Anthropic provider manifest as JSON object
#[napi(js_name = "getAnthropicManifest")]
pub fn get_anthropic_manifest() -> Result<String> {
    let manifest = specado_core::capabilities::ProviderManifest::anthropic();
    serde_json::to_string(&manifest)
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Failed to serialize manifest: {}", e)))
}

/// Compare two capabilities and return lossiness report
#[napi(js_name = "compareCapabilities")]
pub fn compare_capabilities(source_json: String, target_json: String) -> Result<String> {
    // Parse source capability
    let source: specado_core::capabilities::Capability = serde_json::from_str(&source_json)
        .map_err(|e| napi::Error::new(napi::Status::InvalidArg, format!("Invalid source capability: {}", e)))?;
    
    // Parse target capability
    let target: specado_core::capabilities::Capability = serde_json::from_str(&target_json)
        .map_err(|e| napi::Error::new(napi::Status::InvalidArg, format!("Invalid target capability: {}", e)))?;
    
    // Compare capabilities
    let comparison = source.compare(&target);
    
    // Serialize result
    serde_json::to_string(&comparison)
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Failed to serialize comparison: {}", e)))
}

/// Get model capabilities for a specific provider and model
#[napi(js_name = "getModelCapabilities")]
pub fn get_model_capabilities(provider: String, model_id: String) -> Result<Option<String>> {
    let manifest = match provider.to_lowercase().as_str() {
        "openai" => specado_core::capabilities::ProviderManifest::openai(),
        "anthropic" => specado_core::capabilities::ProviderManifest::anthropic(),
        _ => return Ok(None),
    };
    
    match manifest.get_model_capabilities(&model_id) {
        Some(capabilities) => {
            let json = serde_json::to_string(capabilities)
                .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Failed to serialize capabilities: {}", e)))?;
            Ok(Some(json))
        }
        None => Ok(None),
    }
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
    fn test_message_creation() {
        let msg = create_message("user".to_string(), "Hello".to_string());
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }
    
    #[test]
    fn test_uuid_generation() {
        let id1 = uuid();
        let id2 = uuid();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
    }
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = Client::new(None);
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.get_config("primary_provider".to_string()), Some("openai".to_string()));
    }
}
