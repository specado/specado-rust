# Changelog

All notable changes to the @specado/core Node.js package will be documented in this file.

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
  
- **High-Performance Node.js Bindings**: NAPI-RS-based native bindings
  - OpenAI-compatible API with `Client`, `Chat`, and `ChatCompletions` classes
  - `createMessage` function for structured message creation
  - Full async/await operation support with Promise integration
  - TypeScript definitions with comprehensive type safety
  
- **Comprehensive Error Handling**: Production-ready error management
  - Provider-specific error mapping (timeout, rate limits, auth failures)
  - Automatic retry logic with intelligent backoff
  - Detailed error context and recovery information
  
- **Rich Metadata and Observability**: Complete request/response lifecycle tracking
  - Provider usage tracking (`providerUsed`, `fallbackTriggered`)
  - Attempt counting and timing information
  - Transformation metadata including lossiness indicators
  - Routing decision audit trail
  
- **Configuration System**: Flexible provider and routing configuration
  - Primary/fallback provider specification via Client constructor
  - Custom retry policies and timeout settings
  - Metadata tracking controls

### Technical Details
- Built with Rust core for high performance and memory safety
- NAPI-RS 3.1.4 bindings with Node.js 16+ support
- Zero-copy operations for efficient data transfer
- Multi-platform prebuilt binaries (macOS, Linux, Windows)
- Cross-architecture support (x64, ARM64)
- Comprehensive test suite with integration tests

### Development Experience
- Full TypeScript support with generated type definitions
- OpenAI-compatible API for easy migration
- Comprehensive error messages and debugging support
- Development scripts for building, testing, and verification
- Hot-reload support in watch mode

### Week 2 MVP Scope
This release represents the Week 2 MVP of the Specado project, focusing on:
- **Transformation**: Proven OpenAI ↔ Anthropic format conversion
- **Routing**: Reliable fallback between providers
- **Node.js Integration**: Production-ready NAPI-RS bindings
- **Developer Experience**: TypeScript-first API and comprehensive documentation

### Platform Support
- **Node.js**: 16.0.0 or higher
- **Operating Systems**: macOS (Intel/Apple Silicon), Linux (x64/ARM64), Windows (x64/ARM64)
- **Package Managers**: npm, yarn, pnpm
- **Architectures**: x86_64, aarch64

### Known Limitations
- Currently supports OpenAI and Anthropic providers only
- No streaming response support yet
- Limited to chat completion endpoints
- Basic retry policies (advanced circuit breakers planned)

### Performance Characteristics
- **Binding Overhead**: < 1ms for typical API calls
- **Memory Usage**: Minimal heap allocation with Rust core
- **Concurrent Operations**: Thread-safe with async support
- **Package Size**: Optimized native binaries (~5-10MB per platform)

## Links
- [Repository](https://github.com/specado/specado-rust)
- [Issues](https://github.com/specado/specado-rust/issues)
- [npm Package](https://www.npmjs.com/package/@specado/core)
- [Documentation](https://docs.specado.com)

[Unreleased]: https://github.com/specado/specado-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/specado/specado-rust/releases/tag/v0.1.0