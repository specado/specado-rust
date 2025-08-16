# Specado Rust Core API Documentation

This document provides comprehensive API documentation for the Specado Core Rust library (`specado-core`), which implements the foundational layer for spec-driven LLM integration with FFI-safe interfaces for Python and Node.js bindings.

## Table of Contents

1. [FFI Interface](#ffi-interface)
2. [Core Module APIs](#core-module-apis)
3. [Capability Taxonomy](#capability-taxonomy)
4. [Configuration System](#configuration-system)
5. [HTTP Client](#http-client)
6. [Protocol Types](#protocol-types)
7. [Provider System](#provider-system)
8. [Developer Guide](#developer-guide)

---

## FFI Interface

The FFI interface in `src/lib.rs` provides C-compatible exports for external language bindings. All FFI functions are marked with `#[no_mangle]` and use `extern "C"` calling convention.

### Memory Management Pattern

**Critical**: All string-returning FFI functions allocate memory that **must** be freed by the caller using `specado_free_string`.

```rust
// Safe FFI string allocation pattern
match CString::new(data) {
    Ok(c_str) => c_str.into_raw(),
    Err(_) => std::ptr::null_mut(),
}
```

### Core FFI Functions

#### `specado_hello_world() -> *mut c_char`
**Location**: `src/lib.rs:23`

Returns a test message for FFI binding verification.

```c
// C usage
char* message = specado_hello_world();
printf("%s\n", message);
specado_free_string(message);  // Required cleanup
```

**Returns**: Pointer to C string "Hello from Specado Core!" or `null` on error.

#### `specado_free_string(s: *mut c_char)`
**Location**: `src/lib.rs:39`

**Safety**: This function is `unsafe` and must only be called with pointers returned from Specado FFI functions.

```c
// C usage - always pair with string-returning functions
char* result = specado_get_openai_manifest();
if (result != NULL) {
    // Use result...
    specado_free_string(result);
}
```

#### `specado_version() -> *const c_char`
**Location**: `src/lib.rs:53`

Returns the library version as a static null-terminated string.

```c
// C usage - no cleanup required (static string)
const char* version = specado_version();
printf("Version: %s\n", version);
```

#### `specado_get_openai_manifest() -> *mut c_char`
**Location**: `src/capabilities/ffi.rs:12`

Returns OpenAI provider capabilities as JSON string.

```c
// C usage
char* manifest = specado_get_openai_manifest();
if (manifest != NULL) {
    // Parse JSON manifest...
    specado_free_string(manifest);
}
```

#### `specado_get_anthropic_manifest() -> *mut c_char`
**Location**: `src/capabilities/ffi.rs:26`

Returns Anthropic provider capabilities as JSON string.

#### `specado_compare_capabilities(source_json: *const c_char, target_json: *const c_char) -> *mut c_char`
**Location**: `src/capabilities/ffi.rs:41`

**Safety**: Both input pointers must be valid null-terminated JSON strings.

Compares two capability manifests and returns lossiness analysis as JSON.

```c
// C usage
const char* source = "{\"version\":\"0.1.0\",\"features\":{...}}";
const char* target = "{\"version\":\"0.1.0\",\"features\":{...}}";
char* comparison = specado_compare_capabilities(source, target);
if (comparison != NULL) {
    // Parse comparison result...
    specado_free_string(comparison);
}
```

#### `specado_get_model_capabilities(provider: *const c_char, model_id: *const c_char) -> *mut c_char`
**Location**: `src/capabilities/ffi.rs:84`

Returns capabilities for a specific model from a provider.

```c
// C usage
char* caps = specado_get_model_capabilities("openai", "gpt-4-turbo");
if (caps != NULL) {
    // Use model capabilities...
    specado_free_string(caps);
}
```

### C Header Generation

For external consumers, generate C headers with:

```bash
# Using cbindgen (recommended)
cbindgen --config cbindgen.toml --crate specado-core --output specado.h

# Manual header stub
extern "C" {
    char* specado_hello_world(void);
    void specado_free_string(char* s);
    const char* specado_version(void);
    char* specado_get_openai_manifest(void);
    char* specado_get_anthropic_manifest(void);
    char* specado_compare_capabilities(const char* source_json, const char* target_json);
    char* specado_get_model_capabilities(const char* provider, const char* model_id);
}
```

---

## Core Module APIs

### Library Structure

The core library is organized into five main modules:

```rust
pub mod capabilities;  // Provider capability taxonomy
pub mod config;       // Configuration management  
pub mod http;         // HTTP client and request handling
pub mod protocol;     // Protocol types and message structures
pub mod providers;    // Provider adapters and routing
```

---

## Capability Taxonomy

**Location**: `src/capabilities/mod.rs`

The capability system provides a structured model for describing and comparing LLM provider capabilities, enabling intelligent provider selection and lossiness detection during transformations.

### Core Types

#### `Capability`
**Location**: `src/capabilities/mod.rs:23`

Central structure defining a provider's complete capability set.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    pub version: String,
    pub modalities: ModalitySupport,
    pub features: ModelFeatures,
    pub parameters: ControlParameters,
    pub roles: RoleSupport,
    pub constraints: Constraints,
    pub extensions: Extensions,
}
```

**Usage Example**:
```rust
use specado_core::capabilities::Capability;

// Create default capability
let cap = Capability::default();

// Use builder pattern
let cap = Capability::builder()
    .version("1.0.0")
    .with_features(ModelFeatures {
        function_calling: true,
        json_mode: true,
        ..Default::default()
    })
    .build();

// Check feature support
if cap.supports_feature("function_calling") {
    println!("Function calling supported");
}
```

#### `ModelFeatures`
**Location**: `src/capabilities/mod.rs:47`

Defines boolean feature flags for model capabilities.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelFeatures {
    pub function_calling: bool,
    pub json_mode: bool,
    pub streaming: bool,
    pub logprobs: bool,
    pub multiple_responses: bool,
    pub stop_sequences: bool,
    pub seed_support: bool,
    pub tool_use: bool,
    pub vision: bool,
}
```

#### `ControlParameters`
**Location**: `src/capabilities/mod.rs:78`

Defines parameter support with bounds and defaults.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlParameters {
    pub temperature: ParameterSupport<f32>,
    pub top_p: ParameterSupport<f32>,
    pub top_k: ParameterSupport<i32>,
    pub max_tokens: ParameterSupport<i32>,
    pub frequency_penalty: ParameterSupport<f32>,
    pub presence_penalty: ParameterSupport<f32>,
    pub repetition_penalty: ParameterSupport<f32>,
}
```

#### `ParameterSupport<T>`
**Location**: `src/capabilities/mod.rs:103`

Generic support definition with bounds.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterSupport<T> {
    pub supported: bool,
    pub min: Option<T>,
    pub max: Option<T>,
    pub default: Option<T>,
}
```

**Usage Example**:
```rust
let temp_support = ParameterSupport {
    supported: true,
    min: Some(0.0),
    max: Some(2.0),
    default: Some(0.7),
};
```

### Provider Manifests

**Location**: `src/capabilities/provider_manifest.rs`

Provider manifests define the complete capability set for a specific provider.

```rust
use specado_core::capabilities::ProviderManifest;

// Get built-in provider manifests
let openai = ProviderManifest::openai();
let anthropic = ProviderManifest::anthropic();

// List available models
for model in openai.list_models() {
    println!("Model: {}", model);
}

// Get specific model capabilities
if let Some(caps) = openai.get_model_capabilities("gpt-4-turbo") {
    println!("GPT-4 supports vision: {}", caps.features.vision);
}
```

### Capability Comparison

**Location**: `src/capabilities/comparison.rs`

Compare capabilities between providers to detect lossiness.

```rust
use specado_core::capabilities::{CapabilityComparison, LossinessType};

let gpt4 = openai.get_model_capabilities("gpt-4-turbo").unwrap();
let claude = anthropic.get_model_capabilities("claude-3-opus").unwrap();

let comparison = gpt4.compare(claude);

println!("Is lossy: {}", comparison.lossiness_report.is_lossy);
println!("Severity: {:?}", comparison.lossiness_report.severity);

for detail in &comparison.lossiness_report.details {
    println!("Loss: {}", detail);
}
```

### Extending the Capability System

To add new capability types:

1. **Add to ModelFeatures**:
```rust
// In src/capabilities/mod.rs
pub struct ModelFeatures {
    // existing fields...
    pub new_feature: bool,
}
```

2. **Update supports_feature method**:
```rust
pub fn supports_feature(&self, feature: &str) -> bool {
    match feature {
        // existing cases...
        "new_feature" => self.features.new_feature,
        _ => self.extensions.experimental.contains(feature),
    }
}
```

3. **Update provider manifests**:
```rust
// In provider_manifest.rs
pub fn openai() -> Self {
    // Update feature flags for new capability
}
```

---

## Configuration System

**Location**: `src/config/mod.rs`

The configuration system provides YAML/JSON loading with environment variable interpolation and comprehensive validation.

### Core Types

#### `SpecadoConfig`
**Location**: `src/config/schema.rs:12`

Root configuration structure with validation.

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpecadoConfig {
    pub version: String,
    pub providers: Vec<Provider>,
    pub routing: RoutingConfig,
    pub connection: ConnectionConfig,
    pub defaults: DefaultConfig,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### `Provider`
**Location**: `src/config/schema.rs:38`

Provider configuration with secrets management.

```rust
#[derive(Clone, Deserialize, Serialize)]
pub struct Provider {
    pub name: String,
    pub provider_type: ProviderType,
    pub api_key: SecretString,  // Secure secret handling
    pub base_url: String,
    pub models: Vec<Model>,
    pub rate_limit: Option<RateLimitConfig>,
    pub retry_policy: Option<RetryPolicy>,
    pub enabled: bool,
    pub priority: u32,
}
```

### Configuration Loading

#### From YAML
**Location**: `src/config/mod.rs:24`

```rust
use specado_core::config::load_from_yaml;

// Load with environment variable interpolation
let config = load_from_yaml("config.yaml")?;

// Validation is automatic
assert!(config.providers.len() > 0);
```

#### From JSON
**Location**: `src/config/mod.rs:52`

```rust
use specado_core::config::load_from_json;

let config = load_from_json("config.json")?;
```

### Environment Variable Interpolation

**Location**: `src/config/env.rs`

The config system supports environment variable interpolation using `${VAR_NAME}` syntax:

```yaml
# config.yaml
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: ${OPENAI_API_KEY}
    base_url: ${OPENAI_BASE_URL:-https://api.openai.com/v1}
```

```rust
// Variables are interpolated during load
let config = load_from_yaml("config.yaml")?;
// api_key will contain the value from environment
```

### Secret Management

**Location**: `src/config/secrets.rs`

Secure handling of sensitive configuration values.

```rust
use specado_core::config::{SecretString, SafeLogging};

// SecretString redacts in debug output
let secret = SecretString::new("sensitive-api-key".to_string());
println!("{:?}", secret);  // Prints: SecretString([REDACTED])

// Safe logging redacts by field name
use specado_core::config::redact_by_field_name;
let safe_json = redact_by_field_name(&original_json, &["api_key", "secret"]);
```

### Configuration Validation

**Location**: `src/config/validator.rs`

Comprehensive validation with detailed error messages.

```rust
use specado_core::config::{ConfigValidator, ValidationError};

let validator = ConfigValidator::new();
match validator.validate(&config) {
    Ok(()) => println!("Configuration valid"),
    Err(ValidationError { kind, field, message, .. }) => {
        eprintln!("Validation failed for {}: {}", field.unwrap_or_default(), message);
    }
}
```

### Error Handling

**Location**: `src/config/error.rs`

```rust
use specado_core::config::{ConfigError, ValidationError};

match load_from_yaml("config.yaml") {
    Ok(config) => { /* use config */ },
    Err(ConfigError::IoError { path, source }) => {
        eprintln!("Failed to read {}: {}", path, source);
    },
    Err(ConfigError::ParseError { path, line, column, message }) => {
        eprintln!("Parse error in {} at {}:{}: {}", path, line.unwrap_or(0), column.unwrap_or(0), message);
    },
    Err(ConfigError::ValidationError(err)) => {
        eprintln!("Validation error: {}", err.message);
    },
    Err(ConfigError::EnvVarError { var, message }) => {
        eprintln!("Environment variable {}: {}", var, message);
    },
}
```

---

## HTTP Client

**Location**: `src/http/mod.rs`

The HTTP module provides connection pooling, request/response transformation, and error mapping for LLM provider APIs.

### Core Types

#### `HttpExecutor`
**Location**: `src/http/mod.rs:112`

Main trait for executing HTTP requests to LLM providers.

```rust
#[async_trait]
pub trait HttpExecutor: Send + Sync {
    async fn execute_json(
        &self,
        provider: &dyn Provider,
        request: ChatRequest,
        options: RequestOptions,
    ) -> Result<ChatResponse, ProviderError>;
    
    // Streaming support (Phase 2)
    async fn execute_stream(
        &self,
        provider: &dyn Provider,
        request: ChatRequest,
        options: RequestOptions,
    ) -> Result<Vec<StreamDelta>, ProviderError>;
}
```

#### `RequestOptions`
**Location**: `src/http/mod.rs:42`

Configuration for HTTP requests with correlation IDs and timeouts.

```rust
#[derive(Debug, Clone)]
pub struct RequestOptions {
    pub call_kind: CallKind,
    pub request_id: Uuid,
    pub timeout: Duration,
    pub idempotency_key: Option<String>,
    pub context_id: Option<String>,
}
```

**Usage Example**:
```rust
use specado_core::http::{RequestOptions, CallKind};
use std::time::Duration;

let options = RequestOptions::new(CallKind::Chat)
    .with_timeout(Duration::from_secs(60))
    .with_idempotency_key("request-123".to_string())
    .with_context_id("conversation-456".to_string());
```

#### `CallKind`
**Location**: `src/http/mod.rs:22`

Enumeration of supported API call types.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallKind {
    Chat,
    // Future: Completions, Embeddings, Images, Audio
}

// Get endpoint path
assert_eq!(CallKind::Chat.endpoint(), "/chat/completions");
```

### HTTP Client Implementation

**Location**: `src/http/client.rs`

```rust
use specado_core::http::client::HttpClient;

// Create client with connection pooling
let client = HttpClient::new();

// Execute request
let response = client.execute_json(&provider, request, options).await?;
```

### Error Handling

**Location**: `src/http/error.rs`

Comprehensive error mapping with retry hints.

```rust
use specado_core::http::error::{HttpError, RetryHint};

match http_result {
    Err(HttpError::Timeout { .. }) => {
        // Handle timeout with possible retry
    },
    Err(HttpError::RateLimited { retry_after, .. }) => {
        // Wait and retry after specified duration
        tokio::time::sleep(retry_after).await;
    },
    Err(HttpError::InvalidResponse { status, body, .. }) => {
        eprintln!("HTTP {}: {}", status, body);
    },
}
```

---

## Protocol Types

**Location**: `src/protocol/mod.rs`

Protocol types define the canonical data models for LLM requests and responses, designed to be provider-agnostic and forward-compatible.

### Core Message Types

#### `ChatRequest`
**Location**: `src/protocol/types.rs`

```rust
use specado_core::protocol::{ChatRequest, Message, MessageRole};

let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![
        Message {
            role: MessageRole::System,
            content: MessageContent::Text("You are a helpful assistant.".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: MessageRole::User,
            content: MessageContent::Text("Hello!".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
    ],
    temperature: Some(0.7),
    max_tokens: Some(150),
    stream: false,
    // ... other fields
};
```

#### `ChatResponse`
**Location**: `src/protocol/types.rs`

```rust
use specado_core::protocol::{ChatResponse, ResponseChoice, CompletionUsage};

// Typical response structure
let response = ChatResponse {
    id: "chatcmpl-123".to_string(),
    object: "chat.completion".to_string(),
    created: 1677652288,
    model: "gpt-4".to_string(),
    choices: vec![
        ResponseChoice {
            index: 0,
            message: Message {
                role: MessageRole::Assistant,
                content: MessageContent::Text("Hello! How can I help you?".to_string()),
                // ...
            },
            finish_reason: Some("stop".to_string()),
        }
    ],
    usage: Some(CompletionUsage {
        prompt_tokens: 20,
        completion_tokens: 10,
        total_tokens: 30,
    }),
    system_fingerprint: None,
};
```

### Message Builder Pattern

**Location**: `src/protocol/types.rs`

```rust
use specado_core::protocol::{MessageBuilder, MessageRole};

// Convenient message building
let message = MessageBuilder::new()
    .role(MessageRole::User)
    .content("What's the weather like?")
    .build();

// With tool calls
let message = MessageBuilder::new()
    .role(MessageRole::Assistant)
    .add_tool_call("get_weather", r#"{"location": "NYC"}"#)
    .build();
```

### Content Types

#### `MessageContent`
**Location**: `src/protocol/types.rs`

Support for text and multimodal content.

```rust
use specado_core::protocol::{MessageContent, ContentPart};

// Text content
let text_content = MessageContent::Text("Hello world".to_string());

// Multimodal content
let multimodal_content = MessageContent::Parts(vec![
    ContentPart::Text { text: "Describe this image:".to_string() },
    ContentPart::Image { 
        image_url: ImageUrl {
            url: "data:image/jpeg;base64,/9j/4AAQ...".to_string(),
            detail: Some("high".to_string()),
        }
    },
]);
```

### Function/Tool Calling

#### `FunctionCall` and `ToolCall`
**Location**: `src/protocol/types.rs`

Support for both OpenAI-style functions and Anthropic-style tools.

```rust
use specado_core::protocol::{FunctionCall, ToolCall, FunctionDefinition};

// Function call (OpenAI style)
let function_call = FunctionCall {
    name: "get_weather".to_string(),
    arguments: r#"{"location": "New York"}"#.to_string(),
};

// Tool call (newer style)
let tool_call = ToolCall {
    id: "call_123".to_string(),
    call_type: "function".to_string(),
    function: function_call,
};

// Function definition
let function_def = FunctionDefinition {
    name: "get_weather".to_string(),
    description: Some("Get current weather for a location".to_string()),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "location": {"type": "string"}
        },
        "required": ["location"]
    }),
};
```

---

## Provider System

**Location**: `src/providers/mod.rs`

The provider system implements adapters for different LLM providers with intelligent routing, retry logic, and transformation tracking.

### Provider Trait

#### `Provider`
**Location**: `src/providers/adapter.rs:13`

Core abstraction that all LLM providers implement.

```rust
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> &ProviderCapabilities;
    fn transform_request(&self, request: ChatRequest) -> ChatRequest;
    fn transform_response(&self, response: ChatResponse) -> ChatResponse;
    fn base_url(&self) -> &str;
    fn endpoint(&self, call_kind: CallKind) -> &str;
    fn headers(&self, api_key: &str) -> HashMap<String, String>;
}
```

### Concrete Providers

#### OpenAI Provider
**Location**: `src/providers/openai.rs`

```rust
use specado_core::providers::OpenAIProvider;

let provider = OpenAIProvider::new();
assert_eq!(provider.name(), "openai");
assert!(provider.capabilities().supports_functions);

// Transform request to OpenAI format
let openai_request = provider.transform_request(canonical_request);
```

#### Anthropic Provider
**Location**: `src/providers/anthropic.rs`

```rust
use specado_core::providers::AnthropicProvider;

let provider = AnthropicProvider::new();
assert_eq!(provider.name(), "anthropic");
assert!(provider.capabilities().supports_tool_use);
```

### Transformation Engine

**Location**: `src/providers/transform.rs`

Tracks lossiness during provider transformations.

```rust
use specado_core::providers::{transform_request, TransformResult, LossinessReason};

let result: TransformResult<ChatRequest> = transform_request(
    &source_request,
    &source_provider,
    &target_provider,
);

match result {
    TransformResult::Lossless(transformed) => {
        // Perfect transformation
    },
    TransformResult::Lossy { transformed, lossiness } => {
        println!("Transformation warnings:");
        for reason in lossiness {
            println!("  - {}", reason);
        }
    },
}
```

### Routing System

**Location**: `src/providers/routing.rs`

Intelligent provider selection with fallbacks.

```rust
use specado_core::providers::{RoutingStrategy, RoutingBuilder};

// Build routing strategy
let strategy = RoutingBuilder::new()
    .add_primary("openai", 1.0)  // Weight 1.0
    .add_fallback("anthropic", 0.8)  // Weight 0.8
    .with_max_retries(3)
    .build();

// Execute with routing
let result = strategy.execute(request, &providers).await?;
```

### Retry System

**Location**: `src/providers/retry.rs`

Configurable retry logic with exponential backoff.

```rust
use specado_core::providers::{RetryPolicy, RetryExecutor};

let retry_policy = RetryPolicy {
    max_retries: 3,
    base_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(30),
    exponential_base: 2.0,
    jitter: true,
};

let executor = RetryExecutor::new(retry_policy);
let result = executor.execute(|| async {
    // Your async operation here
    provider.execute_request(request).await
}).await?;
```

### Adding New Providers

To add a new provider:

1. **Implement Provider trait**:
```rust
use specado_core::providers::{Provider, ProviderCapabilities};

pub struct MyProvider {
    capabilities: ProviderCapabilities,
}

impl Provider for MyProvider {
    fn name(&self) -> &str { "myprovider" }
    
    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }
    
    fn transform_request(&self, mut request: ChatRequest) -> ChatRequest {
        // Transform to provider format
        request
    }
    
    fn transform_response(&self, mut response: ChatResponse) -> ChatResponse {
        // Transform from provider format
        response
    }
    
    fn base_url(&self) -> &str { "https://api.myprovider.com" }
    
    fn endpoint(&self, call_kind: CallKind) -> &str {
        match call_kind {
            CallKind::Chat => "/v1/chat/completions",
        }
    }
    
    fn headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}
```

2. **Register provider type**:
```rust
// In src/providers/adapter.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    MyProvider,  // Add new type
}

impl ProviderType {
    pub fn create_provider(&self) -> Box<dyn Provider> {
        match self {
            // existing cases...
            ProviderType::MyProvider => Box::new(MyProvider::new()),
        }
    }
}
```

3. **Add capability manifest** (optional):
```rust
// In src/capabilities/provider_manifest.rs
impl ProviderManifest {
    pub fn my_provider() -> Self {
        // Define provider capabilities
    }
}
```

---

## Developer Guide

### Testing

The library includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test capabilities
cargo test config
cargo test http

# Run integration tests
cargo test --test '*'

# Run with logging
RUST_LOG=debug cargo test
```

### Examples

Run examples to understand usage patterns:

```bash
# Capability taxonomy demo
cargo run --example capability_usage

# Configuration demo
cargo run --example week1_demo

# HTTP client demo
cargo run --example week2_routing_demo

# JSON transformation demo
cargo run --example test_json_transform
```

### Performance Considerations

1. **Connection Pooling**: HTTP client reuses connections automatically
2. **Async Operations**: All network operations are async with configurable timeouts
3. **Memory Management**: FFI functions use efficient string allocation patterns
4. **Serialization**: Uses `serde` with optimized JSON handling

### Error Handling Patterns

The library uses `thiserror` for structured error types:

```rust
use specado_core::providers::ProviderError;

match result {
    Ok(response) => { /* handle success */ },
    Err(ProviderError::RateLimited { retry_after, .. }) => {
        tokio::time::sleep(retry_after).await;
        // Retry logic
    },
    Err(ProviderError::InvalidModel { model, provider, .. }) => {
        eprintln!("Model {} not supported by {}", model, provider);
    },
    Err(err) => {
        eprintln!("Provider error: {}", err);
    },
}
```

### Logging and Observability

Use `tracing` for structured logging:

```rust
use tracing::{info, warn, error, debug};

// In your application
tracing_subscriber::init();

// Library automatically logs important events
info!("Executing request to provider: {}", provider.name());
warn!("Transformation is lossy: {:?}", lossiness_reasons);
error!("Provider request failed: {}", error);
```

### Extension Points

The library provides several extension points:

1. **Custom Providers**: Implement the `Provider` trait
2. **Custom Capabilities**: Extend the capability taxonomy
3. **Custom Routing**: Implement routing strategies
4. **Custom Validation**: Extend configuration validation
5. **Custom Transformations**: Add transformation logic

### FFI Safety Guidelines

When working with FFI:

1. **Always** pair string-returning functions with `specado_free_string`
2. **Never** pass null pointers to functions expecting valid strings
3. **Check** return values for null before dereferencing
4. **Use** proper error handling in bindings
5. **Test** FFI boundary thoroughly

### Memory Safety

The library is designed with memory safety in mind:

- All FFI functions handle null pointers gracefully
- String allocation uses safe patterns
- Provider implementations are memory-safe
- Async operations handle cancellation correctly

---

## Conclusion

The Specado Core API provides a robust foundation for spec-driven LLM integration with:

- **Type-safe** provider abstractions
- **FFI-safe** interfaces for language bindings
- **Comprehensive** capability modeling
- **Intelligent** routing and fallback
- **Configurable** retry and error handling
- **Extensible** architecture for new providers

For questions or contributions, see the project repository at https://github.com/specado/specado.