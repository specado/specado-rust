# Specado Knowledge Base

## 🧠 Design Philosophy & Principles

### Core Design Philosophy
Specado is built on the principle of **spec-driven LLM integration** that prioritizes:
- **Provider Agnostic**: Abstract away provider-specific differences while preserving unique capabilities
- **Transparency**: Track lossiness and transformations explicitly rather than hiding them
- **Performance**: Zero-copy operations and minimal overhead through Rust core + FFI
- **Safety**: Memory safety through Rust, type safety through structured bindings
- **Composability**: Build complex workflows from simple, well-defined primitives

### Architectural Principles

#### 1. **Single Source of Truth (Rust Core)**
All business logic lives in the Rust core (`specado-core`), with language bindings providing thin wrappers.

**Rationale**: Ensures consistency across languages, maximizes performance, and minimizes maintenance burden.

**Implementation**: 
- Core logic in `specado-core/src/`
- FFI interface in `lib.rs` for C-compatible exports
- Language-specific bindings handle only serialization/deserialization

#### 2. **Capability-Aware Operations**
Every operation considers provider capabilities and tracks transformations.

**Rationale**: Different providers have different capabilities (OpenAI supports function calling, Claude supports tool use, etc.). Rather than forcing a lowest-common-denominator approach, Specado models these differences explicitly.

**Implementation**:
- `capabilities::Capability` struct models provider features
- `comparison::CapabilityComparison` tracks lossiness
- Routing decisions based on capability requirements

#### 3. **Transformation Transparency**
All request/response transformations are tracked and reported to users.

**Rationale**: Users need to understand what changes when requests are transformed between providers.

**Implementation**:
- `providers::transform` module handles all transformations
- `LossinessReport` documents what was lost/changed
- Metadata includes transformation details

#### 4. **Fail-Fast with Graceful Degradation**
Operations fail quickly when impossible, but degrade gracefully when partial success is acceptable.

**Rationale**: Better to fail early with clear error messages than to silently produce incorrect results.

**Implementation**:
- Structured error types with context
- Provider fallback with explicit metadata
- Circuit breaker patterns for reliability

## 🏗️ Technical Architecture Patterns

### FFI Integration Pattern

#### **Memory Management Strategy**
- **Rust Ownership**: Rust core owns all data structures
- **C-Compatible Interface**: FFI functions use `extern "C"` with manual memory management
- **Language Binding Responsibility**: Each binding handles memory lifecycle

```rust
// Pattern: Rust owns, FFI exposes, bindings manage
#[no_mangle]
pub extern "C" fn specado_hello_world() -> *mut c_char {
    let message = hello_world();  // Rust function
    CString::new(message).unwrap().into_raw()  // Transfer ownership
}

// Bindings must call this to prevent leaks
#[no_mangle]
pub unsafe extern "C" fn specado_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);  // Reclaim ownership and drop
    }
}
```

#### **Error Propagation Pattern**
- **Rust Core**: Uses `Result<T, E>` with structured error types
- **FFI Boundary**: Converts to status codes + error strings
- **Language Bindings**: Map to language-native exception systems

```
Rust Result<T, E> → FFI status_code + error_string → Python Exception / JS Promise.reject
```

### Capability System Architecture

#### **Provider Manifest Pattern**
Each provider defines capabilities through a structured manifest:

```rust
pub struct ProviderManifest {
    pub info: ProviderInfo,           // Name, version, description
    pub capability: Capability,       // Feature support matrix
    pub models: Vec<ModelCapability>, // Per-model capabilities
}
```

**Benefits**:
- Declarative capability definition
- Version-aware compatibility checking
- Automatic routing based on requirements

#### **Capability Comparison Algorithm**
The system automatically compares capabilities and tracks lossiness:

```rust
pub enum LossinessType {
    FeatureLoss,        // Feature not supported by target
    ParameterClamp,     // Parameter value adjusted to fit constraints
    ContentTransform,   // Content modified during transformation
    MetadataLoss,       // Metadata stripped or simplified
}
```

### Provider Integration Patterns

#### **Adapter Pattern Implementation**
Each provider implements a common `Provider` trait:

```rust
#[async_trait]
pub trait Provider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse>;
    fn capabilities(&self) -> &Capability;
    fn transform_request(&self, request: &ChatRequest) -> Result<ProviderRequest>;
    fn transform_response(&self, response: ProviderResponse) -> Result<ChatResponse>;
}
```

**Benefits**:
- Consistent interface across providers
- Easy to add new providers
- Transformation logic is provider-specific but interface-consistent

#### **Routing Strategy Pattern**
Request routing follows a sophisticated decision tree:

1. **Capability Matching**: Filter providers by required capabilities
2. **Preference Ordering**: Apply user-defined provider preferences
3. **Fallback Chain**: Configure automatic fallback sequences
4. **Circuit Breaker**: Avoid repeatedly failing providers

## 🔧 Implementation Guidelines

### Adding New Providers

#### **1. Provider Implementation Checklist**
- [ ] Create provider module in `specado-core/src/providers/`
- [ ] Implement `Provider` trait with all methods
- [ ] Define `ProviderManifest` with accurate capabilities
- [ ] Implement request/response transformations
- [ ] Add comprehensive tests
- [ ] Update FFI exports if needed
- [ ] Update language bindings
- [ ] Document lossiness characteristics

#### **2. Capability Modeling Guidelines**
When modeling provider capabilities:

- **Be Accurate**: Don't claim support for features that don't work reliably
- **Be Specific**: Use parameter bounds (min/max) rather than boolean flags
- **Document Quirks**: Use `Extensions.custom` for provider-specific behaviors
- **Version Awareness**: Different model versions may have different capabilities

#### **3. Transformation Implementation**
Request/response transformations should:

- **Preserve Intent**: Maintain the semantic meaning of requests
- **Document Changes**: Return lossiness reports for all modifications
- **Fail Explicitly**: Don't silently drop important information
- **Be Reversible**: When possible, transformations should be bidirectional

### Error Handling Patterns

#### **Structured Error Types**
Use the established error hierarchy:

```rust
#[derive(thiserror::Error, Debug)]
pub enum SpecadoError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Provider error: {provider} - {message}")]
    Provider { provider: String, message: String },
    
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),
    
    #[error("Capability error: {0}")]
    Capability(String),
}
```

#### **Error Context Guidelines**
Every error should include:
- **What Operation Failed**: Specific context about what was being attempted
- **Why It Failed**: Root cause when determinable
- **How to Fix**: Actionable guidance when possible
- **Provider Context**: Which provider/model was involved

### Performance Optimization Patterns

#### **Zero-Copy Operations**
Minimize data copying across FFI boundaries:

```rust
// Good: Pass references through FFI
pub extern "C" fn process_data(data: *const u8, len: usize) -> i32

// Avoid: Creating unnecessary copies
pub extern "C" fn process_data_copy(data: *mut c_char) -> *mut c_char
```

#### **Connection Pooling Strategy**
- **HTTP Clients**: Reuse connections within provider clients
- **Provider Instances**: Cache provider instances across requests
- **Configuration**: Load configuration once, share across operations

#### **Async Integration**
- **Rust Core**: Use `tokio` for async operations
- **Python Bindings**: Bridge to Python's `asyncio` event loop
- **Node.js Bindings**: Integrate with Node.js event loop via NAPI-RS

## 🔗 Integration Best Practices

### Cross-Language Consistency

#### **API Design Patterns**
Maintain consistent patterns across language bindings:

**Configuration**:
```python
# Python
client = specado.Client(config_path="config.yaml")

# Node.js
const client = new specado.Client({ configPath: "config.yaml" });
```

**Error Handling**:
```python
# Python
try:
    response = await client.chat_completion(message)
except specado.ProviderError as e:
    print(f"Provider {e.provider} failed: {e}")

# Node.js
try {
    const response = await client.chatCompletion(message);
} catch (error) {
    if (error instanceof specado.ProviderError) {
        console.log(`Provider ${error.provider} failed: ${error.message}`);
    }
}
```

#### **Type System Alignment**
- **Rust**: Use `serde` for serialization with clear field names
- **Python**: Provide type hints matching Rust struct fields
- **Node.js**: Generate TypeScript definitions from Rust types

### Testing Strategies

#### **Multi-Layer Testing**
1. **Rust Core**: Unit tests for all modules, integration tests for workflows
2. **FFI Interface**: C tests to verify memory management and ABI compliance
3. **Language Bindings**: Language-specific tests for binding logic
4. **End-to-End**: Cross-language integration tests
5. **Performance**: Benchmarks for critical paths

#### **Provider Testing Patterns**
- **Mock Providers**: For unit testing without external dependencies
- **Provider Contracts**: Shared test suites that all providers must pass
- **Capability Validation**: Automated tests to verify declared capabilities
- **Lossiness Testing**: Verify transformation accuracy and reporting

## 🚨 Troubleshooting & Common Issues

### FFI-Related Issues

#### **Memory Management Problems**
**Symptom**: Segfaults, memory leaks, or random crashes
**Diagnosis**: Usually caused by improper string lifecycle management
**Solutions**:
- Ensure every `*mut c_char` from Rust is freed with `specado_free_string`
- Never use freed pointers in language bindings
- Use memory debugging tools (`valgrind`, AddressSanitizer)

#### **ABI Compatibility Issues**
**Symptom**: Crashes on function calls or data corruption
**Diagnosis**: Mismatched calling conventions or struct layouts
**Solutions**:
- Verify `extern "C"` on all exported functions
- Use `#[repr(C)]` for structs passed across FFI
- Test with different compiler versions

### Provider Integration Issues

#### **Capability Mismatch**
**Symptom**: Requests fail or produce unexpected results
**Diagnosis**: Provider capabilities not accurately modeled
**Solutions**:
- Audit provider manifest against actual API behavior
- Add integration tests for edge cases
- Update capability definitions based on real-world usage

#### **Transformation Errors**
**Symptom**: Requests work with one provider but fail with another
**Diagnosis**: Transformation logic not handling provider differences
**Solutions**:
- Review transformation logic for edge cases
- Add comprehensive transformation tests
- Improve error messages to identify transformation failures

### Performance Issues

#### **High Latency**
**Symptom**: Requests taking longer than expected
**Diagnosis**: Could be connection pooling, serialization, or provider issues
**Solutions**:
- Profile with `cargo flamegraph` to identify bottlenecks
- Monitor HTTP connection reuse
- Compare direct provider API calls vs. Specado routing

#### **Memory Usage**
**Symptom**: High memory consumption or growth over time
**Diagnosis**: Possible memory leaks in FFI or caching issues
**Solutions**:
- Use memory profilers to identify leak sources
- Audit FFI string handling
- Review cache eviction policies

## 🔮 Future Evolution & Considerations

### Planned Enhancements

#### **Streaming Support**
**Timeline**: Sprint 2-3
**Implementation Strategy**:
- Extend protocol types to support streaming responses
- Implement async iterators in language bindings
- Handle provider-specific streaming differences

#### **Plugin Architecture**
**Timeline**: Sprint 4-5
**Implementation Strategy**:
- Define plugin interface for custom providers
- Dynamic loading of provider plugins
- Security sandboxing for third-party plugins

#### **Observability**
**Timeline**: Sprint 5
**Implementation Strategy**:
- OpenTelemetry integration for tracing
- Metrics collection for performance monitoring
- Structured logging throughout the system

### Architectural Evolution

#### **Multi-Modal Support**
Current architecture supports multi-modal capabilities in the taxonomy, but implementation is text-focused. Future expansion:
- Image/audio processing pipelines
- Provider-specific modal transformation
- Unified interface for multi-modal interactions

#### **Distributed Deployment**
Current architecture is single-process. Future scalability:
- Service mesh deployment patterns
- Load balancing across multiple instances
- Shared capability discovery and routing

#### **Advanced Routing**
Current routing is capability-based. Future enhancements:
- Cost-aware routing optimization
- Performance-based provider selection
- A/B testing infrastructure for provider comparison

### Technology Evolution

#### **Rust Ecosystem**
- Keep up with async Rust improvements
- Evaluate new serialization frameworks
- Consider WebAssembly compilation for additional platforms

#### **Language Binding Evolution**
- **Python**: Monitor PyO3 developments, consider async improvements
- **Node.js**: Follow NAPI-RS evolution, evaluate TypeScript improvements
- **Additional Languages**: Go, Java, C# bindings based on demand

#### **AI/ML Ecosystem**
- Monitor LLM provider API evolution
- Adapt to new provider capabilities (multimodal, agents, etc.)
- Integration with emerging AI infrastructure patterns

## 📚 Decision Context & Rationale

### Key Architectural Decisions

#### **Why Rust Core?**
**Decision**: Use Rust for core library with FFI bindings
**Context**: Need for performance, safety, and cross-language support
**Alternatives Considered**: 
- Pure Python/Node.js implementation
- C++ core with bindings
- Language-specific native implementations

**Rationale**:
- Memory safety without garbage collection overhead
- Excellent FFI support with minimal runtime
- Growing ecosystem for HTTP/async operations
- Single codebase for core logic across languages

#### **Why FFI Over gRPC/HTTP?**
**Decision**: Use FFI for language integration
**Context**: Need for low-latency, high-throughput operations
**Alternatives Considered**:
- gRPC service architecture
- HTTP API with language-specific clients
- Language-specific reimplementations

**Rationale**:
- Lower latency for frequent operations
- Simpler deployment (single binary vs. service mesh)
- Better error propagation and debugging
- Consistent behavior across languages

#### **Why Capability Taxonomy?**
**Decision**: Model provider differences explicitly rather than abstracting them away
**Context**: LLM providers have genuinely different capabilities
**Alternatives Considered**:
- Lowest-common-denominator API
- Provider-specific interfaces
- Plugin architecture from the start

**Rationale**:
- Preserves provider strengths rather than limiting to common features
- Enables intelligent routing based on requirements
- Provides transparency about transformations and limitations
- Allows incremental capability adoption

### Technology Choices

#### **PyO3 for Python Bindings**
**Decision**: Use PyO3 0.23+ for Python integration
**Context**: Need for type-safe, performant Python bindings
**Alternatives**: ctypes, Cython, pybind11

**Rationale**:
- Excellent Rust integration with automatic type conversion
- Strong type safety with Python type hints
- Active development and community
- Good performance characteristics

#### **NAPI-RS for Node.js Bindings**
**Decision**: Use NAPI-RS for Node.js integration
**Context**: Need for native async support and TypeScript compatibility
**Alternatives**: node-addon-api, native N-API, WebAssembly

**Rationale**:
- Native async/await support without callback conversion
- Excellent TypeScript definition generation
- Cross-platform compatibility
- Performance advantages over WASM for I/O-heavy operations

---

**Knowledge Base Version**: 1.0.0  
**Last Updated**: Sprint 0 completion - January 2025  
**Maintainers**: Specado Core Team  
**Review Cycle**: Every sprint cycle