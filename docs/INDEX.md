# Specado Documentation Index

## 📚 Documentation Overview

This documentation provides comprehensive coverage of the Specado spec-driven LLM integration library, from architecture to implementation details across all supported language bindings.

## 🏗️ Architecture & Design

### [System Architecture](../ARCHITECTURE.md)
**Comprehensive system design and architectural decisions**
- High-level system overview with component relationships
- FFI integration patterns and data flow diagrams
- Architectural Decision Records (ADRs) for key design choices
- Security, performance, and scalability considerations
- Future enhancement roadmap

**Key Sections:**
- [System Architecture Overview](../ARCHITECTURE.md#system-architecture)
- [Component Architecture](../ARCHITECTURE.md#component-architecture)
- [Data Flow Diagrams](../ARCHITECTURE.md#data-flow-diagrams)
- [Architectural Decision Records](../ARCHITECTURE.md#architectural-decision-records)

## 🦀 Rust Core Library

### [Rust Core API Reference](rust-core-api.md)
**Complete API documentation for the Rust core library**
- FFI interface documentation with memory management patterns
- Core module APIs: capabilities, config, http, protocol, providers
- Developer guide for extending the capability system
- Testing and performance considerations

**Key Sections:**
- [FFI Interface](rust-core-api.md#ffi-interface) - C-compatible exports for language bindings
- [Capabilities Module](rust-core-api.md#capabilities-module) - Provider capability taxonomy
- [Configuration Module](rust-core-api.md#configuration-module) - YAML/JSON loading and validation
- [HTTP Module](rust-core-api.md#http-module) - Async client and error handling
- [Protocol Module](rust-core-api.md#protocol-module) - Message types and builders
- [Providers Module](rust-core-api.md#providers-module) - Adapters, routing, and transformations

## 🐍 Python Integration

### [Python API Reference](python-api.md)
**Complete Python bindings documentation using PyO3**
- Installation and setup for Python 3.9-3.12
- Complete API reference with type hints
- PyO3 integration patterns and best practices
- Error handling and exception mapping

**Key Sections:**
- [Installation & Setup](python-api.md#installation-and-setup) - PyPI and development setup
- [Python API Reference](python-api.md#python-api-reference) - Complete class and function documentation
- [Usage Examples](python-api.md#usage-examples) - Common patterns and scenarios
- [PyO3 Integration](python-api.md#pyo3-integration-patterns) - Type conversion and memory management

## 🟢 Node.js Integration

### [Node.js API Reference](nodejs-api.md)
**Complete Node.js bindings documentation using NAPI-RS**
- Installation and setup for Node.js 16+
- TypeScript type definitions and examples
- Async patterns and error handling
- NAPI-RS integration and performance considerations

**Key Sections:**
- [Installation & Setup](nodejs-api.md#installation-and-setup) - npm/yarn/pnpm installation
- [TypeScript Integration](nodejs-api.md#typescript-integration) - Type definitions and IDE support
- [API Reference](nodejs-api.md#api-reference) - Complete interface documentation
- [Usage Examples](nodejs-api.md#usage-examples) - Async patterns and error handling

## 🔧 Development & Operations

### [CI/CD Pipeline](ci-cd.md)
**Complete development workflow and automation documentation**
- GitHub Actions workflow architecture
- Multi-platform build and test processes
- Release automation and package publishing
- Quality assurance and security checks

**Key Sections:**
- [Pipeline Overview](ci-cd.md#pipeline-overview) - Workflow architecture and triggers
- [Development Workflows](ci-cd.md#development-workflows) - Local development and PR process
- [Build & Test Process](ci-cd.md#build-and-test-process) - Multi-language build configuration
- [Quality Assurance](ci-cd.md#quality-assurance) - Formatting, linting, and security

### [Development Setup](../DEVELOPMENT.md)
**Getting started with development environment setup**
- Prerequisites and tool installation
- Local build and test procedures
- IDE configuration and debugging

### [Release Process](../RELEASING.md)
**Release management and versioning procedures**
- Version management across workspace
- Release checklist and procedures
- Package publishing workflows

## 🔗 Cross-Reference Guide

### Core Concepts Across Languages

| Concept | Rust Core | Python | Node.js | Architecture |
|---------|-----------|--------|---------|--------------|
| **Client Configuration** | [config::Config](rust-core-api.md#configuration-module) | [specado.Client](python-api.md#client-class) | [Client class](nodejs-api.md#client-class) | [Config Architecture](../ARCHITECTURE.md#configuration-architecture) |
| **Capability System** | [capabilities::Capability](rust-core-api.md#capabilities-module) | [get_*_manifest()](python-api.md#capability-functions) | [capability functions](nodejs-api.md#capability-functions) | [Capability Taxonomy](../ARCHITECTURE.md#capability-taxonomy) |
| **Provider Integration** | [providers::Provider](rust-core-api.md#providers-module) | [Client routing](python-api.md#provider-routing) | [routing config](nodejs-api.md#routing-configuration) | [Provider Architecture](../ARCHITECTURE.md#provider-architecture) |
| **Error Handling** | [Error types](rust-core-api.md#error-handling) | [Python exceptions](python-api.md#error-handling) | [Promise rejection](nodejs-api.md#error-handling) | [Error Flow](../ARCHITECTURE.md#error-handling-architecture) |
| **FFI Interface** | [FFI functions](rust-core-api.md#ffi-interface) | [PyO3 bindings](python-api.md#pyo3-integration-patterns) | [NAPI-RS bindings](nodejs-api.md#napi-rs-integration) | [FFI Architecture](../ARCHITECTURE.md#ffi-integration-patterns) |

### Key Integration Points

#### **Capability Discovery**
- **Architecture**: [Capability Flow](../ARCHITECTURE.md#capability-discovery-flow)
- **Rust**: [ProviderManifest](rust-core-api.md#provider-manifest)
- **Python**: [get_openai_manifest()](python-api.md#capability-discovery)
- **Node.js**: [getOpenaiManifest()](nodejs-api.md#capability-discovery)

#### **Request/Response Flow**
- **Architecture**: [Request Flow Diagram](../ARCHITECTURE.md#request-response-flow)
- **Rust**: [Protocol Types](rust-core-api.md#protocol-types)
- **Python**: [ChatCompletionResponse](python-api.md#response-handling)
- **Node.js**: [Response Interface](nodejs-api.md#response-interface)

#### **Provider Fallback**
- **Architecture**: [Fallback Architecture](../ARCHITECTURE.md#provider-fallback-architecture)
- **Rust**: [Routing Module](rust-core-api.md#routing-module)
- **Python**: [Fallback Configuration](python-api.md#fallback-configuration)
- **Node.js**: [Routing Configuration](nodejs-api.md#routing-configuration)

## 📖 Quick Start by Language

### Python Developers
1. Start with [Python Installation](python-api.md#installation-and-setup)
2. Review [Basic Usage Examples](python-api.md#basic-usage)
3. Understand [Error Handling](python-api.md#error-handling)
4. Explore [Advanced Patterns](python-api.md#advanced-usage)

### Node.js/TypeScript Developers
1. Start with [Node.js Installation](nodejs-api.md#installation-and-setup)
2. Review [TypeScript Integration](nodejs-api.md#typescript-integration)
3. Understand [Async Patterns](nodejs-api.md#async-patterns)
4. Explore [Performance Optimization](nodejs-api.md#performance-optimization)

### Rust Developers
1. Start with [Architecture Overview](../ARCHITECTURE.md#system-architecture)
2. Review [FFI Interface](rust-core-api.md#ffi-interface)
3. Understand [Core Modules](rust-core-api.md#core-modules)
4. Explore [Extension Patterns](rust-core-api.md#extension-patterns)

### DevOps Engineers
1. Start with [CI/CD Overview](ci-cd.md#pipeline-overview)
2. Review [Development Workflows](ci-cd.md#development-workflows)
3. Understand [Release Process](ci-cd.md#deployment-and-release)
4. Explore [Monitoring](ci-cd.md#monitoring-and-maintenance)

## 🎯 Common Tasks

### Adding a New Provider
1. **Architecture**: Review [Provider Architecture](../ARCHITECTURE.md#provider-architecture)
2. **Rust Implementation**: Follow [Provider Extension Guide](rust-core-api.md#adding-new-providers)
3. **Python Integration**: Update [capability functions](python-api.md#capability-functions)
4. **Node.js Integration**: Update [TypeScript definitions](nodejs-api.md#typescript-definitions)
5. **Testing**: Add tests following [CI/CD Testing Guide](ci-cd.md#testing-strategy)

### Capability System Extension
1. **Architecture**: Review [Capability Taxonomy](../ARCHITECTURE.md#capability-taxonomy)
2. **Rust Core**: Extend [Capability struct](rust-core-api.md#capability-struct)
3. **Python Bindings**: Update [PyO3 mappings](python-api.md#capability-mapping)
4. **Node.js Bindings**: Update [NAPI-RS exports](nodejs-api.md#capability-exports)

### Performance Optimization
1. **Architecture**: Review [Performance Architecture](../ARCHITECTURE.md#performance-architecture)
2. **Rust**: Apply [Performance Patterns](rust-core-api.md#performance-considerations)
3. **Python**: Use [Async Patterns](python-api.md#performance-optimization)
4. **Node.js**: Leverage [Native Performance](nodejs-api.md#performance-optimization)

## 📝 Contributing

1. **Development Setup**: [DEVELOPMENT.md](../DEVELOPMENT.md)
2. **Architecture Understanding**: [ARCHITECTURE.md](../ARCHITECTURE.md)
3. **Component Documentation**: Choose relevant API documentation
4. **CI/CD Process**: [ci-cd.md](ci-cd.md)
5. **Release Process**: [RELEASING.md](../RELEASING.md)

---

**Last Updated**: Sprint 0 completion - January 2025  
**Documentation Version**: 1.0.0  
**Project Status**: Pre-Alpha Development