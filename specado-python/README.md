# Specado Python

**Spec-driven LLM integration library with automatic fallback routing and provider transformations**

Specado provides a unified interface for working with multiple LLM providers (OpenAI, Anthropic) with automatic fallback routing, request transformation, and comprehensive error handling.

## 🚀 Features

- **🔄 Automatic Fallback Routing**: Seamlessly switch between providers when primary fails
- **🔧 Request Transformation**: Convert between OpenAI and Anthropic formats automatically  
- **📊 Lossiness Tracking**: Monitor when transformations lose information (e.g., system role merging)
- **⚡ Built with Rust**: High-performance core with Python bindings via PyO3
- **🛡️ Production Ready**: Comprehensive error handling, retry policies, and observability
- **🔗 OpenAI Compatible**: Drop-in replacement for OpenAI Python client patterns

## 📦 Installation

```bash
pip install specado
```

## 🏃 Quick Start

### Basic Usage

```python
from specado import Client, Message

# Create client with default configuration (OpenAI primary, Anthropic fallback)
client = Client()

# Simple chat completion
response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        Message("system", "You are a helpful assistant"),
        Message("user", "Hello, world!")
    ]
)

print(response.choices[0].message.content)
print(f"Provider used: {response.extensions.provider_used}")
```

### Custom Provider Configuration

```python
from specado import Client, Message

# Configure specific providers and fallback behavior
client = Client({
    "primary_provider": "openai",
    "fallback_provider": "anthropic"
})

response = client.chat.completions.create(
    model="gpt-4",
    messages=[Message("user", "Explain quantum computing")],
    temperature=0.7,
    max_tokens=500
)

# Access routing metadata
print(f"Fallback triggered: {response.extensions.fallback_triggered}")
print(f"Attempts made: {response.extensions.attempts}")
print(f"Provider used: {response.extensions.provider_used}")
```

### Fallback Routing Demo

```python
from specado import Client, Message

client = Client({
    "primary_provider": "openai",
    "fallback_provider": "anthropic"
})

# Trigger fallback by using a model that causes timeout
response = client.chat.completions.create(
    model="timeout-test-model",  # This will fail on OpenAI
    messages=[Message("user", "This will use fallback")]
)

# Verify fallback occurred
assert response.extensions.fallback_triggered == True
assert response.extensions.provider_used == "anthropic"
assert response.extensions.attempts == 2  # Primary failed, fallback succeeded
```

### Transformation and Lossiness Tracking

```python
from specado import Client, Message

# Use Anthropic as primary (system roles are lossy)
client = Client({"primary_provider": "anthropic"})

response = client.chat.completions.create(
    model="claude-3",
    messages=[
        Message("system", "You are a coding assistant"),  # Will be merged
        Message("user", "Write a Python function")
    ]
)

# Check transformation metadata
metadata = response.extensions.metadata
if metadata.get("transformation_lossy"):
    print("Transformation was lossy:")
    print(f"Reasons: {metadata.get('lossy_reasons')}")
```

### Error Handling

```python
from specado import Client, Message

client = Client()

try:
    response = client.chat.completions.create(
        model="auth-error-test-model",  # Triggers auth error
        messages=[Message("user", "This will fail")]
    )
except RuntimeError as e:
    print(f"Request failed: {e}")
    # Non-retryable errors (auth, invalid request) are not sent to fallback
```

## 🔧 Configuration Options

### Client Configuration

```python
config = {
    "primary_provider": "openai",    # Primary provider to try first
    "fallback_provider": "anthropic" # Fallback when primary fails
}
client = Client(config)
```

### Supported Providers

- **OpenAI**: GPT-4, GPT-3.5, and other OpenAI models
- **Anthropic**: Claude-3, Claude-2, and other Claude models

### Message Types

```python
from specado import Message

# Supported message roles
system_msg = Message("system", "You are helpful")
user_msg = Message("user", "Hello")
assistant_msg = Message("assistant", "Hi there!")
```

## 📊 Response Format

Specado responses extend the OpenAI format with additional routing metadata:

```python
response = client.chat.completions.create(...)

# Standard OpenAI fields
response.id            # Completion ID
response.model         # Model used
response.choices       # Response choices
response.created       # Timestamp

# Specado extensions
response.extensions.provider_used       # "openai" | "anthropic"
response.extensions.fallback_triggered  # bool
response.extensions.attempts           # Number of provider attempts
response.extensions.metadata          # Dict with routing/transformation details
```

## 🛠️ Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/specado/specado
cd specado/specado-python

# Install maturin for building
pip install maturin

# Build and install in development mode
maturin develop

# Run tests
pytest tests/
```

### Running Tests

```bash
# Install test dependencies
pip install pytest

# Run the test suite
pytest tests/ -v

# Run specific test
pytest tests/test_bindings.py::test_fallback_triggered -v
```

## 🚦 Status

**Current Version: 0.1.0 (Week 2 MVP)**

### ✅ Implemented Features
- OpenAI ↔ Anthropic request/response transformation
- Automatic fallback routing with retry policies
- Lossiness tracking for transformations
- Python bindings with OpenAI-compatible API
- Comprehensive error handling and metadata

### 🚧 Planned Features
- Additional provider support (Cohere, etc.)
- Advanced routing strategies (circuit breakers, health checks)
- Streaming response support
- Performance optimizations and caching

## 📄 License

Apache-2.0 - see [LICENSE](https://github.com/specado/specado/blob/main/LICENSE) for details.

## 🔗 Links

- **Homepage**: https://github.com/specado/specado
- **Documentation**: https://docs.specado.com
- **Issues**: https://github.com/specado/specado/issues
- **Changelog**: https://github.com/specado/specado/blob/main/specado-python/CHANGELOG.md

## 🤝 Contributing

We welcome contributions! Please see our [contributing guidelines](https://github.com/specado/specado/blob/main/CONTRIBUTING.md) for details.

---

Built with ❤️ using [Rust](https://rust-lang.org/) and [PyO3](https://pyo3.rs/)