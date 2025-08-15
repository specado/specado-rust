//! Conversion between Specado protocol and OpenAI format

use crate::protocol::{
    ChatRequest, ChatResponse, ChatStreamChunk, CompletionUsage, ContentPart, FunctionCall,
    Message, MessageContent, MessageDelta, MessageRole, ResponseChoice, ResponseFormat,
    StreamChoice, ToolCall, ToolChoice, ToolDefinition,
};
use super::types::*;

/// Convert Specado ChatRequest to OpenAI format
pub fn to_openai_request(request: &ChatRequest) -> OpenAIRequest {
    OpenAIRequest {
        model: request.model.clone(),
        messages: request.messages.iter().map(to_openai_message).collect(),
        temperature: request.temperature,
        max_tokens: request.max_tokens.map(|t| t as u32),
        top_p: request.top_p,
        n: request.n.map(|n| n as u32),
        stop: request.stop.clone(),
        stream: request.stream,
        stream_options: request.stream_options.as_ref().map(|so| OpenAIStreamOptions {
            include_usage: so.include_usage,
        }),
        frequency_penalty: request.frequency_penalty,
        presence_penalty: request.presence_penalty,
        user: request.user.clone(),
        response_format: request.response_format.as_ref().map(to_openai_response_format),
        seed: request.seed,
        tools: request.tools.as_ref().map(|tools| {
            tools.iter().map(to_openai_tool).collect()
        }),
        tool_choice: request.tool_choice.as_ref().map(to_openai_tool_choice),
    }
}

/// Convert Specado Message to OpenAI format
fn to_openai_message(message: &Message) -> OpenAIMessage {
    OpenAIMessage {
        role: match message.role {
            MessageRole::System => "system".to_string(),
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
            MessageRole::Function => "function".to_string(),
            MessageRole::Tool => "tool".to_string(),
        },
        content: Some(to_openai_content(&message.content)),
        name: message.name.clone(),
        function_call: message.function_call.as_ref().map(|fc| OpenAIFunctionCall {
            name: fc.name.clone(),
            arguments: fc.arguments.clone(),
        }),
        tool_calls: message.tool_calls.as_ref().map(|tcs| {
            tcs.iter().map(|tc| OpenAIToolCall {
                id: tc.id.clone(),
                tool_type: tc.tool_type.clone(),
                function: OpenAIFunctionCall {
                    name: tc.function.name.clone(),
                    arguments: tc.function.arguments.clone(),
                },
            }).collect()
        }),
        tool_call_id: message.tool_call_id.clone(),
    }
}

/// Convert MessageContent to OpenAI format
fn to_openai_content(content: &MessageContent) -> OpenAIContent {
    match content {
        MessageContent::Text(text) => OpenAIContent::Text(text.clone()),
        MessageContent::Parts(parts) => {
            let openai_parts: Vec<OpenAIContentPart> = parts.iter().filter_map(|part| {
                match part {
                    ContentPart::Text { text } => {
                        Some(OpenAIContentPart::Text { text: text.clone() })
                    }
                    ContentPart::Image { url, base64 } => {
                        let image_url = if let Some(url) = url {
                            url.clone()
                        } else if let Some(base64) = base64 {
                            format!("data:image/jpeg;base64,{}", base64)
                        } else {
                            return None;
                        };
                        
                        Some(OpenAIContentPart::ImageUrl {
                            image_url: OpenAIImageUrl {
                                url: image_url,
                                detail: None,
                            },
                        })
                    }
                    ContentPart::Audio { .. } => {
                        // OpenAI doesn't support audio in chat yet
                        None
                    }
                }
            }).collect();
            
            OpenAIContent::Parts(openai_parts)
        }
    }
}

/// Convert ResponseFormat to OpenAI format
fn to_openai_response_format(format: &ResponseFormat) -> OpenAIResponseFormat {
    match format {
        ResponseFormat::Text => OpenAIResponseFormat {
            format_type: "text".to_string(),
            json_schema: None,
        },
        ResponseFormat::JsonObject => OpenAIResponseFormat {
            format_type: "json_object".to_string(),
            json_schema: None,
        },
        ResponseFormat::JsonSchema { schema } => OpenAIResponseFormat {
            format_type: "json_schema".to_string(),
            json_schema: Some(schema.clone()),
        },
    }
}

/// Convert ToolDefinition to OpenAI format
fn to_openai_tool(tool: &ToolDefinition) -> OpenAITool {
    OpenAITool {
        tool_type: tool.tool_type.clone(),
        function: OpenAIFunction {
            name: tool.function.name.clone(),
            description: tool.function.description.clone(),
            parameters: tool.function.parameters.clone(),
        },
    }
}

/// Convert ToolChoice to OpenAI format
fn to_openai_tool_choice(choice: &ToolChoice) -> serde_json::Value {
    match choice {
        ToolChoice::Mode(mode) => serde_json::json!(mode),
        ToolChoice::Function { function, .. } => {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": function.name
                }
            })
        }
    }
}

/// Convert OpenAI response to Specado format
pub fn from_openai_response(response: OpenAIResponse) -> ChatResponse {
    ChatResponse {
        id: response.id,
        object: response.object,
        created: response.created,
        model: response.model,
        choices: response.choices.into_iter().map(from_openai_choice).collect(),
        usage: response.usage.map(from_openai_usage),
        system_fingerprint: response.system_fingerprint,
        metadata: Default::default(),
    }
}

/// Convert OpenAI choice to Specado format
fn from_openai_choice(choice: OpenAIChoice) -> ResponseChoice {
    ResponseChoice {
        index: choice.index,
        message: from_openai_message(choice.message),
        finish_reason: choice.finish_reason,
        logprobs: choice.logprobs,
    }
}

/// Convert OpenAI message to Specado format
fn from_openai_message(message: OpenAIMessage) -> Message {
    let role = match message.role.as_str() {
        "system" => MessageRole::System,
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "function" => MessageRole::Function,
        "tool" => MessageRole::Tool,
        _ => MessageRole::Assistant, // Default fallback
    };
    
    let content = message.content.map(from_openai_content).unwrap_or_else(|| {
        MessageContent::Text(String::new())
    });
    
    Message {
        role,
        content,
        name: message.name,
        function_call: message.function_call.map(|fc| FunctionCall {
            name: fc.name,
            arguments: fc.arguments,
        }),
        tool_calls: message.tool_calls.map(|tcs| {
            tcs.into_iter().map(|tc| ToolCall {
                id: tc.id,
                tool_type: tc.tool_type,
                function: FunctionCall {
                    name: tc.function.name,
                    arguments: tc.function.arguments,
                },
            }).collect()
        }),
        tool_call_id: message.tool_call_id,
        metadata: Default::default(),
    }
}

/// Convert OpenAI content to Specado format
fn from_openai_content(content: OpenAIContent) -> MessageContent {
    match content {
        OpenAIContent::Text(text) => MessageContent::Text(text),
        OpenAIContent::Parts(parts) => {
            let content_parts: Vec<ContentPart> = parts.into_iter().map(|part| {
                match part {
                    OpenAIContentPart::Text { text } => {
                        ContentPart::Text { text }
                    }
                    OpenAIContentPart::ImageUrl { image_url } => {
                        // Check if it's a data URL or regular URL
                        if image_url.url.starts_with("data:") {
                            // Extract base64 from data URL
                            if let Some(base64_start) = image_url.url.find("base64,") {
                                let base64 = image_url.url[base64_start + 7..].to_string();
                                ContentPart::Image {
                                    url: None,
                                    base64: Some(base64),
                                }
                            } else {
                                ContentPart::Image {
                                    url: Some(image_url.url),
                                    base64: None,
                                }
                            }
                        } else {
                            ContentPart::Image {
                                url: Some(image_url.url),
                                base64: None,
                            }
                        }
                    }
                }
            }).collect();
            
            MessageContent::Parts(content_parts)
        }
    }
}

/// Convert OpenAI usage to Specado format
fn from_openai_usage(usage: OpenAIUsage) -> CompletionUsage {
    CompletionUsage {
        prompt_tokens: usage.prompt_tokens,
        completion_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
    }
}

/// Convert OpenAI streaming chunk to Specado format
pub fn from_openai_stream_chunk(chunk: OpenAIStreamChunk) -> ChatStreamChunk {
    ChatStreamChunk {
        id: chunk.id,
        object: chunk.object,
        created: chunk.created,
        model: chunk.model,
        choices: chunk.choices.into_iter().map(from_openai_stream_choice).collect(),
        usage: chunk.usage.map(from_openai_usage),
        system_fingerprint: chunk.system_fingerprint,
    }
}

/// Convert OpenAI stream choice to Specado format
fn from_openai_stream_choice(choice: OpenAIStreamChoice) -> StreamChoice {
    StreamChoice {
        index: choice.index,
        delta: from_openai_delta(choice.delta),
        finish_reason: choice.finish_reason,
        logprobs: choice.logprobs,
    }
}

/// Convert OpenAI delta to Specado format
fn from_openai_delta(delta: OpenAIDelta) -> MessageDelta {
    MessageDelta {
        role: delta.role.map(|r| match r.as_str() {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "function" => MessageRole::Function,
            "tool" => MessageRole::Tool,
            _ => MessageRole::Assistant,
        }),
        content: delta.content,
        function_call: delta.function_call.map(|fc| crate::protocol::FunctionCallDelta {
            name: fc.name,
            arguments: fc.arguments,
        }),
        tool_calls: delta.tool_calls.map(|tcs| {
            tcs.into_iter().map(|tc| crate::protocol::ToolCallDelta {
                index: tc.index,
                id: tc.id,
                tool_type: tc.tool_type,
                function: tc.function.map(|f| crate::protocol::FunctionCallDelta {
                    name: f.name,
                    arguments: f.arguments,
                }),
            }).collect()
        }),
    }
}