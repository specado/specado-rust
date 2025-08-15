# Provider Abstraction & Transformation Engine

## Week 1 Implementation - Core Engine

This module implements the core transformation engine that enables transparent conversion between different LLM provider formats while explicitly tracking lossiness.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User Request                          │
│                    (OpenAI Format - Canonical)               │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  Transformation Engine                       │
│                                                              │
│  1. Analyze source capabilities                             │
│  2. Check target provider capabilities                      │
│  3. Transform incompatible features                         │
│  4. Track lossiness (what was changed/removed)              │
│  5. Add metadata to request                                 │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   Transformed Request                        │
│              (Target Provider Format + Metadata)             │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. Provider Trait (`adapter.rs`)
- Abstract interface that all providers implement
- Defines capabilities (what features each provider supports)
- Hardcoded for MVP, will be manifest-driven in future

### 2. Transformation Engine (`transform.rs`)
- Core logic for transforming between formats
- Tracks lossiness with human-readable reasons
- Handles common incompatibilities:
  - System role merging (Anthropic doesn't support separate system messages)
  - JSON mode removal (not all providers support structured output)
  - Consecutive same-role message merging
  - Parameter removal for unsupported features

### 3. Provider Implementations
- **OpenAI** (`openai.rs`): Reference implementation, supports all features
- **Anthropic** (`anthropic.rs`): Different capabilities, requires transformations

## Lossiness Tracking

The transformation engine explicitly tracks when information is lost:

```rust
pub struct TransformResult {
    pub transformed: ChatRequest,    // The transformed request
    pub lossy: bool,                 // Was information lost?
    pub reasons: Vec<String>,         // Human-readable reasons
    pub metadata: HashMap<String, Value>, // Additional metadata
}
```

### Common Lossiness Reasons
- `system_role_merged`: System messages merged into user messages
- `json_mode_not_supported`: JSON response format removed
- `consecutive_same_role_not_supported`: Multiple same-role messages merged
- `function_calling_not_supported`: Tool/function features removed

## Usage Example

```rust
use specado_core::providers::transform_request;
use specado_core::protocol::types::{ChatRequest, Message};

let request = ChatRequest::new(
    "gpt-4",
    vec![
        Message::system("You are helpful"),
        Message::user("Hello!"),
    ],
);

// Transform for Anthropic
let result = transform_request(request, "anthropic");

if result.lossy {
    println!("Warning: Transformation was lossy!");
    println!("Reasons: {:?}", result.reasons);
}
```

## Future Evolution (Post-MVP)

### From Hardcoded to Manifest-Driven
```rust
// Current (MVP) - Hardcoded
pub struct OpenAIProvider {
    capabilities: ProviderCapabilities { 
        supports_json_mode: true,
        // ... hardcoded
    }
}

// Future - Manifest-driven
pub struct Provider {
    manifest: Manifest, // Loaded from YAML/JSON
    capabilities: ProviderCapabilities, // Parsed from manifest
}
```

### From Simple to Pipeline
```rust
// Current (MVP) - Simple function
fn transform_request(request, provider) -> TransformResult

// Future - Full pipeline
pipeline
    .normalize()    // Standardize format
    .plan()         // Compute transformation plan
    .apply()        // Apply transformations
    .validate()     // Verify invariants
    .postflight()   // Add metadata
```

## Testing

Comprehensive test suite in `tests/transformation_tests.rs`:
- System role transformation
- JSON mode handling
- Consecutive same-role merging
- Multiple lossiness reasons
- Metadata preservation
- Edge cases

Run tests:
```bash
cargo test --package specado-core transformation_tests
```

Run demo:
```bash
cargo run --package specado-core --example week1_demo
```

## Week 1 Deliverables ✅

- [x] Provider trait abstraction
- [x] OpenAI provider implementation
- [x] Anthropic provider implementation  
- [x] Transformation engine with lossiness tracking
- [x] Comprehensive test suite
- [x] Working demo example

## Next Steps (Week 2)

- Add routing layer for fallback support
- Implement Python bindings via PyO3
- Add error handling and retry logic
- Create working Python example