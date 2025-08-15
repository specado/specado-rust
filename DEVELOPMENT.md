# Development Guide

This guide covers the development environment setup for the Specado project, including requirements for the Rust toolchain, Python bindings, Node.js bindings, and recommended IDE configurations.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Rust Development Setup](#rust-development-setup)
- [Python Development Setup](#python-development-setup)
- [Node.js Development Setup](#nodejs-development-setup)
- [IDE Setup](#ide-setup)
- [Building and Testing](#building-and-testing)
- [CI/CD Pipeline](#cicd-pipeline)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Operating System Support

- **macOS**: 11.0+ (Big Sur or later)
- **Linux**: Ubuntu 20.04+, Debian 11+, Fedora 35+, or compatible
- **Windows**: Windows 10/11 with WSL2 (native Windows support coming soon)

### Required Tools

- Git 2.25+
- Make (optional but recommended)
- C compiler (gcc, clang, or MSVC)

## Rust Development Setup

### Install Rust Toolchain

1. Install Rust using rustup (recommended):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Configure your shell:
   ```bash
   source $HOME/.cargo/env
   ```

3. Verify installation:
   ```bash
   rustc --version  # Should be 1.77.0 or later
   cargo --version
   ```

### Required Rust Version

- **Minimum**: 1.77.0
- **Recommended**: Latest stable

### Rust Components

Install additional components:
```bash
rustup component add rustfmt clippy
rustup component add rust-src  # For better IDE support
```

### Cargo Tools

Install useful development tools:
```bash
cargo install cargo-watch    # Auto-rebuild on file changes
cargo install cargo-audit    # Security vulnerability scanning
cargo install cargo-expand   # Macro expansion viewer
cargo install cargo-edit     # Add/upgrade dependencies from CLI
```

## Python Development Setup

### Python Version Requirements

- **Minimum**: Python 3.9
- **Recommended**: Python 3.11 or 3.12
- **Tested versions**: 3.9, 3.10, 3.11, 3.12

### Install Python

#### macOS
```bash
# Using Homebrew
brew install python@3.11

# Or using pyenv (recommended for multiple versions)
brew install pyenv
pyenv install 3.11.7
pyenv global 3.11.7
```

#### Linux
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install python3.11 python3.11-dev python3-pip

# Fedora
sudo dnf install python3.11 python3.11-devel
```

### Python Development Dependencies

Install maturin for building Python bindings:
```bash
pip install --upgrade pip
pip install maturin[patchelf]  # patchelf needed on Linux
```

Install development tools:
```bash
pip install pytest pytest-cov black ruff mypy
```

### Virtual Environment (Recommended)

Create a virtual environment for the project:
```bash
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements-dev.txt  # If available
```

## Node.js Development Setup

### Node.js Version Requirements

- **Minimum**: Node.js 16.x
- **Recommended**: Node.js 20.x (LTS)
- **Tested versions**: 16.x, 18.x, 20.x

### Install Node.js

#### Using Node Version Manager (nvm) - Recommended

macOS/Linux:
```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install Node.js
nvm install 20
nvm use 20
nvm alias default 20
```

Windows (using nvm-windows):
```powershell
# Download from: https://github.com/coreybutler/nvm-windows
nvm install 20.11.0
nvm use 20.11.0
```

#### Direct Installation

Download from [nodejs.org](https://nodejs.org/) or use package managers:

macOS:
```bash
brew install node@20
```

Linux:
```bash
# Using NodeSource repository
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### Node.js Development Dependencies

Install NAPI-RS CLI globally:
```bash
npm install -g @napi-rs/cli
```

Install project dependencies:
```bash
cd specado-node
npm install
```

## IDE Setup

### Visual Studio Code

#### Required Extensions

1. **rust-analyzer**: Rust language support
   - Extension ID: `rust-lang.rust-analyzer`
   
2. **CodeLLDB**: Rust debugging
   - Extension ID: `vadimcn.vscode-lldb`

3. **Python**: Python language support
   - Extension ID: `ms-python.python`

4. **Pylance**: Python language server
   - Extension ID: `ms-python.vscode-pylance`

5. **ESLint**: JavaScript/TypeScript linting
   - Extension ID: `dbaeumer.vscode-eslint`

#### Recommended Extensions

- **Even Better TOML**: TOML syntax highlighting
- **crates**: Cargo.toml dependency management
- **Error Lens**: Inline error display
- **GitLens**: Enhanced Git integration

#### VS Code Settings

Create `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.procMacro.enable": true,
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[python]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "ms-python.black-formatter"
  },
  "[javascript]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescript]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "python.linting.enabled": true,
  "python.linting.ruffEnabled": true,
  "python.testing.pytestEnabled": true,
  "files.watcherExclude": {
    "**/target/**": true,
    "**/node_modules/**": true,
    "**/__pycache__/**": true
  }
}
```

### IntelliJ IDEA / CLion

1. Install the Rust plugin
2. Install the Python plugin (if using IntelliJ IDEA Ultimate)
3. Configure Rust toolchain in Settings → Languages & Frameworks → Rust
4. Enable cargo features in Rust settings

### Vim/Neovim

For Neovim users, install these plugins:
```vim
" Using vim-plug
Plug 'rust-lang/rust.vim'
Plug 'neovim/nvim-lspconfig'
Plug 'simrat39/rust-tools.nvim'
```

Configure rust-analyzer:
```lua
-- In your Neovim config
require('rust-tools').setup({
  server = {
    settings = {
      ["rust-analyzer"] = {
        cargo = { features = "all" },
        checkOnSave = { command = "clippy" },
      }
    }
  }
})
```

## Building and Testing

### Building the Project

#### Build everything:
```bash
cargo build --all
```

#### Build in release mode:
```bash
cargo build --all --release
```

#### Build specific crate:
```bash
cargo build -p specado-core
cargo build -p specado-python
cargo build -p specado-node
```

### Running Tests

#### Run all Rust tests:
```bash
cargo test --all
```

#### Run tests with output:
```bash
cargo test --all -- --nocapture
```

#### Run specific test:
```bash
cargo test test_hello_world
```

### Building Python Module

```bash
cd specado-python
maturin develop  # For development
maturin build --release  # For distribution
```

### Testing Python Module

```bash
cd specado-python
python test_specado.py
# Or using pytest
pytest test_specado.py -v
```

### Building Node.js Module

```bash
cd specado-node
npm run build        # Production build
npm run build:debug  # Debug build
```

### Testing Node.js Module

```bash
cd specado-node
npm test
```

## CI/CD Pipeline

The project uses GitHub Actions for continuous integration. The pipeline runs on every push and pull request.

### CI Jobs

1. **Format Check**: Ensures code is properly formatted with `rustfmt`
2. **Linting**: Runs `clippy` to catch common mistakes
3. **Rust Tests**: Tests core library on Ubuntu and macOS
4. **Python Tests**: Tests Python bindings with Python 3.9-3.12
5. **Node Tests**: Tests Node.js bindings with Node 16, 18, 20
6. **Integration Tests**: Verifies FFI works across all bindings
7. **Security Audit**: Checks for known vulnerabilities

### Running CI Locally

You can simulate CI checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all
cd specado-python && python test_specado.py && cd ..
cd specado-node && npm test && cd ..

# Security audit
cargo audit
```

## Troubleshooting

### Common Issues

#### Rust Installation Issues

**Problem**: `rustc` not found after installation
**Solution**: 
```bash
source $HOME/.cargo/env
# Add to your shell profile (.bashrc, .zshrc, etc.)
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

#### Python Module Build Failures

**Problem**: `error: Microsoft Visual C++ 14.0 is required` (Windows)
**Solution**: Install Visual Studio Build Tools or Visual Studio Community

**Problem**: `error: Python.h not found`
**Solution**: Install Python development headers:
```bash
# Ubuntu/Debian
sudo apt install python3-dev

# macOS (should be included with Python from Homebrew)
brew reinstall python@3.11
```

#### Node.js Module Build Failures

**Problem**: `Error: Cannot find module '@napi-rs/cli'`
**Solution**: 
```bash
npm install -g @napi-rs/cli
```

**Problem**: Build fails with node-gyp errors
**Solution**: Ensure you have build tools installed:
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install build-essential

# Windows
npm install -g windows-build-tools
```

### Getting Help

1. Check the [GitHub Issues](https://github.com/specado/specado/issues)
2. Read the [API Documentation](https://docs.rs/specado)
3. Join our [Discord Server](https://discord.gg/specado) (if available)
4. Email the maintainers: dev@specado.com

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [PyO3 Documentation](https://pyo3.rs/)
- [NAPI-RS Documentation](https://napi.rs/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)