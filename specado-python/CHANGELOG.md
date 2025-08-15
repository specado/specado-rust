# Changelog

All notable changes to the Specado Python package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-08-15

### Added
- **Core Transformation Engine**: OpenAI ↔ Anthropic request/response format conversion
  - Automatic detection and handling of format differences
  - System role merging for Anthropic compatibility
  - Comprehensive lossiness tracking with detailed reasons
  
- **Automatic Fallback Routing**: Intelligent provider switching on failures
  - Primary-to-fallback routing strategy with configurable providers
  - Retry policies with exponential backoff for transient errors
  - Error classification (retryable vs non-retryable)
  
- **Python Bindings**: High-performance PyO3-based Python interface
  - OpenAI-compatible `Client` API with `chat.completions.create()`
  - `Message` class for structured message creation
  - Full async/sync operation support
  
- **Comprehensive Error Handling**: Production-ready error management
  - Provider-specific error mapping (timeout, rate limits, auth failures)
  - Automatic retry logic with intelligent backoff
  - Detailed error context and recovery information
  
- **Rich Metadata and Observability**: Complete request/response lifecycle tracking
  - Provider usage tracking (`provider_used`, `fallback_triggered`)
  - Attempt counting and timing information
  - Transformation metadata including lossiness indicators
  - Routing decision audit trail
  
- **Configuration System**: Flexible provider and routing configuration
  - Primary/fallback provider specification
  - Custom retry policies and timeout settings
  - Metadata tracking controls

### Technical Details
- Built with Rust core for high performance and memory safety
- PyO3 0.25.1 bindings with Python 3.9+ stable ABI support
- Tokio async runtime for non-blocking operations
- Comprehensive test suite with integration tests
- Maturin-based build system for cross-platform wheels

### Week 2 MVP Scope
This release represents the Week 2 MVP of the Specado project, focusing on:
- **Transformation**: Proven OpenAI ↔ Anthropic format conversion
- **Routing**: Reliable fallback between providers
- **Python Integration**: Production-ready Python bindings
- **Developer Experience**: Clear API and comprehensive documentation

### Known Limitations
- Currently supports OpenAI and Anthropic providers only
- No streaming response support yet
- Limited to chat completion endpoints
- Basic retry policies (advanced circuit breakers planned)

## Links
- [Repository](https://github.com/specado/specado-rust)
- [Issues](https://github.com/specado/specado-rust/issues)
- [Documentation](https://docs.specado.com)

[Unreleased]: https://github.com/specado/specado-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/specado/specado-rust/releases/tag/v0.1.0