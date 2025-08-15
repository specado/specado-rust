//! Rate limiting tracking and management

use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

/// Information about current rate limits
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Requests limit per minute
    pub requests_per_minute: Option<u32>,
    
    /// Tokens limit per minute
    pub tokens_per_minute: Option<u32>,
    
    /// Requests remaining in current window
    pub requests_remaining: Option<u32>,
    
    /// Tokens remaining in current window
    pub tokens_remaining: Option<u32>,
    
    /// When the current window resets
    pub reset_at: Option<DateTime<Utc>>,
    
    /// Requests used in current window
    pub requests_used: u32,
    
    /// Tokens used in current window
    pub tokens_used: u32,
}

impl Default for RateLimitInfo {
    fn default() -> Self {
        Self {
            requests_per_minute: None,
            tokens_per_minute: None,
            requests_remaining: None,
            tokens_remaining: None,
            reset_at: None,
            requests_used: 0,
            tokens_used: 0,
        }
    }
}

/// Tracks rate limit information for a provider
#[derive(Debug, Clone)]
pub struct RateLimitTracker {
    info: Arc<Mutex<RateLimitInfo>>,
}

impl RateLimitTracker {
    /// Create a new rate limit tracker
    pub fn new() -> Self {
        Self {
            info: Arc::new(Mutex::new(RateLimitInfo::default())),
        }
    }
    
    /// Update rate limit information from response headers
    pub fn update_from_headers(&self, headers: &reqwest::header::HeaderMap) {
        let mut info = self.info.lock().unwrap();
        
        // OpenAI-style headers
        if let Some(limit) = headers.get("x-ratelimit-limit-requests") {
            if let Ok(val) = limit.to_str() {
                if let Ok(num) = val.parse::<u32>() {
                    info.requests_per_minute = Some(num);
                }
            }
        }
        
        if let Some(remaining) = headers.get("x-ratelimit-remaining-requests") {
            if let Ok(val) = remaining.to_str() {
                if let Ok(num) = val.parse::<u32>() {
                    info.requests_remaining = Some(num);
                }
            }
        }
        
        if let Some(reset) = headers.get("x-ratelimit-reset-requests") {
            if let Ok(val) = reset.to_str() {
                // Parse various time formats
                if let Ok(timestamp) = val.parse::<i64>() {
                    info.reset_at = Some(DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now));
                }
            }
        }
        
        // Token limits
        if let Some(limit) = headers.get("x-ratelimit-limit-tokens") {
            if let Ok(val) = limit.to_str() {
                if let Ok(num) = val.parse::<u32>() {
                    info.tokens_per_minute = Some(num);
                }
            }
        }
        
        if let Some(remaining) = headers.get("x-ratelimit-remaining-tokens") {
            if let Ok(val) = remaining.to_str() {
                if let Ok(num) = val.parse::<u32>() {
                    info.tokens_remaining = Some(num);
                }
            }
        }
    }
    
    /// Record a request was made
    pub fn record_request(&self, tokens_used: u32) {
        let mut info = self.info.lock().unwrap();
        info.requests_used += 1;
        info.tokens_used += tokens_used;
        
        // Update remaining counts if known
        if let Some(remaining) = info.requests_remaining {
            info.requests_remaining = Some(remaining.saturating_sub(1));
        }
        if let Some(remaining) = info.tokens_remaining {
            info.tokens_remaining = Some(remaining.saturating_sub(tokens_used));
        }
    }
    
    /// Get current rate limit information
    pub fn get_info(&self) -> RateLimitInfo {
        self.info.lock().unwrap().clone()
    }
    
    /// Check if we should wait before making another request
    pub fn should_wait(&self) -> Option<std::time::Duration> {
        let info = self.info.lock().unwrap();
        
        // Check if we're at the limit
        if let Some(remaining) = info.requests_remaining {
            if remaining == 0 {
                if let Some(reset) = info.reset_at {
                    let now = Utc::now();
                    if reset > now {
                        let duration = reset.signed_duration_since(now);
                        return Some(std::time::Duration::from_secs(duration.num_seconds() as u64));
                    }
                }
            }
        }
        
        None
    }
}

impl Default for RateLimitTracker {
    fn default() -> Self {
        Self::new()
    }
}