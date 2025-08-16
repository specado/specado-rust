# Specado Python API Documentation

Comprehensive documentation for Specado's Python bindings, providing spec-driven LLM integration with automatic fallback routing and provider transformations.

## Table of Contents

1. [Installation and Setup](#installation-and-setup)
2. [Quick Start](#quick-start)
3. [API Reference](#api-reference)
4. [PyO3 Integration Patterns](#pyo3-integration-patterns)
5. [Error Handling](#error-handling)
6. [Usage Patterns](#usage-patterns)
7. [Development Guide](#development-guide)
8. [Capabilities System](#capabilities-system)
9. [Performance and Best Practices](#performance-and-best-practices)

## Installation and Setup

### Requirements

- **Python**: 3.9, 3.10, 3.11, or 3.12
- **Operating System**: Linux, macOS, or Windows
- **Rust**: 1.77.0+ (for building from source)

### Installation

#### From PyPI (when available)

```bash
pip install specado
```

#### Development Installation

```bash
# Clone the repository
git clone https://github.com/specado/specado.git
cd specado/specado-python

# Create and activate virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install maturin (build tool)
pip install maturin

# Build and install in development mode
maturin develop

# Or build release version
maturin build --release
```

#### Dependencies

Specado has zero runtime dependencies for maximum compatibility:

```toml
# specado-python/pyproject.toml
[project]
requires-python = ">=3.9"
dependencies = []  # Zero runtime dependencies
```

Optional development dependencies:

```bash
pip install specado[dev]  # Includes pytest, black, ruff, mypy
```

## Quick Start

### Basic Usage

```python
import specado

# Create a client with default providers
client = specado.Client()

# Create messages
messages = [
    specado.Message('system', 'You are a helpful assistant'),
    specado.Message('user', 'Hello, world!')
]

# Make a chat completion request
response = client.chat.completions.create(
    model='gpt-4',
    messages=messages,
    temperature=0.7,
    max_tokens=150
)

# Access the response
print(f"Response: {response.choices[0].message.content}")
print(f"Provider used: {response.extensions.provider_used}")
print(f"Fallback triggered: {response.extensions.fallback_triggered}")
```

### Custom Configuration

```python
import specado

# Configure providers
config = {
    'primary_provider': 'openai',
    'fallback_provider': 'anthropic'
}

client = specado.Client(config)

# The client automatically handles:
# - Provider failures with fallback
# - Rate limiting with retry
# - Exponential backoff
# - Error classification
```

## API Reference

### Core Classes

#### `specado.Client`

Main client for LLM interactions with automatic routing and fallback.

```python
class Client:
    def __init__(self, config: Optional[Dict[str, Any]] = None) -> None
```

**Parameters:**
- `config` (Optional[Dict[str, Any]]): Configuration dictionary for providers and routing

**Attributes:**
- `chat`: Chat namespace containing completion APIs

**Methods:**

##### `get_config(key: str) -> Optional[str]`

Get a configuration value by key.

```python
client = specado.Client({'primary_provider': 'openai'})
primary = client.get_config('primary_provider')  # Returns 'openai'
```

##### `config_keys() -> List[str]`

Get all configuration keys.

```python
keys = client.config_keys()  # Returns ['primary_provider', 'fallback_provider', ...]
```

#### `specado.Message`

Represents a chat message with role and content.

```python
class Message:
    def __init__(self, role: str, content: str) -> None
    
    role: str     # Message role: 'system', 'user', or 'assistant'
    content: str  # Message content
```

**Example:**
```python
# Create different message types
system_msg = specado.Message('system', 'You are a helpful assistant')
user_msg = specado.Message('user', 'Hello!')
assistant_msg = specado.Message('assistant', 'Hi there!')
```

#### `specado.ChatCompletionResponse`

Response object containing the completion result and metadata.

```python
class ChatCompletionResponse:
    id: str                    # Unique response identifier
    object: str               # Object type ('chat.completion')
    created: int              # Unix timestamp
    model: str                # Model used for completion
    choices: List[Choice]     # List of completion choices
    extensions: Extensions    # Specado-specific metadata
```

**Example:**
```python
response = client.chat.completions.create(...)
print(f"ID: {response.id}")
print(f"Model: {response.model}")
print(f"Created: {response.created}")
```

#### `specado.Choice`

Individual completion choice within a response.

```python
class Choice:
    index: int                      # Choice index
    message: Message               # Generated message
    finish_reason: Optional[str]   # Completion finish reason
```

#### `specado.Extensions`

Specado-specific metadata about routing and fallback behavior.

```python
class Extensions:
    provider_used: str          # Provider that generated the response
    fallback_triggered: bool    # Whether fallback was used
    attempts: int              # Total attempts made
    metadata: Dict[str, Any]   # Additional routing metadata
```

**Metadata contents:**
```python
metadata = response.extensions.metadata
# Common keys:
# - 'primary_provider': str
# - 'fallback_provider': str  
# - 'fallback_used': bool
# - 'fallback_index': int (if fallback triggered)
# - 'provider_errors': List[str] (if errors occurred)
```

### Chat API

#### `client.chat.completions.create()`

Create a chat completion with automatic routing and fallback.

```python
def create(
    self,
    model: str,
    messages: List[Message],
    temperature: Optional[float] = None,
    max_tokens: Optional[int] = None
) -> ChatCompletionResponse
```

**Parameters:**
- `model` (str): Model identifier (e.g., 'gpt-4', 'claude-3-opus')
- `messages` (List[Message]): Conversation messages
- `temperature` (Optional[float]): Sampling temperature (0.0-1.0)
- `max_tokens` (Optional[int]): Maximum tokens to generate

**Returns:**
- `ChatCompletionResponse`: Complete response with routing metadata

**Example:**
```python
response = client.chat.completions.create(
    model='gpt-4',
    messages=[
        specado.Message('system', 'You are a helpful assistant'),
        specado.Message('user', 'Explain quantum computing')
    ],
    temperature=0.7,
    max_tokens=500
)
```

#### `client.chat.completions.create_async()` 

Async version of chat completion (MVP implementation wraps sync version).

```python
async def create_async(
    self,
    model: str,
    messages: List[Message], 
    temperature: Optional[float] = None,
    max_tokens: Optional[int] = None
) -> ChatCompletionResponse
```

**Note:** Current implementation is a placeholder. Production version will use `pyo3-asyncio` for proper async support.

### Capability Functions

#### `specado.get_openai_manifest() -> Dict[str, Any]`

Get OpenAI provider capability manifest.

```python
manifest = specado.get_openai_manifest()
print(f"Provider: {manifest['info']['name']}")
print(f"Models: {list(manifest['models'].keys())}")
```

#### `specado.get_anthropic_manifest() -> Dict[str, Any]`

Get Anthropic provider capability manifest.

```python
manifest = specado.get_anthropic_manifest()
print(f"Provider: {manifest['info']['name']}")
```

#### `specado.get_model_capabilities(provider: str, model_id: str) -> Optional[Dict[str, Any]]`

Get capabilities for a specific model.

```python
# Get GPT-4 capabilities
caps = specado.get_model_capabilities('openai', 'gpt-4-turbo')
if caps:
    print(f"Function calling: {caps['features']['function_calling']}")
    print(f"Vision: {caps['features']['vision']}")
    print(f"Max context: {caps['constraints']['tokens']['max_context_window']}")

# Get Claude capabilities  
caps = specado.get_model_capabilities('anthropic', 'claude-3-opus')
```

#### `specado.compare_capabilities(source: Dict, target: Dict) -> Dict[str, Any]`

Compare two capability specifications and analyze lossiness.

```python
gpt4_caps = specado.get_model_capabilities('openai', 'gpt-4-turbo')
claude_caps = specado.get_model_capabilities('anthropic', 'claude-3-opus')

comparison = specado.compare_capabilities(gpt4_caps, claude_caps)
print(f"Is lossy: {comparison['lossiness_report']['is_lossy']}")
print(f"Severity: {comparison['lossiness_report']['severity']}")

if comparison['missing_capabilities']:
    print(f"Missing: {comparison['missing_capabilities']}")
```

### Version Information

#### `specado.version() -> str`

Get library version string.

```python
version = specado.version()  # Returns "0.1.0"
```

#### `specado.__version__`

Module version attribute.

```python
print(specado.__version__)  # Same as specado.version()
```

## PyO3 Integration Patterns

Specado uses PyO3 0.23+ for seamless Rust-Python integration with modern patterns.

### Memory Management

PyO3 handles memory management automatically with proper RAII patterns:

```rust
// specado-python/src/lib.rs:242-247
let extensions = Py::new(py, Extensions {
    provider_used: result.provider_used.clone(),
    fallback_triggered: result.used_fallback,
    attempts: result.attempts,
    metadata_internal: result.metadata.clone(),
})?;
```

**Best Practices:**
- Objects are automatically freed when Python references drop
- No manual memory management required
- GIL (Global Interpreter Lock) handled automatically by PyO3

### Type Conversions

#### Rust → Python

```rust
// specado-python/src/lib.rs:266-269
let message = Py::new(py, Message {
    role: "assistant".to_string(),
    content: message_content,
})?;
```

#### Python → Rust

```rust
// specado-python/src/lib.rs:204-218
let core_messages: PyResult<Vec<CoreMessage>> = messages
    .iter()
    .map(|msg| {
        let msg_ref = msg.bind(py);
        let role = msg_ref.getattr("role")?.extract::<String>()?;
        let content = msg_ref.getattr("content")?.extract::<String>()?;
        
        Ok(match role.as_str() {
            "system" => CoreMessage::system(&content),
            "user" => CoreMessage::user(&content),
            "assistant" => CoreMessage::assistant(&content),
            _ => CoreMessage::user(&content), // Default to user
        })
    })
    .collect();
```

### Error Mapping

Structured error mapping from Rust to Python exceptions:

```rust
// specado-python/src/lib.rs:18-36
#[derive(Debug)]
enum SpecadoError {
    ProviderError(String),
    ConfigurationError(String),
    RuntimeError(String),
    MessageFormatError(String),
}

impl From<SpecadoError> for PyErr {
    fn from(err: SpecadoError) -> PyErr {
        match err {
            SpecadoError::ProviderError(msg) => PyRuntimeError::new_err(format!("Provider error: {}", msg)),
            SpecadoError::ConfigurationError(msg) => PyValueError::new_err(format!("Configuration error: {}", msg)),
            SpecadoError::RuntimeError(msg) => PyRuntimeError::new_err(format!("Runtime error: {}", msg)),
            SpecadoError::MessageFormatError(msg) => PyTypeError::new_err(format!("Message format error: {}", msg)),
        }
    }
}
```

### JSON Handling

Seamless JSON conversion between Python and Rust:

```rust
// specado-python/src/capabilities.rs:37-68
fn compare_capabilities(py: Python, source: Bound<'_, PyDict>, target: Bound<'_, PyDict>) -> PyResult<PyObject> {
    // Convert Python dicts to JSON strings
    let json_module = py.import("json")?;
    let dumps = json_module.getattr("dumps")?;
    
    let source_json = dumps.call1((&source,))?;
    let source_str = source_json.extract::<String>()?;
    
    // Parse into Rust capabilities
    let source_cap: Capability = serde_json::from_str(&source_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid source capability: {}", e)
        ))?;
    
    // Process and return as Python dict
    let comparison = source_cap.compare(&target_cap);
    let comparison_json = serde_json::to_string(&comparison)?;
    
    let loads = json_module.getattr("loads")?;
    let result = loads.call1((comparison_json,))?;
    Ok(result.into())
}
```

## Error Handling

### Exception Types

Specado maps Rust errors to appropriate Python exceptions:

- **`RuntimeError`**: Provider failures, routing errors, runtime issues
- **`ValueError`**: Configuration errors, invalid capability data
- **`TypeError`**: Message format errors, type mismatches

### Error Classification

#### Retryable Errors
- Network timeouts
- Rate limiting (429)
- Server errors (5xx)
- Temporary service unavailability

#### Non-Retryable Errors
- Authentication failures (401, 403)
- Invalid requests (400)
- Not found errors (404)
- Malformed data

### Error Handling Patterns

#### Basic Error Handling

```python
import specado

try:
    client = specado.Client()
    response = client.chat.completions.create(
        model='gpt-4',
        messages=[specado.Message('user', 'Hello')]
    )
except RuntimeError as e:
    print(f"Provider error: {e}")
except ValueError as e:
    print(f"Configuration error: {e}")
except TypeError as e:
    print(f"Message format error: {e}")
```

#### Comprehensive Error Handling

```python
import specado
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def safe_completion(messages, max_retries=3):
    """Make a completion with comprehensive error handling."""
    
    for attempt in range(max_retries):
        try:
            client = specado.Client({
                'primary_provider': 'openai',
                'fallback_provider': 'anthropic'
            })
            
            response = client.chat.completions.create(
                model='gpt-4',
                messages=messages,
                temperature=0.7
            )
            
            # Log routing information
            ext = response.extensions
            logger.info(f"Success: provider={ext.provider_used}, "
                       f"fallback={ext.fallback_triggered}, "
                       f"attempts={ext.attempts}")
            
            return response
            
        except RuntimeError as e:
            logger.warning(f"Attempt {attempt + 1} failed: {e}")
            if attempt == max_retries - 1:
                logger.error("All attempts exhausted")
                raise
                
        except (ValueError, TypeError) as e:
            # These are typically non-retryable
            logger.error(f"Non-retryable error: {e}")
            raise
            
    return None
```

#### Async Error Handling

```python
import asyncio
import specado

async def async_completion(messages):
    """Async completion with error handling."""
    try:
        client = specado.Client()
        # Note: create_async is MVP implementation
        response = await client.chat.completions.create_async(
            model='gpt-4',
            messages=messages
        )
        return response
    except Exception as e:
        print(f"Async error: {e}")
        raise
```

## Usage Patterns

### Basic Chat Completion

```python
import specado

# Simple completion
client = specado.Client()
messages = [specado.Message('user', 'Hello')]

response = client.chat.completions.create(
    model='gpt-4',
    messages=messages
)

print(response.choices[0].message.content)
```

### Multi-Turn Conversations

```python
import specado

client = specado.Client()

# Build conversation history
conversation = [
    specado.Message('system', 'You are a helpful assistant'),
    specado.Message('user', 'What is machine learning?'),
]

# First response
response1 = client.chat.completions.create(
    model='gpt-4',
    messages=conversation
)

# Add assistant response to conversation
conversation.append(specado.Message(
    'assistant', 
    response1.choices[0].message.content
))

# Continue conversation
conversation.append(specado.Message('user', 'Can you give me an example?'))

response2 = client.chat.completions.create(
    model='gpt-4',
    messages=conversation,
    temperature=0.7
)
```

### Provider Fallback Testing

```python
import specado

client = specado.Client({
    'primary_provider': 'openai',
    'fallback_provider': 'anthropic'
})

# Test different failure scenarios
test_cases = [
    ('gpt-4', 'Normal request'),
    ('timeout-test-model', 'Triggers timeout'),
    ('rate-limit-test-model', 'Triggers rate limit'), 
    ('server-error-test-model', 'Triggers server error'),
]

for model, description in test_cases:
    print(f"\nTesting: {description}")
    
    try:
        response = client.chat.completions.create(
            model=model,
            messages=[specado.Message('user', 'Test message')]
        )
        
        ext = response.extensions
        print(f"  Provider: {ext.provider_used}")
        print(f"  Fallback: {ext.fallback_triggered}")
        print(f"  Attempts: {ext.attempts}")
        
    except RuntimeError as e:
        print(f"  Error: {e}")
```

### Capability-Aware Routing

```python
import specado

def smart_model_selection(task_type):
    """Select model based on task requirements."""
    
    if task_type == 'vision':
        # Check vision capabilities
        gpt4_caps = specado.get_model_capabilities('openai', 'gpt-4-turbo')
        claude_caps = specado.get_model_capabilities('anthropic', 'claude-3-opus')
        
        if gpt4_caps and gpt4_caps['features']['vision']:
            return 'gpt-4-turbo'
        elif claude_caps and claude_caps['features']['vision']:
            return 'claude-3-opus'
    
    elif task_type == 'function_calling':
        # Check function calling capabilities
        gpt4_caps = specado.get_model_capabilities('openai', 'gpt-4-turbo')
        if gpt4_caps and gpt4_caps['features']['function_calling']:
            return 'gpt-4-turbo'
    
    # Default fallback
    return 'gpt-4'

# Use capability-aware selection
model = smart_model_selection('vision')
client = specado.Client()

response = client.chat.completions.create(
    model=model,
    messages=[specado.Message('user', 'Analyze this image')]
)
```

### Metadata Analysis

```python
import specado
import json

client = specado.Client()

response = client.chat.completions.create(
    model='gpt-4',
    messages=[specado.Message('user', 'Hello')]
)

# Detailed metadata inspection
ext = response.extensions
metadata = ext.metadata

print("Routing Analysis:")
print(f"  Provider used: {ext.provider_used}")
print(f"  Fallback triggered: {ext.fallback_triggered}")
print(f"  Total attempts: {ext.attempts}")

print("\nMetadata:")
for key, value in metadata.items():
    print(f"  {key}: {value}")

# Check for provider errors
if 'provider_errors' in metadata:
    print(f"\nProvider errors encountered: {len(metadata['provider_errors'])}")
    for error in metadata['provider_errors']:
        print(f"  - {error}")
```

### Configuration Management

```python
import specado
import os

# Environment-based configuration
def create_configured_client():
    config = {}
    
    # Read from environment variables
    if os.getenv('SPECADO_PRIMARY_PROVIDER'):
        config['primary_provider'] = os.getenv('SPECADO_PRIMARY_PROVIDER')
    
    if os.getenv('SPECADO_FALLBACK_PROVIDER'):
        config['fallback_provider'] = os.getenv('SPECADO_FALLBACK_PROVIDER')
    
    # Add custom settings
    config.update({
        'max_retries': 3,
        'timeout_seconds': 30,
        'enable_fallback': True
    })
    
    return specado.Client(config)

# Usage
client = create_configured_client()
print(f"Configuration: {client.config_keys()}")
```

## Development Guide

### Building from Source

#### Prerequisites

```bash
# Install Rust (1.77.0+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Python development tools
pip install maturin pytest black ruff mypy
```

#### Build Process

```bash
# Clone repository
git clone https://github.com/specado/specado.git
cd specado/specado-python

# Development build (debug mode)
maturin develop

# Release build  
maturin build --release

# Install built wheel
pip install target/wheels/specado-*.whl
```

### Testing

#### Unit Tests

```bash
# Run Python tests
pytest test_*.py -v

# Run Rust tests
cargo test

# Combined test suite
maturin develop && pytest tests/ -v
```

#### Integration Tests

```bash
# Test with real providers (requires API keys)
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"

python examples/week2_demo.py
```

#### Test Structure

```
specado-python/
├── test_specado.py          # Basic FFI tests
├── test_capabilities.py     # Capability system tests
├── test_pyo3_upgrade.py     # PyO3 upgrade verification
└── tests/
    └── test_bindings.py     # Comprehensive binding tests
```

### Debugging PyO3 Bindings

#### Enable Debug Logging

```python
import logging
logging.basicConfig(level=logging.DEBUG)

# PyO3 errors will show Rust stack traces
import specado
```

#### Common Issues

1. **Module Import Errors**
   ```bash
   # Rebuild after changes
   maturin develop --force
   
   # Check Python path
   python -c "import sys; print(sys.path)"
   ```

2. **Type Conversion Errors**
   ```python
   # Check type compatibility
   isinstance(msg, specado.Message)  # Should be True
   ```

3. **Memory Issues** 
   ```python
   # PyO3 handles memory automatically
   # No manual cleanup required
   del response  # Optional, GC will handle
   ```

### Contributing Guidelines

#### Code Style

```bash
# Format Rust code
cargo fmt

# Format Python code  
black --line-length 100 *.py

# Lint Rust code
cargo clippy -- -D warnings

# Lint Python code
ruff check *.py

# Type checking
mypy test_*.py
```

#### Pull Request Process

1. **Feature Development**
   ```bash
   git checkout -b feature/new-feature
   # Implement feature
   maturin develop
   pytest tests/ -v
   ```

2. **Testing Requirements**
   - All tests must pass
   - New features need tests
   - Maintain test coverage >90%

3. **Documentation**
   - Update this API documentation
   - Add docstrings to new functions
   - Include usage examples

#### Performance Testing

```python
import time
import specado

# Benchmark completion time
client = specado.Client()
messages = [specado.Message('user', 'Short test')]

start = time.time()
response = client.chat.completions.create(
    model='gpt-4',
    messages=messages
)
duration = time.time() - start

print(f"Completion time: {duration:.2f}s")
print(f"Provider: {response.extensions.provider_used}")
```

## Capabilities System

The capabilities system provides detailed provider and model specifications for intelligent routing.

### Provider Manifests

#### OpenAI Manifest

```python
import specado
import json

manifest = specado.get_openai_manifest()

print(f"Provider: {manifest['info']['name']}")
print(f"Version: {manifest['info']['version']}")

# Available models
for model_id, model_info in manifest['models'].items():
    print(f"\nModel: {model_id}")
    print(f"  Name: {model_info['name']}")
    print(f"  Version: {model_info['version']}")
    
    caps = model_info['capabilities']
    print(f"  Features:")
    print(f"    Function calling: {caps['features']['function_calling']}")
    print(f"    Vision: {caps['features']['vision']}")
    print(f"    JSON mode: {caps['features']['json_mode']}")
```

#### Anthropic Manifest

```python
manifest = specado.get_anthropic_manifest()

print(f"Provider: {manifest['info']['name']}")

# Model comparison
for model_id in manifest['models']:
    caps = specado.get_model_capabilities('anthropic', model_id)
    if caps:
        print(f"\n{model_id}:")
        print(f"  Tool use: {caps['features']['tool_use']}")
        print(f"  Max context: {caps['constraints']['tokens']['max_context_window']}")
```

### Model Capabilities

#### Capability Structure

```python
# Example capability object structure
capability = {
    "version": "1.0.0",
    "modalities": {
        "input": ["Text", "Image"],  # Input types supported
        "output": ["Text"],          # Output types supported
        "configs": {
            "image": {"max_size": "20MB", "formats": ["png", "jpg"]},
            "audio": None,
            "video": None,
            "document": None
        }
    },
    "features": {
        "function_calling": True,
        "json_mode": True,
        "streaming": True,
        "tool_use": False,
        "vision": True
    },
    "parameters": {
        "temperature": {
            "supported": True,
            "min": 0.0,
            "max": 2.0,
            "default": 1.0
        },
        "max_tokens": {
            "supported": True,
            "min": 1,
            "max": 4096,
            "default": 256
        }
    },
    "constraints": {
        "tokens": {
            "max_context_window": 128000,
            "max_output_tokens": 4096
        },
        "rate_limits": {
            "requests_per_minute": 500,
            "tokens_per_minute": 80000
        }
    }
}
```

### Capability Comparison

#### Lossiness Analysis

```python
import specado

# Compare different models
gpt4_caps = specado.get_model_capabilities('openai', 'gpt-4-turbo')
claude_caps = specado.get_model_capabilities('anthropic', 'claude-3-opus')

comparison = specado.compare_capabilities(gpt4_caps, claude_caps)

print("Comparison Results:")
print(f"  Lossy migration: {comparison['lossiness_report']['is_lossy']}")
print(f"  Severity: {comparison['lossiness_report']['severity']}")

# Missing capabilities
if comparison['missing_capabilities']:
    print(f"\nMissing in target:")
    for missing in comparison['missing_capabilities']:
        print(f"  - {missing}")

# Recommendations
if comparison['lossiness_report']['recommendations']:
    print(f"\nRecommendations:")
    for rec in comparison['lossiness_report']['recommendations']:
        print(f"  - {rec}")

# Detailed analysis
details = comparison['lossiness_report']['details']
print(f"\nDetailed analysis: {len(details)} issues found")
for detail in details:
    print(f"  - {detail}")
```

#### Custom Capability Definition

```python
# Define custom model capabilities
custom_model = {
    "version": "1.0.0",
    "modalities": {
        "input": ["Text"],
        "output": ["Text"]
    },
    "features": {
        "function_calling": True,
        "json_mode": False,
        "streaming": True,
        "vision": False
    },
    "constraints": {
        "tokens": {
            "max_context_window": 8192,
            "max_output_tokens": 1024
        }
    }
}

# Compare with existing models
gpt4_caps = specado.get_model_capabilities('openai', 'gpt-4-turbo')
comparison = specado.compare_capabilities(custom_model, gpt4_caps)

print(f"Custom model compatibility: {not comparison['lossiness_report']['is_lossy']}")
```

## Performance and Best Practices

### Performance Optimization

#### Connection Reuse

```python
import specado

# Reuse client instances for better performance
class SpecadoManager:
    def __init__(self):
        self.client = specado.Client({
            'primary_provider': 'openai',
            'fallback_provider': 'anthropic'
        })
    
    def complete(self, messages, **kwargs):
        return self.client.chat.completions.create(
            messages=messages,
            **kwargs
        )

# Global instance
manager = SpecadoManager()

# Use throughout application
response = manager.complete([
    specado.Message('user', 'Hello')
], model='gpt-4')
```

#### Batch Processing

```python
import concurrent.futures
import specado

def process_completion(messages):
    client = specado.Client()
    return client.chat.completions.create(
        model='gpt-4',
        messages=messages
    )

# Process multiple requests concurrently
requests = [
    [specado.Message('user', f'Request {i}')]
    for i in range(10)
]

with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
    futures = [executor.submit(process_completion, req) for req in requests]
    
    results = []
    for future in concurrent.futures.as_completed(futures):
        try:
            result = future.result()
            results.append(result)
        except Exception as e:
            print(f"Request failed: {e}")

print(f"Completed {len(results)} requests")
```

### Memory Management

#### Efficient Message Handling

```python
import specado

# Efficient conversation management
class Conversation:
    def __init__(self, system_message=None):
        self.messages = []
        if system_message:
            self.messages.append(specado.Message('system', system_message))
        self.client = specado.Client()
    
    def add_user_message(self, content):
        self.messages.append(specado.Message('user', content))
    
    def get_response(self, model='gpt-4', **kwargs):
        response = self.client.chat.completions.create(
            model=model,
            messages=self.messages,
            **kwargs
        )
        
        # Add assistant response to conversation
        assistant_msg = response.choices[0].message
        self.messages.append(specado.Message('assistant', assistant_msg.content))
        
        return response
    
    def truncate_if_needed(self, max_messages=20):
        """Keep conversation within limits."""
        if len(self.messages) > max_messages:
            # Keep system message + recent messages
            system_msgs = [m for m in self.messages if m.role == 'system']
            recent_msgs = self.messages[-(max_messages-len(system_msgs)):]
            self.messages = system_msgs + recent_msgs

# Usage
conv = Conversation("You are a helpful assistant")
conv.add_user_message("Hello")
response = conv.get_response()
```

### Error Recovery Patterns

#### Exponential Backoff

```python
import time
import random
import specado

def completion_with_backoff(messages, max_retries=3, base_delay=1.0):
    """Completion with exponential backoff on failures."""
    
    for attempt in range(max_retries):
        try:
            client = specado.Client()
            return client.chat.completions.create(
                model='gpt-4',
                messages=messages
            )
            
        except RuntimeError as e:
            if attempt == max_retries - 1:
                raise
            
            # Exponential backoff with jitter
            delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
            print(f"Attempt {attempt + 1} failed, retrying in {delay:.1f}s")
            time.sleep(delay)
    
    return None
```

#### Circuit Breaker Pattern

```python
import time
import specado

class CircuitBreaker:
    def __init__(self, failure_threshold=5, timeout=60):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.failure_count = 0
        self.last_failure_time = None
        self.state = 'CLOSED'  # CLOSED, OPEN, HALF_OPEN
    
    def call(self, func, *args, **kwargs):
        if self.state == 'OPEN':
            if time.time() - self.last_failure_time > self.timeout:
                self.state = 'HALF_OPEN'
            else:
                raise RuntimeError("Circuit breaker is OPEN")
        
        try:
            result = func(*args, **kwargs)
            self.on_success()
            return result
        except Exception as e:
            self.on_failure()
            raise
    
    def on_success(self):
        self.failure_count = 0
        self.state = 'CLOSED'
    
    def on_failure(self):
        self.failure_count += 1
        self.last_failure_time = time.time()
        
        if self.failure_count >= self.failure_threshold:
            self.state = 'OPEN'

# Usage
breaker = CircuitBreaker()
client = specado.Client()

def protected_completion(messages):
    return breaker.call(
        client.chat.completions.create,
        model='gpt-4',
        messages=messages
    )
```

### Best Practices

#### 1. Configuration Management

```python
# Use environment-specific configurations
import os
import specado

def get_production_client():
    return specado.Client({
        'primary_provider': 'openai',
        'fallback_provider': 'anthropic',
        'timeout_seconds': 30,
        'max_retries': 3
    })

def get_development_client():
    return specado.Client({
        'primary_provider': 'openai',
        'fallback_provider': 'openai',  # Same provider for dev
        'timeout_seconds': 10,
        'max_retries': 1
    })

# Environment-based selection
if os.getenv('ENVIRONMENT') == 'production':
    client = get_production_client()
else:
    client = get_development_client()
```

#### 2. Logging and Monitoring

```python
import logging
import specado

# Configure structured logging
logging.basicConfig(
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    level=logging.INFO
)
logger = logging.getLogger('specado')

def monitored_completion(messages, model='gpt-4'):
    """Completion with comprehensive monitoring."""
    client = specado.Client()
    
    start_time = time.time()
    try:
        response = client.chat.completions.create(
            model=model,
            messages=messages
        )
        
        duration = time.time() - start_time
        ext = response.extensions
        
        logger.info(
            f"Completion successful: "
            f"model={model}, "
            f"provider={ext.provider_used}, "
            f"fallback={ext.fallback_triggered}, "
            f"attempts={ext.attempts}, "
            f"duration={duration:.2f}s"
        )
        
        return response
        
    except Exception as e:
        duration = time.time() - start_time
        logger.error(
            f"Completion failed: "
            f"model={model}, "
            f"duration={duration:.2f}s, "
            f"error={str(e)}"
        )
        raise
```

#### 3. Testing Strategies

```python
import unittest
import specado

class TestSpecadoIntegration(unittest.TestCase):
    def setUp(self):
        self.client = specado.Client({
            'primary_provider': 'openai',
            'fallback_provider': 'anthropic'
        })
    
    def test_basic_completion(self):
        """Test basic completion functionality."""
        messages = [specado.Message('user', 'Hello')]
        
        response = self.client.chat.completions.create(
            model='gpt-4',
            messages=messages
        )
        
        self.assertIsNotNone(response)
        self.assertGreater(len(response.choices), 0)
        self.assertIsInstance(response.choices[0].message.content, str)
    
    def test_fallback_behavior(self):
        """Test fallback routing."""
        messages = [specado.Message('user', 'Test fallback')]
        
        response = self.client.chat.completions.create(
            model='timeout-test-model',
            messages=messages
        )
        
        # Should trigger fallback
        self.assertTrue(response.extensions.fallback_triggered)
        self.assertEqual(response.extensions.provider_used, 'anthropic')
    
    def test_capability_discovery(self):
        """Test capability system."""
        caps = specado.get_model_capabilities('openai', 'gpt-4-turbo')
        
        self.assertIsNotNone(caps)
        self.assertIn('features', caps)
        self.assertIn('constraints', caps)

if __name__ == '__main__':
    unittest.main()
```

---

## License

Licensed under the Apache License, Version 2.0. See the [LICENSE](../LICENSE) file for details.

## Support

- **GitHub Issues**: [https://github.com/specado/specado/issues](https://github.com/specado/specado/issues)
- **Documentation**: [https://docs.specado.com](https://docs.specado.com)
- **Repository**: [https://github.com/specado/specado](https://github.com/specado/specado)

For questions specific to Python bindings, please tag issues with `python` label.