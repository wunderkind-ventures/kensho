// T056: CORS configuration middleware
// Reference: research.md section 5 "Security Measures"
// Reference: plan.md line 38 for constraints

use tower_http::cors::{CorsLayer, Any};
use axum::http::{Method, HeaderName, header};

/// Configure CORS for the application
/// Allows the frontend to communicate with the backend
pub fn cors_layer() -> CorsLayer {
    // Get allowed origins from environment or use defaults
    let frontend_origin = std::env::var("CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let allowed_origins = vec![
        frontend_origin.parse().unwrap(),
        "http://localhost:3000".parse().unwrap(), // Backend dev server
        "http://localhost:8080".parse().unwrap(), // Frontend dev server
        "http://127.0.0.1:8080".parse().unwrap(),
    ];

    CorsLayer::new()
        // Allow specific origins
        .allow_origin(allowed_origins)
        // Allow common HTTP methods
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
        ])
        // Allow common headers
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-correlation-id"),
        ])
        // Expose headers that the frontend might need
        .expose_headers([
            header::CONTENT_LENGTH,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-ratelimit-limit"),
            HeaderName::from_static("x-ratelimit-remaining"),
            HeaderName::from_static("x-ratelimit-reset"),
        ])
        // Allow credentials (cookies, auth headers)
        .allow_credentials(true)
        // Cache preflight requests for 1 hour
        .max_age(std::time::Duration::from_secs(3600))
}

/// Permissive CORS for development
/// WARNING: Only use this in development environments
pub fn cors_layer_permissive() -> CorsLayer {
    tracing::warn!("Using permissive CORS configuration - DO NOT use in production!");
    
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-correlation-id"),
        ])
        .expose_headers([
            header::CONTENT_LENGTH,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-ratelimit-limit"),
            HeaderName::from_static("x-ratelimit-remaining"),
            HeaderName::from_static("x-ratelimit-reset"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
}

/// Get the appropriate CORS layer based on environment
pub fn get_cors_layer() -> CorsLayer {
    let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    
    match env.as_str() {
        "production" => {
            tracing::info!("Using production CORS configuration");
            cors_layer()
        }
        _ => {
            tracing::info!("Using development CORS configuration");
            // Use permissive CORS in development for easier testing
            cors_layer_permissive()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_layer_creation() {
        // Should not panic
        let _ = cors_layer();
        let _ = cors_layer_permissive();
        let _ = get_cors_layer();
    }
}