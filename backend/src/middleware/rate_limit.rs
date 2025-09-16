// T058: Rate limiting middleware
// Reference: research.md section 2 "Limitations Discovered" for rate limiting need

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Rate limit configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window duration
    pub window: Duration,
    /// Burst allowance above max_requests
    pub burst: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            max_requests: 60,  // 60 requests
            window: Duration::from_secs(60), // per minute
            burst: 10, // Allow 10 extra requests in burst
        }
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        let max_requests = std::env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        
        let burst = std::env::var("RATE_LIMIT_BURST")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        
        RateLimitConfig {
            max_requests,
            window: Duration::from_secs(60),
            burst,
        }
    }
}

/// Rate limiter state for tracking requests
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of client ID to their request history
    clients: Arc<RwLock<HashMap<String, ClientRateLimit>>>,
    config: RateLimitConfig,
}

/// Per-client rate limit tracking
#[derive(Debug)]
struct ClientRateLimit {
    /// Number of requests in current window
    requests: u32,
    /// Start of current window
    window_start: Instant,
    /// Burst tokens available
    burst_tokens: u32,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        RateLimiter {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a client is rate limited
    async fn check_rate_limit(&self, client_id: &str) -> RateLimitResult {
        let mut clients = self.clients.write().await;
        let now = Instant::now();
        
        let client = clients.entry(client_id.to_string()).or_insert_with(|| {
            ClientRateLimit {
                requests: 0,
                window_start: now,
                burst_tokens: self.config.burst,
            }
        });

        // Check if we need to reset the window
        if now.duration_since(client.window_start) >= self.config.window {
            client.requests = 0;
            client.window_start = now;
            client.burst_tokens = self.config.burst;
        }

        // Calculate remaining capacity
        let limit = self.config.max_requests + client.burst_tokens;
        let remaining = limit.saturating_sub(client.requests);
        let reset_time = client.window_start + self.config.window;
        
        if client.requests >= self.config.max_requests {
            // Use burst tokens if available
            if client.burst_tokens > 0 {
                client.burst_tokens -= 1;
                client.requests += 1;
                
                RateLimitResult::Allowed {
                    limit,
                    remaining: remaining.saturating_sub(1),
                    reset: reset_time,
                }
            } else {
                RateLimitResult::Limited {
                    limit,
                    reset: reset_time,
                }
            }
        } else {
            client.requests += 1;
            
            RateLimitResult::Allowed {
                limit,
                remaining: remaining.saturating_sub(1),
                reset: reset_time,
            }
        }
    }

    /// Clean up old entries periodically (call this in a background task)
    pub async fn cleanup(&self) {
        let mut clients = self.clients.write().await;
        let now = Instant::now();
        
        clients.retain(|_, client| {
            now.duration_since(client.window_start) < self.config.window * 2
        });
    }
}

#[derive(Debug)]
enum RateLimitResult {
    Allowed {
        limit: u32,
        remaining: u32,
        reset: Instant,
    },
    Limited {
        limit: u32,
        reset: Instant,
    },
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get client identifier (use IP, user ID, or API key)
    let client_id = extract_client_id(&req);
    
    // Check rate limit
    let result = limiter.check_rate_limit(&client_id).await;
    
    match result {
        RateLimitResult::Allowed { limit, remaining, reset } => {
            // Add rate limit headers
            let mut response = next.run(req).await;
            let headers = response.headers_mut();
            
            headers.insert("X-RateLimit-Limit", HeaderValue::from(limit));
            headers.insert("X-RateLimit-Remaining", HeaderValue::from(remaining));
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from(reset.elapsed().as_secs()),
            );
            
            Ok(response)
        }
        RateLimitResult::Limited { limit, reset } => {
            // Return 429 Too Many Requests
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests. Please slow down.",
                    "retry_after_seconds": reset.elapsed().as_secs()
                }))
            ).into_response();
            
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", HeaderValue::from(limit));
            headers.insert("X-RateLimit-Remaining", HeaderValue::from(0u32));
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from(reset.elapsed().as_secs()),
            );
            headers.insert(
                "Retry-After",
                HeaderValue::from(reset.elapsed().as_secs()),
            );
            
            Ok(response)
        }
    }
}

/// Extract client identifier from request
fn extract_client_id(req: &Request) -> String {
    // Try to get authenticated user ID from extensions
    if let Some(auth_user) = req.extensions().get::<crate::middleware::auth::AuthUser>() {
        return auth_user.session.user_id.clone();
    }
    
    // Fall back to IP address
    req.headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// Per-endpoint rate limiting with different limits
pub fn endpoint_rate_limiter(endpoint: &str) -> RateLimitConfig {
    match endpoint {
        "/api/auth/login" => RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(300), // 5 requests per 5 minutes
            burst: 2,
        },
        "/api/stream" => RateLimitConfig {
            max_requests: 10,
            window: Duration::from_secs(60), // 10 requests per minute
            burst: 5,
        },
        _ => RateLimitConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(1),
            burst: 1,
        };
        
        let limiter = RateLimiter::new(config);
        
        // Should allow first 3 requests
        for _ in 0..3 {
            match limiter.check_rate_limit("test_client").await {
                RateLimitResult::Allowed { .. } => {}
                _ => panic!("Should allow request"),
            }
        }
        
        // Should allow one more with burst
        match limiter.check_rate_limit("test_client").await {
            RateLimitResult::Allowed { .. } => {}
            _ => panic!("Should allow burst request"),
        }
        
        // Should be rate limited now
        match limiter.check_rate_limit("test_client").await {
            RateLimitResult::Limited { .. } => {}
            _ => panic!("Should be rate limited"),
        }
    }
}