# Specado Architecture Documentation

## Overview

Specado is a spec-driven LLM integration library built with a **Rust core + FFI bindings** architecture. This design provides high performance, memory safety, and language interoperability while maintaining a unified abstraction layer across different LLM providers.

### Core Architectural Principles

1. **Provider Agnostic**: Unified interface abstracts provider-specific differences
2. **Type Safety**: Rust core ensures memory safety and compile-time guarantees
3. **Language Interoperability**: FFI-based bindings for Python and Node.js
4. **Spec-Driven Design**: Configuration-based provider selection and transformation
5. **Performance First**: Zero-copy transformations where possible, async operations
6. **Capability-Aware**: Rich capability taxonomy for provider comparison and selection

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Application Layer                             │
├─────────────────────┬─────────────────────┬─────────────────────┤
│   Python Apps      │    Node.js Apps     │     Rust Apps       │
│                     │                     │                     │
│  import specado     │  const specado =    │  use specado_core   │
│  client.chat.       │  require('specado') │  ChatRequest::new() │
│  completions.create │  client.chat.       │                     │
│                     │  completions.create │                     │
└─────────────────────┴─────────────────────┴─────────────────────┘
           │                       │                       │
           │                       │                       │
┌─────────────────────┬─────────────────────┬─────────────────────┐
│   PyO3 Bindings     │   NAPI-RS Bindings  │    Direct Access    │
│  (specado-python)   │   (specado-node)    │                     │
│                     │                     │                     │
│  • Type conversion  │  • Type conversion  │  • Native types     │
│  • Error mapping    │  • Error mapping    │  • Zero overhead    │
│  • Async bridging   │  • Native async     │                     │
└─────────────────────┴─────────────────────┴─────────────────────┘
           │                       │                       │
           └───────────────────────┼───────────────────────┘
                                   │
┌─────────────────────────────────────────────────────────────────┐
│                     Specado Core (Rust)                        │
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────┐ │
│  │  Capabilities │  │  Protocol     │  │  Configuration      │ │
│  │               │  │               │  │                     │ │
│  │  • Taxonomy   │  │  • ChatRequest│  │  • Schema           │ │
│  │  • Comparison │  │  • Message    │  │  • Validation       │ │
│  │  • Manifests  │  │  • Response   │  │  • Env Variables    │ │
│  └───────────────┘  └───────────────┘  └─────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                 Provider Layer                              │ │
│  │                                                             │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │  Transform  │  │   Routing   │  │     Retry           │ │ │
│  │  │             │  │             │  │                     │ │ │
│  │  │ • Request   │  │ • Strategy  │  │ • Exponential       │ │ │
│  │  │ • Response  │  │ • Fallback  │  │ • Circuit Breaker   │ │ │
│  │  │ • Lossiness │  │ • Load Bal. │  │ • Error Recovery    │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                 Provider Adapters                           │ │
│  │                                                             │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │   OpenAI    │  │  Anthropic  │  │    Future           │ │ │
│  │  │             │  │             │  │    Providers        │ │ │
│  │  │ • Chat API  │  │ • Messages  │  │                     │ │ │
│  │  │ • Functions │  │ • Tools     │  │ • Google AI         │ │ │
│  │  │ • Streaming │  │ • Streaming │  │ • Mistral           │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    HTTP Layer                               │ │
│  │                                                             │ │
│  │  • Reqwest client with connection pooling                  │ │
│  │  • TLS termination (rustls)                                │ │
│  │  • Request/response logging and metrics                    │ │
│  │  • Rate limiting and timeout handling                      │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. Specado Core (Rust)

The heart of the system, providing all core functionality through a C-compatible FFI interface.

#### Key Modules

##### Capabilities Module (`src/capabilities/`)
```rust
pub struct Capability {
    pub version: String,
    pub modalities: ModalitySupport,      // Text, Images, Audio, Video
    pub features: ModelFeatures,          // Function calling, JSON mode, etc.
    pub parameters: ControlParameters,    // Temperature, top_p, max_tokens
    pub roles: RoleSupport,              // System, user, assistant, tool
    pub constraints: Constraints,         // Rate limits, token limits
    pub extensions: Extensions,          // Custom capabilities
}
```

**Responsibilities:**
- Define provider capability taxonomy
- Enable capability comparison and lossiness analysis
- Provide capability discovery for routing decisions
- Support capability-based provider selection

##### Protocol Module (`src/protocol/`)
```rust
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    // ... other parameters
}

pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ResponseChoice>,
    pub usage: Option<CompletionUsage>,
}
```

**Responsibilities:**
- Define canonical request/response structures
- Provider-agnostic message format
- Support for multimodal content
- Extensible metadata system

##### Provider Layer (`src/providers/`)

###### Transform Engine (`transform.rs`)
```rust
pub struct TransformationEngine {
    pub fn transform_request(
        &self,
        request: &ChatRequest,
        target_capability: &Capability
    ) -> TransformResult<serde_json::Value>
}

pub struct TransformResult<T> {
    pub result: T,
    pub lossiness: LossinessReport,
    pub warnings: Vec<String>,
}
```

###### Routing Strategy (`routing.rs`)
```rust
pub trait RoutingStrategy {
    async fn route(&self, request: ChatRequest) -> Result<RoutingResult>;
}

pub struct PrimaryWithFallbacks {
    primary: Box<dyn Provider>,
    fallbacks: Vec<Box<dyn Provider>>,
    retry_policy: RetryPolicy,
}
```

###### Provider Adapters
```rust
pub trait Provider: Send + Sync {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse>;
    fn capabilities(&self) -> &Capability;
    fn provider_type(&self) -> ProviderType;
}
```

### 2. Python Bindings (PyO3)

**File:** `specado-python/src/lib.rs`

#### Architecture Pattern
- **Wrapper Types**: Python-compatible structs that mirror core types
- **Type Conversion**: Bidirectional conversion between Python and Rust types
- **Error Mapping**: Rust errors → Python exceptions with context
- **Async Bridging**: Tokio runtime for blocking Python async operations

#### Key Components

```python
# Python API Surface
import specado

client = specado.Client({
    'primary_provider': 'openai',
    'fallback_provider': 'anthropic'
})

response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        specado.Message("system", "You are a helpful assistant"),
        specado.Message("user", "Hello!")
    ],
    temperature=0.7
)

print(f"Provider used: {response.extensions.provider_used}")
print(f"Response: {response.choices[0].message.content}")
```

#### Data Flow
```
Python Dict → PyO3 → Rust HashMap → ChatRequest → Provider → ChatResponse → PyO3 → Python Object
```

### 3. Node.js Bindings (NAPI-RS)

**File:** `specado-node/src/lib.rs`

#### Architecture Pattern
- **Native Objects**: JavaScript-compatible structs using `#[napi(object)]`
- **Async Native**: True async support without blocking the event loop
- **Type Safety**: Compile-time guarantees for JavaScript interface
- **Performance**: Zero-copy where possible, efficient serialization

#### Key Components

```javascript
// Node.js API Surface
const specado = require('specado');

const client = new specado.Client({
    primary_provider: 'openai',
    fallback_provider: 'anthropic'
});

const response = await client.getChat().getCompletions().create(
    "gpt-4",
    [
        { role: "system", content: "You are a helpful assistant" },
        { role: "user", content: "Hello!" }
    ],
    0.7,  // temperature
    150   // max_tokens
);

console.log(`Provider used: ${response.extensions.provider_used}`);
console.log(`Response: ${response.choices[0].message.content}`);
```

## Data Flow Architecture

### Request Flow

```
Application Request
       │
       ▼
┌─────────────────┐
│ Language Binding │ ◄── Type conversion, validation
│ (PyO3/NAPI-RS)  │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Specado Core    │ ◄── Capability-aware routing
│ Router          │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Transform       │ ◄── Request transformation
│ Engine          │     based on target capabilities
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Provider        │ ◄── HTTP request to LLM API
│ Adapter         │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ LLM Provider    │ ◄── OpenAI, Anthropic, etc.
│ API             │
└─────────────────┘
```

### Response Flow with Fallback

```
LLM Provider API Response
       │
       ▼
┌─────────────────┐
│ Provider        │ ◄── Parse response, handle errors
│ Adapter         │
└─────────────────┘
       │
       ▼ (on error)
┌─────────────────┐
│ Retry Logic &   │ ◄── Exponential backoff,
│ Fallback Router │     circuit breaker
└─────────────────┘
       │
       ▼ (if needed)
┌─────────────────┐
│ Fallback        │ ◄── Secondary provider
│ Provider        │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Response        │ ◄── Transform to canonical format
│ Transform       │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Language Binding │ ◄── Type conversion back to language
│ Response        │
└─────────────────┘
       │
       ▼
Application Response with Metadata
```

### Capability Discovery Flow

```
Request with Model Requirements
       │
       ▼
┌─────────────────┐
│ Capability      │ ◄── Load provider manifests
│ Registry        │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Capability      │ ◄── Compare required vs available
│ Comparison      │     capabilities
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Lossiness       │ ◄── Calculate transformation loss
│ Analysis        │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│ Provider        │ ◄── Select best provider based on
│ Selection       │     capabilities and lossiness
└─────────────────┘
```

## Architectural Decision Records (ADRs)

### ADR-001: Rust Core with FFI Bindings

**Decision:** Use Rust as the core implementation language with FFI bindings for Python and Node.js.

**Rationale:**
- **Performance**: Rust provides zero-cost abstractions and memory safety without garbage collection overhead
- **Safety**: Compile-time guarantees prevent memory leaks and data races in a concurrent environment
- **Interoperability**: C-compatible FFI allows bindings to any language
- **Ecosystem**: Rich ecosystem for HTTP clients, async programming, and serialization
- **Maintainability**: Single source of truth for core logic reduces duplication

**Consequences:**
- ✅ Excellent performance for HTTP-intensive workloads
- ✅ Memory safety guarantees
- ✅ Single implementation to maintain and test
- ❌ FFI complexity requires careful memory management
- ❌ Rust learning curve for contributors

**Alternatives Considered:**
- Pure Python/Node.js implementations (rejected: performance, duplication)
- C/C++ core (rejected: safety concerns, development velocity)
- Go with cgo bindings (rejected: GC overhead, less mature FFI story)

### ADR-002: PyO3 for Python Bindings

**Decision:** Use PyO3 for Python bindings instead of manual FFI or other binding generators.

**Rationale:**
- **Type Safety**: Compile-time checking of Python API surface
- **Ergonomics**: Rust-native development experience with Python integration
- **Performance**: Minimal overhead for type conversions
- **Ecosystem**: Active community and excellent documentation
- **Error Handling**: Structured error mapping from Rust to Python exceptions

**Consequences:**
- ✅ Type-safe Python API development
- ✅ Excellent performance characteristics
- ✅ Native Python object integration
- ❌ Additional build complexity (maturin required)
- ❌ PyO3 version compatibility requirements

### ADR-003: NAPI-RS for Node.js Bindings

**Decision:** Use NAPI-RS for Node.js bindings instead of manual N-API or other solutions.

**Rationale:**
- **Native Async**: True async support without blocking the Node.js event loop
- **Type System**: Compile-time type checking for JavaScript interface
- **Performance**: Zero-copy operations where possible
- **Stability**: Uses stable N-API instead of internal V8 APIs
- **Tooling**: Excellent build and development tooling

**Consequences:**
- ✅ Native async performance
- ✅ Stable ABI across Node.js versions
- ✅ Type-safe JavaScript API
- ❌ Node.js 16+ requirement
- ❌ Learning curve for NAPI concepts

### ADR-004: Capability Taxonomy Design

**Decision:** Implement a structured capability taxonomy for provider comparison and routing decisions.

**Rationale:**
- **Provider Agnostic**: Abstract differences between provider capabilities
- **Routing Intelligence**: Enable automatic provider selection based on requirements
- **Lossiness Tracking**: Quantify what is lost in provider transformations
- **Future Proofing**: Extensible system for new capabilities and providers

**Architecture:**
```rust
struct Capability {
    modalities: ModalitySupport,    // What inputs/outputs are supported
    features: ModelFeatures,        // What features are available  
    parameters: ControlParameters,  // What parameters can be controlled
    constraints: Constraints,       // Rate limits, token limits, etc.
}
```

**Consequences:**
- ✅ Intelligent provider routing
- ✅ Transparent capability discovery
- ✅ Lossiness-aware transformations
- ❌ Additional complexity in provider adapters
- ❌ Capability manifest maintenance overhead

### ADR-005: Provider Transform Engine

**Decision:** Implement a transformation engine that converts between provider formats while tracking lossiness.

**Rationale:**
- **Transparency**: Users understand what is lost in transformations
- **Debugging**: Clear audit trail for transformation decisions
- **Quality**: Prioritize providers that support requested features natively
- **Extensibility**: Plugin architecture for custom transformations

**Implementation:**
```rust
pub fn transform_request(
    request: &ChatRequest,
    target_capability: &Capability
) -> TransformResult<serde_json::Value> {
    // Transform request format
    // Track what capabilities are lost
    // Generate warnings for lossy transformations
}
```

**Consequences:**
- ✅ Transparent transformation process
- ✅ Quality-aware provider selection
- ✅ Rich debugging information
- ❌ Implementation complexity
- ❌ Performance overhead for transformation tracking

## Security Architecture

### Memory Safety
- **Rust Core**: Memory safety guaranteed at compile time
- **FFI Boundaries**: Careful validation of all data crossing FFI boundaries
- **String Handling**: Proper C string lifecycle management with `specado_free_string`

### API Security
- **Input Validation**: All requests validated before processing
- **Secret Management**: Secure handling of API keys and sensitive configuration
- **Error Sanitization**: No sensitive data leaked in error messages

### Network Security
- **TLS**: All external requests use TLS 1.2+ with certificate validation
- **Connection Pooling**: Secure connection reuse with proper lifecycle management
- **Rate Limiting**: Respect provider rate limits and implement backoff

## Performance Architecture

### Core Performance Characteristics
- **Zero-Copy**: Minimize data copying at FFI boundaries
- **Async**: Non-blocking I/O throughout the stack
- **Connection Pooling**: Reuse HTTP connections for better latency
- **Streaming**: Support for streaming responses (future enhancement)

### Language Binding Performance
- **Python**: Tokio runtime managed for optimal async performance
- **Node.js**: Native async integration with the event loop
- **Type Conversion**: Efficient serialization between language boundaries

### Scalability Considerations
- **Stateless**: No shared state between requests enables horizontal scaling
- **Provider Fallback**: Multiple provider support distributes load
- **Circuit Breaker**: Fail fast to prevent cascade failures

## Future Architecture Enhancements

### Planned Features

#### Streaming Support
```rust
pub trait StreamingProvider {
    fn stream_complete(&self, request: ChatRequest) 
        -> Result<Pin<Box<dyn Stream<Item = ChatStreamChunk>>>>;
}
```

#### Plugin Architecture
```rust
pub trait TransformPlugin {
    fn name(&self) -> &str;
    fn transform(&self, input: &serde_json::Value) -> Result<serde_json::Value>;
}
```

#### Metrics and Observability
```rust
pub struct MetricsCollector {
    pub fn record_request(&self, provider: &str, latency: Duration, tokens: usize);
    pub fn record_error(&self, provider: &str, error_type: &str);
}
```

### Extensibility Points

1. **Provider Adapters**: Plugin system for new LLM providers
2. **Transform Plugins**: Custom transformation logic
3. **Routing Strategies**: Custom routing and load balancing algorithms
4. **Capability Extensions**: Support for provider-specific capabilities
5. **Observability Hooks**: Custom metrics and tracing integration

## Development and Testing Architecture

### Build System
- **Workspace**: Unified Cargo workspace for all Rust components
- **CI/CD**: Comprehensive testing across Python 3.9-3.12 and Node.js 16-20
- **Cross-Platform**: Support for Linux, macOS, and Windows

### Testing Strategy
- **Unit Tests**: Core logic testing in Rust
- **Integration Tests**: FFI boundary testing
- **Property Tests**: Capability transformation correctness
- **End-to-End Tests**: Full workflow testing with mock providers

### Quality Assurance
- **Linting**: Rustfmt, Clippy for code quality
- **Security**: Cargo audit for dependency vulnerabilities
- **Performance**: Criterion.rs for benchmarking critical paths

---

This architecture provides a solid foundation for spec-driven LLM integration while maintaining performance, safety, and language interoperability. The modular design enables incremental enhancement and supports the project's growth from Sprint 1 through production deployment.