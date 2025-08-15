# Protocol Module

The protocol module defines the canonical request/response structures for LLM interactions in Specado. These types serve as the core data model for all provider integrations and are designed to be forward-compatible with streaming and other advanced features.

## Architecture Principles

### 1. Provider Agnostic
The protocol structures are designed to work with any LLM provider (OpenAI, Anthropic, Google, etc.) while allowing provider-specific extensions through metadata fields.

### 2. Forward Compatible
The design anticipates future features:
- **Streaming Support**: `ResponseType` enum handles both complete and streaming responses
- **Multimodal Content**: `MessageContent` supports text and structured parts (images, audio)
- **Function Calling**: Full support for tools and function definitions
- **Extensibility**: Metadata fields allow provider-specific parameters

### 3. Type Safety
Strong typing with Rust enums and structs ensures compile-time safety and clear API contracts.

## Core Types

### Message
The fundamental unit of conversation:
```rust
use specado_core::protocol::Message;

// Simple message construction
let system_msg = Message::system("You are a helpful assistant");
let user_msg = Message::user("What is the weather?");
let assistant_msg = Message::assistant("I'll help you check the weather.");

// Function message
let function_msg = Message::function("get_weather", r#"{"temp": 72, "condition": "sunny"}"#);

// Tool message
let tool_msg = Message::tool("call_123", "Weather retrieved successfully");
```

### MessageBuilder
For complex message construction:
```rust
use specado_core::protocol::{MessageBuilder, MessageRole, ContentPart};

// Text message with metadata
let message = MessageBuilder::new(MessageRole::Assistant, "Processing your request")
    .with_name("assistant_1")
    .with_function_call("calculate", r#"{"operation": "sum", "values": [1, 2, 3]}"#)
    .with_metadata("provider", serde_json::json!("openai"))
    .build();

// Multimodal message with parts
let multimodal = MessageBuilder::with_parts(MessageRole::User, vec![
    ContentPart::Text { text: "Analyze this chart:".to_string() },
    ContentPart::Image { 
        url: Some("https://example.com/chart.png".to_string()),
        base64: None,
    },
])
.with_name("user_123")
.build();
```

### ChatRequest
Request structure for LLM completions:
```rust
use specado_core::protocol::{ChatRequest, Message};

let request = ChatRequest::new("gpt-4", vec![
    Message::system("You are a helpful assistant"),
    Message::user("Explain quantum computing"),
])
.with_temperature(0.7)
.with_max_tokens(1000)
.with_top_p(0.9)  // Nucleus sampling
.with_stop_sequence("END")  // Stop generation at "END"
.with_streaming(true);  // Enable streaming with usage tracking
```

### Advanced Request Features
```rust
use specado_core::protocol::{
    ChatRequest, Message, ToolDefinition, FunctionDefinition,
    ToolChoice, ResponseFormat
};

let mut request = ChatRequest::new("gpt-4", messages);

// Add tool definitions for function calling
request.tools = Some(vec![
    ToolDefinition {
        tool_type: "function".to_string(),
        function: FunctionDefinition {
            name: "search_web".to_string(),
            description: Some("Search the web for information".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["query"]
            })),
        },
    }
]);

// Control tool usage
request.tool_choice = Some(ToolChoice::Mode("auto".to_string()));

// Request JSON output
request.response_format = Some(ResponseFormat::JsonObject);

// Set deterministic seed
request.seed = Some(42);
```

### ChatResponse
Complete response from LLM:
```rust
use specado_core::protocol::{ChatResponse, ResponseChoice, CompletionUsage};

let response = ChatResponse {
    id: "chatcmpl-123".to_string(),
    object: "chat.completion".to_string(),
    created: 1234567890,
    model: "gpt-4".to_string(),
    choices: vec![
        ResponseChoice {
            index: 0,
            message: Message::assistant("Here's the answer..."),
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        }
    ],
    usage: Some(CompletionUsage {
        prompt_tokens: 50,
        completion_tokens: 100,
        total_tokens: 150,
    }),
    system_fingerprint: Some("fp_abc123".to_string()),
    metadata: Default::default(),
};
```

## Streaming Support

The protocol is designed for efficient streaming:

### Stream Chunks
```rust
use specado_core::protocol::{ChatStreamChunk, StreamChoice, MessageDelta};

let chunk = ChatStreamChunk {
    id: "chatcmpl-123".to_string(),
    object: "chat.completion.chunk".to_string(),
    created: 1234567890,
    model: "gpt-4".to_string(),
    choices: vec![
        StreamChoice {
            index: 0,
            delta: MessageDelta {
                role: Some(MessageRole::Assistant),  // First chunk
                content: Some("The answer ".to_string()),
                function_call: None,
                tool_calls: None,
            },
            finish_reason: None,
            logprobs: None,
        }
    ],
    usage: None,  // Usage in final chunk if requested
    system_fingerprint: None,
};
```

### Accumulating Stream Responses
```rust
use specado_core::protocol::{MessageDelta, MessageRole};

// Accumulator for streaming responses
struct StreamAccumulator {
    role: Option<MessageRole>,
    content: String,
}

impl StreamAccumulator {
    fn new() -> Self {
        Self {
            role: None,
            content: String::new(),
        }
    }
    
    fn add_delta(&mut self, delta: &MessageDelta) {
        if let Some(role) = delta.role {
            self.role = Some(role);
        }
        if let Some(content) = &delta.content {
            self.content.push_str(content);
        }
    }
    
    fn to_message(self) -> Message {
        Message {
            role: self.role.unwrap_or(MessageRole::Assistant),
            content: MessageContent::Text(self.content),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: Default::default(),
        }
    }
}
```

### Response Type Handling
```rust
use specado_core::protocol::{ResponseType, ChatResponse, ChatStreamChunk};

fn handle_response(response: ResponseType) {
    match response {
        ResponseType::Complete(chat_response) => {
            // Handle complete response
            println!("Complete response: {}", chat_response.id);
            for choice in &chat_response.choices {
                println!("Message: {:?}", choice.message.content);
            }
        }
        ResponseType::Stream(chunk) => {
            // Handle streaming chunk
            for choice in &chunk.choices {
                if let Some(content) = &choice.delta.content {
                    print!("{}", content);  // Stream to output
                }
            }
        }
    }
}
```

## Multimodal Support

The protocol supports multimodal content:

```rust
use specado_core::protocol::{Message, MessageContent, ContentPart};

let multimodal_message = Message {
    role: MessageRole::User,
    content: MessageContent::Parts(vec![
        ContentPart::Text {
            text: "What's in this image?".to_string(),
        },
        ContentPart::Image {
            url: Some("https://example.com/image.jpg".to_string()),
            base64: None,
        },
    ]),
    name: None,
    function_call: None,
    tool_calls: None,
    tool_call_id: None,
    metadata: Default::default(),
};
```

## Provider-Specific Extensions

Use metadata fields for provider-specific parameters:

```rust
let mut request = ChatRequest::new("claude-3", messages);

// Add Anthropic-specific parameters
request.metadata.insert(
    "anthropic_version".to_string(),
    serde_json::json!("2023-01-01")
);
request.metadata.insert(
    "max_tokens_to_sample".to_string(),
    serde_json::json!(1000)
);

// Add OpenAI-specific parameters
request.metadata.insert(
    "logit_bias".to_string(),
    serde_json::json!({
        "50256": -100  // Prevent token 50256
    })
);
```

## Serialization

All protocol types implement `Serialize` and `Deserialize`:

```rust
use specado_core::protocol::{ChatRequest, Message};

let request = ChatRequest::new("gpt-4", vec![
    Message::user("Hello!")
]);

// Serialize to JSON
let json = serde_json::to_string(&request)?;

// Deserialize from JSON
let parsed: ChatRequest = serde_json::from_str(&json)?;
```

## Best Practices

### 1. Use Builder Pattern for Complex Messages
When creating messages with multiple fields, use the `MessageBuilder` for clarity.

### 2. Handle Both Response Types
Always design your code to handle both `Complete` and `Stream` response variants.

### 3. Validate Token Limits
Check model-specific token limits before sending requests:
```rust
fn validate_request(request: &ChatRequest, max_context: usize) -> Result<(), String> {
    // Estimate tokens (rough approximation)
    let estimated_tokens: usize = request.messages.iter()
        .filter_map(|m| m.content.as_text())
        .map(|text| text.len() / 4)  // Rough token estimate
        .sum();
    
    if estimated_tokens > max_context {
        return Err(format!("Estimated {} tokens exceeds limit of {}", 
                          estimated_tokens, max_context));
    }
    Ok(())
}
```

### 4. Use Metadata for Tracking
Add request IDs and other tracking information:
```rust
request.metadata.insert(
    "request_id".to_string(),
    serde_json::json!(uuid::Uuid::new_v4().to_string())
);
request.metadata.insert(
    "client_version".to_string(),
    serde_json::json!(env!("CARGO_PKG_VERSION"))
);
```

### 5. Prepare for Streaming
Design your application to handle streaming from the start:
```rust
enum ProcessingMode {
    Blocking,
    Streaming,
}

async fn process_request(
    request: ChatRequest,
    mode: ProcessingMode,
) -> Result<String, Error> {
    match mode {
        ProcessingMode::Blocking => {
            // Get complete response
            let response = send_request(request).await?;
            Ok(extract_content(response))
        }
        ProcessingMode::Streaming => {
            // Stream response
            let mut stream = send_streaming_request(request).await?;
            let mut content = String::new();
            while let Some(chunk) = stream.next().await {
                content.push_str(&process_chunk(chunk?));
            }
            Ok(content)
        }
    }
}
```

## Future Compatibility

The protocol is designed to accommodate future LLM capabilities:

1. **Vision Models**: Already supported through `ContentPart::Image`
2. **Audio Models**: Pre-defined with `ContentPart::Audio`
3. **Function Calling**: Full support for tools and functions
4. **Structured Output**: `ResponseFormat` enum for JSON and schemas
5. **Embeddings**: Can be added as new request/response types
6. **Fine-tuning**: Metadata fields can carry training parameters
7. **Batch Processing**: Can extend with batch request types

## Testing

The protocol module includes comprehensive tests:

```bash
# Run protocol tests
cargo test protocol_tests

# Run with output
cargo test protocol_tests -- --nocapture

# Run specific test
cargo test test_streaming_delta_accumulation
```

## Migration Guide

When updating from direct provider APIs:

### From OpenAI
```rust
// OpenAI format
let openai_request = json!({
    "model": "gpt-4",
    "messages": [...],
    "temperature": 0.7
});

// Specado Protocol
let specado_request = ChatRequest::new("gpt-4", messages)
    .with_temperature(0.7);
```

### From Anthropic
```rust
// Anthropic format
let anthropic_request = json!({
    "model": "claude-3",
    "prompt": "\n\nHuman: Hello\n\nAssistant:",
    "max_tokens_to_sample": 1000
});

// Specado Protocol
let specado_request = ChatRequest::new("claude-3", vec![
    Message::user("Hello")
])
.with_max_tokens(1000);
```

## Performance Considerations

1. **Message Reuse**: Messages are `Clone`, reuse them across requests
2. **Metadata Efficiency**: Only add metadata when necessary
3. **Streaming**: Use streaming for long responses to reduce latency
4. **Serialization**: Consider using `serde_json::to_vec` for better performance

## Security

1. **Never log sensitive content**: Messages may contain PII
2. **Validate inputs**: Check message content before sending
3. **Sanitize metadata**: Ensure metadata doesn't leak sensitive info
4. **Use SecretString**: For API keys in provider configuration