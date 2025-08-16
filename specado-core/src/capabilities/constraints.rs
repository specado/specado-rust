//! Provider constraint definitions

use serde::{Deserialize, Serialize};

/// Provider-specific constraints and limits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraints {
    /// Token limits for the provider
    pub tokens: TokenLimits,
    
    /// Rate limiting constraints
    pub rate_limits: RateLimits,
    
    /// Message/conversation constraints
    pub messages: MessageConstraints,
    
    /// Timeout constraints
    pub timeouts: TimeoutConstraints,
}

/// Token limit constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenLimits {
    /// Maximum context window (input + output)
    pub max_context_window: Option<u32>,
    
    /// Maximum input tokens
    pub max_input_tokens: Option<u32>,
    
    /// Maximum output tokens
    pub max_output_tokens: Option<u32>,
    
    /// Maximum tokens per message
    pub max_tokens_per_message: Option<u32>,
    
    /// Token encoding type (e.g., "cl100k_base", "claude")
    pub encoding: Option<String>,
}

/// Rate limiting constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimits {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    
    /// Requests per hour
    pub requests_per_hour: Option<u32>,
    
    /// Requests per day
    pub requests_per_day: Option<u32>,
    
    /// Tokens per minute
    pub tokens_per_minute: Option<u32>,
    
    /// Tokens per hour
    pub tokens_per_hour: Option<u32>,
    
    /// Tokens per day
    pub tokens_per_day: Option<u32>,
    
    /// Concurrent request limit
    pub max_concurrent_requests: Option<u32>,
}

/// Message and conversation constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageConstraints {
    /// Maximum messages in a conversation
    pub max_messages_per_conversation: Option<u32>,
    
    /// Maximum message length in characters
    pub max_message_length: Option<u32>,
    
    /// Minimum messages required (e.g., some providers need at least 1 user message)
    pub min_messages_required: Option<u32>,
    
    /// Maximum system message length
    pub max_system_message_length: Option<u32>,
    
    /// Support for empty messages
    pub allow_empty_messages: bool,
    
    /// Support for consecutive same-role messages
    pub allow_consecutive_same_role: bool,
}

/// Timeout constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeoutConstraints {
    /// Maximum request timeout in seconds
    pub max_request_timeout_seconds: Option<u32>,
    
    /// Default timeout in seconds
    pub default_timeout_seconds: Option<u32>,
    
    /// Stream timeout in seconds
    pub stream_timeout_seconds: Option<u32>,
    
    /// Connection timeout in seconds
    pub connection_timeout_seconds: Option<u32>,
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            tokens: TokenLimits::default(),
            rate_limits: RateLimits::default(),
            messages: MessageConstraints::default(),
            timeouts: TimeoutConstraints::default(),
        }
    }
}

impl Default for TokenLimits {
    fn default() -> Self {
        Self {
            max_context_window: Some(4096),
            max_input_tokens: None,
            max_output_tokens: Some(4096),
            max_tokens_per_message: None,
            encoding: None,
        }
    }
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_minute: None,
            requests_per_hour: None,
            requests_per_day: None,
            tokens_per_minute: None,
            tokens_per_hour: None,
            tokens_per_day: None,
            max_concurrent_requests: None,
        }
    }
}

impl Default for MessageConstraints {
    fn default() -> Self {
        Self {
            max_messages_per_conversation: None,
            max_message_length: None,
            min_messages_required: Some(1),
            max_system_message_length: None,
            allow_empty_messages: false,
            allow_consecutive_same_role: false,
        }
    }
}

impl Default for TimeoutConstraints {
    fn default() -> Self {
        Self {
            max_request_timeout_seconds: Some(600),  // 10 minutes
            default_timeout_seconds: Some(30),
            stream_timeout_seconds: Some(60),
            connection_timeout_seconds: Some(10),
        }
    }
}

impl Constraints {
    /// Create constraints for OpenAI GPT-4
    pub fn openai_gpt4() -> Self {
        Self {
            tokens: TokenLimits {
                max_context_window: Some(128000),  // GPT-4 Turbo
                max_input_tokens: None,
                max_output_tokens: Some(4096),
                max_tokens_per_message: None,
                encoding: Some("cl100k_base".to_string()),
            },
            rate_limits: RateLimits {
                requests_per_minute: Some(500),
                requests_per_hour: None,
                requests_per_day: Some(10000),
                tokens_per_minute: Some(150000),
                tokens_per_hour: None,
                tokens_per_day: None,
                max_concurrent_requests: Some(50),
            },
            messages: MessageConstraints {
                max_messages_per_conversation: None,
                max_message_length: None,
                min_messages_required: Some(1),
                max_system_message_length: None,
                allow_empty_messages: false,
                allow_consecutive_same_role: false,
            },
            timeouts: TimeoutConstraints::default(),
        }
    }
    
    /// Create constraints for Anthropic Claude
    pub fn anthropic_claude() -> Self {
        Self {
            tokens: TokenLimits {
                max_context_window: Some(200000),  // Claude 3
                max_input_tokens: None,
                max_output_tokens: Some(4096),
                max_tokens_per_message: None,
                encoding: Some("claude".to_string()),
            },
            rate_limits: RateLimits {
                requests_per_minute: Some(50),
                requests_per_hour: None,
                requests_per_day: None,
                tokens_per_minute: Some(100000),
                tokens_per_hour: None,
                tokens_per_day: None,
                max_concurrent_requests: Some(10),
            },
            messages: MessageConstraints {
                max_messages_per_conversation: None,
                max_message_length: None,
                min_messages_required: Some(1),
                max_system_message_length: None,
                allow_empty_messages: false,
                allow_consecutive_same_role: true,  // Claude allows this
            },
            timeouts: TimeoutConstraints::default(),
        }
    }
    
    /// Check if a request would exceed token limits
    pub fn check_token_limits(&self, input_tokens: u32, output_tokens: u32) -> Result<(), String> {
        if let Some(max_context) = self.tokens.max_context_window {
            if input_tokens + output_tokens > max_context {
                return Err(format!(
                    "Total tokens ({}) exceeds context window ({})",
                    input_tokens + output_tokens,
                    max_context
                ));
            }
        }
        
        if let Some(max_input) = self.tokens.max_input_tokens {
            if input_tokens > max_input {
                return Err(format!(
                    "Input tokens ({}) exceeds limit ({})",
                    input_tokens,
                    max_input
                ));
            }
        }
        
        if let Some(max_output) = self.tokens.max_output_tokens {
            if output_tokens > max_output {
                return Err(format!(
                    "Output tokens ({}) exceeds limit ({})",
                    output_tokens,
                    max_output
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_constraints() {
        let constraints = Constraints::default();
        assert_eq!(constraints.tokens.max_context_window, Some(4096));
        assert_eq!(constraints.messages.min_messages_required, Some(1));
    }
    
    #[test]
    fn test_openai_constraints() {
        let constraints = Constraints::openai_gpt4();
        assert_eq!(constraints.tokens.max_context_window, Some(128000));
        assert_eq!(constraints.tokens.encoding, Some("cl100k_base".to_string()));
        assert_eq!(constraints.rate_limits.requests_per_minute, Some(500));
    }
    
    #[test]
    fn test_anthropic_constraints() {
        let constraints = Constraints::anthropic_claude();
        assert_eq!(constraints.tokens.max_context_window, Some(200000));
        assert_eq!(constraints.tokens.encoding, Some("claude".to_string()));
        assert!(constraints.messages.allow_consecutive_same_role);
    }
    
    #[test]
    fn test_check_token_limits() {
        let constraints = Constraints {
            tokens: TokenLimits {
                max_context_window: Some(1000),
                max_input_tokens: Some(800),
                max_output_tokens: Some(200),
                ..Default::default()
            },
            ..Default::default()
        };
        
        // Within limits
        assert!(constraints.check_token_limits(500, 100).is_ok());
        
        // Exceeds context window
        assert!(constraints.check_token_limits(800, 300).is_err());
        
        // Exceeds input limit
        assert!(constraints.check_token_limits(900, 50).is_err());
        
        // Exceeds output limit
        assert!(constraints.check_token_limits(100, 300).is_err());
    }
}