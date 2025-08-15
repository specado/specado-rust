# Specado

[![CI](https://github.com/specado/specado/actions/workflows/ci.yml/badge.svg)](https://github.com/specado/specado/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Spec-driven LLM integration library with Rust core and Python/Node.js bindings.

## Project Status

🚧 **Pre-Alpha Development** - Sprint 0 in progress

This project is in early development. APIs are unstable and subject to change.

## Overview

Specado is a spec-driven library for integrating Large Language Models (LLMs) into applications. It provides a unified interface for working with different LLM providers while maintaining type safety and performance through its Rust core.

### Key Features

- **Spec-Driven Design**: Define LLM interactions through specifications
- **Multi-Provider Support**: OpenAI, Anthropic, and more (coming soon)
- **Language Bindings**: Native support for Python and Node.js
- **Type Safety**: Rust core ensures memory safety and performance
- **Hot Reload**: Dynamic configuration updates without restarts
- **Built-in Observability**: Audit trails, metrics, and debugging tools

## Quick Start

### Prerequisites

- Rust 1.77+
- Python 3.9+ (for Python bindings)
- Node.js 16+ (for Node.js bindings)

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed setup instructions.

### Installation

#### Python
```bash
pip install specado  # Coming soon
```

#### Node.js
```bash
npm install @specado/core  # Coming soon
```

### Basic Usage

#### Python
```python
import specado

# Simple hello world test
message = specado.hello_world()
print(message)  # "Hello from Specado Core!"

# Check version
version = specado.version()
print(f"Specado version: {version}")
```

#### Node.js
```javascript
const specado = require('@specado/core');

// Simple hello world test
const message = specado.helloWorld();
console.log(message);  // "Hello from Specado Core!"

// Check version
const version = specado.version();
console.log(`Specado version: ${version}`);
```

## Development

### Project Structure

```
specado/
├── specado-core/       # Rust core library
├── specado-python/     # Python bindings (PyO3)
├── specado-node/       # Node.js bindings (N-API)
├── .github/workflows/  # CI/CD pipelines
└── DEVELOPMENT.md      # Development guide
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/specado/specado.git
cd specado

# Build all components
cargo build --all

# Run tests
cargo test --all

# Build Python module
cd specado-python
maturin develop

# Build Node module
cd ../specado-node
npm install && npm run build
```

### Running Tests

```bash
# Rust tests
cargo test --all

# Python tests
cd specado-python
python test_specado.py

# Node.js tests
cd specado-node
npm test
```

## Project Management

- **Project Board**: [Specado MVP Implementation](https://github.com/orgs/specado/projects/6)
- **Issues**: [GitHub Issues](https://github.com/specado/specado/issues)
- **Milestones**: [Sprint Planning](https://github.com/specado/specado/milestones)

### Current Sprint: Sprint 0 - Foundations

Focus on repository setup, CI/CD, and basic FFI validation.

**Key Tasks**:
- [x] Initialize Rust workspace structure (#44)
- [x] Setup GitHub Actions CI pipeline (#45)
- [x] Create development environment setup guide (#46)
- [x] Implement basic "hello world" FFI test (#47)

## Roadmap

### Sprint 1 (Sep 11, 2025)
- Spec schema implementation
- Environment variable management
- OpenAI provider manifest
- Python binding foundation

### Sprint 2 (Sep 25, 2025)
- Transform pipeline
- Hot-reload capability
- Audit implementation

### Sprint 3 (Oct 9, 2025)
- Anthropic provider support
- Router and fallback system
- Circuit breaker implementation

### Sprint 4 (Oct 23, 2025)
- Node.js bindings
- OpenAI compatibility layer
- Packaging and distribution

### Sprint 5 (Nov 6, 2025)
- Performance benchmarks
- Documentation
- Security hardening

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) (coming soon) for details.

### Development Process

1. Check the [Project Board](https://github.com/orgs/specado/projects/6) for available tasks
2. Create a feature branch from `develop`
3. Make your changes with appropriate tests
4. Ensure CI passes (format, lint, test)
5. Submit a PR with a clear description

## License

This project is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Support

- **Documentation**: [docs.specado.com](https://docs.specado.com) (coming soon)
- **Issues**: [GitHub Issues](https://github.com/specado/specado/issues)
- **Discord**: [Join our community](https://discord.gg/specado) (coming soon)

## Acknowledgments

Built with:
- [PyO3](https://pyo3.rs/) - Rust bindings for Python
- [NAPI-RS](https://napi.rs/) - Rust bindings for Node.js
- [Tokio](https://tokio.rs/) - Async runtime for Rust

---

© 2025 Specado Contributors