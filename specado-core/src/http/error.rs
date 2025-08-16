//! HTTP error mapping utilities

use crate::providers::routing::ProviderError;
use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

/// Map HTTP status code and response body to a ProviderError
pub fn map_http_error(
    status: StatusCode,
    body: Option<String>,
    request_id: Uuid,
) -> ProviderError {
    // Try to parse error details from response body
    let error_details = body
        .as_ref()
        .and_then(|b| serde_json::from_str::<Value>(b).ok())
        .and_then(|v| extract_error_details(&v));
    
    let error_message = error_details
        .as_ref()
        .map(|d| d.message.clone())
        .or_else(|| body.clone())
        .unwrap_or_else(|| format!("HTTP error {}", status.as_u16()));
    
    // Include request ID in error message
    let message_with_id = format!("{} [request_id: {}]", error_message, request_id);
    
    match status {
        StatusCode::UNAUTHORIZED => ProviderError::AuthenticationError,
        
        StatusCode::TOO_MANY_REQUESTS => {
            // Try to parse Retry-After header value
            let retry_after = error_details
                .and_then(|d| d.retry_after_seconds)
                .map(Duration::from_secs);
            
            ProviderError::RateLimit { retry_after }
        }
        
        StatusCode::BAD_REQUEST => ProviderError::InvalidRequest {
            message: message_with_id,
        },
        
        StatusCode::NOT_FOUND => ProviderError::ModelNotAvailable {
            model: extract_model_from_error(&error_message).unwrap_or_else(|| "unknown".to_string()),
        },
        
        StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => ProviderError::Timeout,
        
        status if status.is_server_error() => ProviderError::ServerError {
            status_code: status.as_u16(),
            message: message_with_id,
        },
        
        status if status.is_client_error() => ProviderError::InvalidRequest {
            message: message_with_id,
        },
        
        _ => ProviderError::Custom {
            code: format!("HTTP_{}", status.as_u16()),
            message: message_with_id,
        },
    }
}

/// Error details extracted from response body
struct ErrorDetails {
    message: String,
    retry_after_seconds: Option<u64>,
}

/// Extract error details from JSON response
fn extract_error_details(json: &Value) -> Option<ErrorDetails> {
    // Try common error response formats
    
    // OpenAI format: { "error": { "message": "...", "type": "...", "code": "..." } }
    if let Some(error) = json.get("error") {
        if let Some(message) = error.get("message").and_then(|v| v.as_str()) {
            return Some(ErrorDetails {
                message: message.to_string(),
                retry_after_seconds: error
                    .get("retry_after")
                    .and_then(|v| v.as_u64()),
            });
        }
    }
    
    // Anthropic format: { "error": { "type": "...", "message": "..." } }
    // (Similar to OpenAI)
    
    // Generic format: { "message": "...", "error": "..." }
    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        return Some(ErrorDetails {
            message: message.to_string(),
            retry_after_seconds: json
                .get("retry_after")
                .and_then(|v| v.as_u64()),
        });
    }
    
    if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
        return Some(ErrorDetails {
            message: error.to_string(),
            retry_after_seconds: None,
        });
    }
    
    None
}

/// Try to extract model name from error message
fn extract_model_from_error(message: &str) -> Option<String> {
    // Look for common patterns like "model 'gpt-4' not found"
    if let Some(start) = message.find("model '") {
        let start = start + 7;
        if let Some(end) = message[start..].find('\'') {
            return Some(message[start..start + end].to_string());
        }
    }
    
    if let Some(start) = message.find("model \"") {
        let start = start + 7;
        if let Some(end) = message[start..].find('"') {
            return Some(message[start..start + end].to_string());
        }
    }
    
    None
}

/// Parse Retry-After header value
pub fn parse_retry_after(header_value: &str) -> Option<Duration> {
    // Try to parse as seconds (integer)
    if let Ok(seconds) = header_value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    
    // Could also be an HTTP date, but for MVP we'll just handle seconds
    None
}