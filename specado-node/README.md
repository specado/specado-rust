# specado

**High-performance Node.js bindings for Specado - Spec-driven LLM integration library with automatic fallback routing and provider transformations**

Specado provides a unified interface for working with multiple LLM providers (OpenAI, Anthropic) with automatic fallback routing, request transformation, and comprehensive error handling, all powered by a high-performance Rust core.

## 🚀 Features

- **🔄 Automatic Fallback Routing**: Seamlessly switch between providers when primary fails
- **🔧 Request Transformation**: Convert between OpenAI and Anthropic formats automatically  
- **📊 Lossiness Tracking**: Monitor when transformations lose information (e.g., system role merging)
- **⚡ High Performance**: Built with Rust core and NAPI-RS for maximum speed
- **🛡️ Production Ready**: Comprehensive error handling, retry policies, and observability
- **🔗 OpenAI Compatible**: Drop-in replacement for OpenAI Node.js client patterns
- **🔧 TypeScript First**: Full TypeScript support with comprehensive type definitions

## 📦 Installation

```bash
npm install specado
# or
yarn add specado
# or
pnpm add specado
```

## 🏃 Quick Start

### Basic Usage

```typescript
import { Client, createMessage } from 'specado';

// Create client with default configuration (OpenAI primary, Anthropic fallback)
const client = new Client();

// Simple chat completion
const response = await client.getChat().getCompletions().create(
  'gpt-4',
  [
    createMessage('system', 'You are a helpful assistant'),
    createMessage('user', 'Hello, world!')
  ]
);

console.log(response.choices[0].message.content);
console.log(`Provider used: ${response.extensions.providerUsed}`);
```

### Custom Provider Configuration

```typescript
import { Client, createMessage } from 'specado';

// Configure specific providers and fallback behavior
const client = new Client({
  primary_provider: 'openai',
  fallback_provider: 'anthropic'
});

const response = await client.getChat().getCompletions().create(
  'gpt-4',
  [createMessage('user', 'Explain quantum computing')],
  0.7,   // temperature
  500    // max_tokens
);

// Access routing metadata
console.log(`Fallback triggered: ${response.extensions.fallbackTriggered}`);
console.log(`Attempts made: ${response.extensions.attempts}`);
console.log(`Provider used: ${response.extensions.providerUsed}`);
```

### Fallback Routing Demo

```typescript
import { Client, createMessage } from 'specado';

const client = new Client({
  primary_provider: 'openai',
  fallback_provider: 'anthropic'
});

// Trigger fallback by using a model that causes timeout
const response = await client.getChat().getCompletions().create(
  'timeout-test-model',  // This will fail on OpenAI
  [createMessage('user', 'This will use fallback')]
);

// Verify fallback occurred
console.assert(response.extensions.fallbackTriggered === true);
console.assert(response.extensions.providerUsed === 'anthropic');
console.assert(response.extensions.attempts === 2);  // Primary failed, fallback succeeded
```

### Transformation and Lossiness Tracking

```typescript
import { Client, createMessage } from 'specado';

// Use Anthropic as primary (system roles are lossy)
const client = new Client({ primary_provider: 'anthropic' });

const response = await client.getChat().getCompletions().create(
  'claude-3',
  [
    createMessage('system', 'You are a coding assistant'),  // Will be merged
    createMessage('user', 'Write a TypeScript function')
  ]
);

// Check transformation metadata
const metadata = response.extensions.metadata;
if (metadata['transformation_lossy'] === 'true') {
  console.log('Transformation was lossy:');
  console.log(`Reasons: ${metadata['lossy_reasons']}`);
}
```

### Error Handling

```typescript
import { Client, createMessage } from 'specado';

const client = new Client();

try {
  const response = await client.getChat().getCompletions().create(
    'auth-error-test-model',  // Triggers auth error
    [createMessage('user', 'This will fail')]
  );
} catch (error) {
  console.error(`Request failed: ${error.message}`);
  // Non-retryable errors (auth, invalid request) are not sent to fallback
}
```

## 🔧 Configuration Options

### Client Configuration

```typescript
const config = {
  primary_provider: 'openai',    // Primary provider to try first
  fallback_provider: 'anthropic' // Fallback when primary fails
};
const client = new Client(config);
```

### Supported Providers

- **OpenAI**: GPT-4, GPT-3.5, and other OpenAI models
- **Anthropic**: Claude-3, Claude-2, and other Claude models

### Message Creation

```typescript
import { createMessage } from 'specado';

// Supported message roles
const systemMsg = createMessage('system', 'You are helpful');
const userMsg = createMessage('user', 'Hello');
const assistantMsg = createMessage('assistant', 'Hi there!');
```

## 📊 Response Format

Specado responses extend the OpenAI format with additional routing metadata:

```typescript
const response = await client.getChat().getCompletions().create(/*...*/);

// Standard OpenAI fields
response.id            // Completion ID
response.model         // Model used
response.choices       // Response choices
response.created       // Timestamp

// Specado extensions
response.extensions.providerUsed       // "openai" | "anthropic"
response.extensions.fallbackTriggered  // boolean
response.extensions.attempts           // Number of provider attempts
response.extensions.metadata          // Record<string, string> with routing details
```

## 🛠️ Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/specado/specado
cd specado/specado-node

# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test
```

### Development Scripts

```bash
# Development build (faster)
npm run build:debug

# Watch mode for development
npm run watch

# Clean build artifacts
npm run clean

# Format code
npm run format

# Lint code
npm run lint

# Verify build and installation
npm run verify
```

### Running Tests

```bash
# Run the comprehensive test suite
npm test

# Run tests in CI mode
npm run test:ci

# Test specific functionality
node test_specado.js
```

## 🚦 Status

**Current Version: 0.1.0 (Week 2 MVP)**

### ✅ Implemented Features
- OpenAI ↔ Anthropic request/response transformation
- Automatic fallback routing with retry policies
- Lossiness tracking for transformations
- High-performance Node.js bindings with NAPI-RS
- Comprehensive error handling and metadata
- Full TypeScript support

### 🚧 Planned Features
- Additional provider support (Cohere, etc.)
- Advanced routing strategies (circuit breakers, health checks)
- Streaming response support
- Performance optimizations and caching

## ⚡ Performance

The Rust core with NAPI-RS bindings provides:

- **Fast Native Performance**: Direct memory management without garbage collection overhead
- **Zero-Copy Operations**: Efficient data transfer between JavaScript and Rust
- **Async/Await Support**: Non-blocking operations with proper Promise integration
- **Multi-Platform**: Prebuilt binaries for macOS (Intel/Apple Silicon), Linux (x64/ARM), and Windows

## 🔧 System Requirements

- **Node.js**: 16.0.0 or higher
- **Operating Systems**: macOS, Linux, Windows
- **Architectures**: x64, ARM64

## 📄 License

Apache-2.0 - see [LICENSE](https://github.com/specado/specado/blob/main/LICENSE) for details.

## 🔗 Links

- **Homepage**: https://github.com/specado/specado
- **Documentation**: https://docs.specado.com
- **Issues**: https://github.com/specado/specado/issues
- **Changelog**: https://github.com/specado/specado/blob/main/specado-node/CHANGELOG.md
- **npm Package**: https://www.npmjs.com/package/specado

## 🤝 Contributing

We welcome contributions! Please see our [contributing guidelines](https://github.com/specado/specado/blob/main/CONTRIBUTING.md) for details.

## 📞 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/specado/specado/issues)
- **Discussions**: [Community discussions](https://github.com/specado/specado/discussions)
- **Email**: hello@specado.com

---

Built with ❤️ using [Rust](https://rust-lang.org/), [NAPI-RS](https://napi.rs/), and [TypeScript](https://typescriptlang.org/)