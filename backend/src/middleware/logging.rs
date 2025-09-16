// T057: Request/response logging middleware
// Reference: research.md section 10 "Structured Logging"
// Reference: plan.md lines 69-71

use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::{Level, Span};
use uuid::Uuid;

/// Request ID extension for tracing requests through the system
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        RequestId(Uuid::new_v4().to_string())
    }
}

/// Logging middleware that adds request ID and logs request/response details
pub async fn logging_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Generate or extract request ID
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Add request ID to extensions for use in handlers
    req.extensions_mut().insert(RequestId(request_id.clone()));

    // Log request details
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        "Incoming request"
    );

    // Call the next middleware/handler
    let response = next.run(req).await;
    
    // Calculate request duration
    let duration = start.elapsed();
    let status = response.status();

    // Log response details
    if status.is_success() {
        tracing::info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed successfully"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Client error response"
        );
    } else {
        tracing::error!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Server error response"
        );
    }

    Ok(response)
}

/// Create a Tower HTTP trace layer for detailed HTTP logging
pub fn create_trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    DefaultMakeSpan,
    DefaultOnRequest,
    DefaultOnResponse,
> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

/// Log slow requests (useful for performance monitoring)
pub async fn slow_request_logger(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let uri = req.uri().clone();
    let method = req.method().clone();
    let start = Instant::now();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    
    // Log if request takes more than 1 second
    if duration.as_secs() >= 1 {
        tracing::warn!(
            method = %method,
            uri = %uri,
            duration_ms = %duration.as_millis(),
            "Slow request detected"
        );
    }
    
    Ok(response)
}

/// Structured logging configuration
pub fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default to info level for our crate, warn for others
            "kensho_backend=info,tower_http=debug,warn".into()
        });

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .json(); // Use JSON format for structured logging

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
    
    tracing::info!("Logging initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_generation() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();
        
        // Should generate different IDs
        assert_ne!(id1.0, id2.0);
        
        // Should be valid UUIDs
        assert!(Uuid::parse_str(&id1.0).is_ok());
        assert!(Uuid::parse_str(&id2.0).is_ok());
    }
}