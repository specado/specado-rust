# Specado Project Context

## Project Overview
Specado is a spec-driven LLM integration library with a Rust core and Python/Node.js bindings. The project provides a unified interface for working with different LLM providers while maintaining type safety and performance.

## Repository Information
- **Organization**: specado
- **Repository**: specado-rust
- **GitHub URL**: https://github.com/specado/specado-rust
- **Website**: https://specado.com
- **Documentation**: https://docs.specado.com (coming soon)
- **License**: Apache-2.0 (single license, not dual-licensed)

## Project Structure
```
specado-rust/
├── specado-core/       # Rust core library with FFI exports
├── specado-python/     # Python bindings using PyO3
├── specado-node/       # Node.js bindings using NAPI-RS
├── .github/workflows/  # CI/CD pipelines
├── DEVELOPMENT.md      # Development setup guide
└── README.md          # Project documentation
```

## Technical Requirements
- **Rust Version**: Minimum 1.77.0 (required for NAPI-RS)
- **Python**: 3.9+ (tested with 3.9, 3.10, 3.11, 3.12)
- **Node.js**: 16+ (tested with 16, 18, 20)
- **PyO3 Version**: 0.23 (updated from 0.22 to fix security vulnerability)

## Sprint 0 Status (Completed)
All Sprint 0 tasks have been completed:
- ✅ Issue #44: Initialize Rust workspace structure
- ✅ Issue #45: Setup GitHub Actions CI pipeline
- ✅ Issue #46: Create development environment setup guide
- ✅ Issue #47: Implement basic "hello world" FFI test

## Key Implementation Details

### FFI Safety
- The `specado_free_string` function is marked as `unsafe` because it dereferences raw pointers
- Proper memory management is implemented for C string allocation/deallocation

### Python Bindings
- Uses PyO3 with `extension-module` feature
- Tests are marked with `#[ignore]` as they require maturin for proper linking
- Includes a separate README.md in the specado-python directory

### Node.js Bindings
- Uses NAPI-RS for bindings
- Async functions are conditionally compiled (`#[cfg(not(test))]`) to avoid test compilation issues
- Requires tokio as a dev dependency for async tests

### CI/CD Pipeline
- Comprehensive GitHub Actions workflow testing on Ubuntu and macOS
- Tests all Python versions (3.9-3.12) and Node versions (16, 18, 20)
- Includes security audit, formatting checks (rustfmt), and linting (clippy)

## Important Notes
- Always use Apache-2.0 license, never MIT
- Domain is specado.com (not specado.io)
- Minimum Rust version is 1.77.0 (not 1.75.0)
- PyO3 version must be 0.23+ to avoid security vulnerabilities

## Project Management
- **Project Board**: https://github.com/orgs/specado/projects/6
- **Milestones**: 6 sprints planned (Sprint 0 completed)
- **Issues**: 47 issues created (11 epics, 32 stories, 4 Sprint 0 tasks)