// T059: Error handling middleware
// Reference: research.md section 10 "Error Handling"
// Reference: contracts/openapi.yaml Error schema

use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::fmt;

/// Standard error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code for client reference
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    // Client errors (4xx)
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    ValidationError(Vec<ValidationError>),
    RateLimited,
    
    // Server errors (5xx)
    InternalServer(String),
    DatabaseError(String),
    ExternalServiceError(String),
    Timeout,
    
    // Specific errors
    AuthenticationFailed,
    SessionExpired,
    CrunchyrollError(String),
    RedisError(String),
}

/// Validation error details
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::ValidationError(errors) => {
                write!(f, "Validation failed: {} errors", errors.len())
            }
            AppError::RateLimited => write!(f, "Rate limit exceeded"),
            AppError::InternalServer(msg) => write!(f, "Internal server error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
            AppError::Timeout => write!(f, "Request timeout"),
            AppError::AuthenticationFailed => write!(f, "Authentication failed"),
            AppError::SessionExpired => write!(f, "Session expired"),
            AppError::CrunchyrollError(msg) => write!(f, "Crunchyroll error: {}", msg),
            AppError::RedisError(msg) => write!(f, "Redis error: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            // Client errors
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg, None)
            }
            AppError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg, None)
            }
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, "FORBIDDEN", msg, None)
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", msg, None)
            }
            AppError::Conflict(msg) => {
                (StatusCode::CONFLICT, "CONFLICT", msg, None)
            }
            AppError::ValidationError(errors) => {
                let details = json!({
                    "errors": errors
                });
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "VALIDATION_ERROR",
                    "Validation failed".to_string(),
                    Some(details),
                )
            }
            AppError::RateLimited => {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "RATE_LIMITED",
                    "Too many requests".to_string(),
                    None,
                )
            }
            
            // Server errors
            AppError::InternalServer(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal error occurred".to_string(),
                    None,
                )
            }
            AppError::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Database operation failed".to_string(),
                    None,
                )
            }
            AppError::ExternalServiceError(msg) => {
                tracing::error!("External service error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    "EXTERNAL_SERVICE_ERROR",
                    "External service unavailable".to_string(),
                    None,
                )
            }
            AppError::Timeout => {
                (
                    StatusCode::GATEWAY_TIMEOUT,
                    "TIMEOUT",
                    "Request timed out".to_string(),
                    None,
                )
            }
            
            // Specific errors
            AppError::AuthenticationFailed => {
                (
                    StatusCode::UNAUTHORIZED,
                    "AUTH_FAILED",
                    "Authentication failed".to_string(),
                    None,
                )
            }
            AppError::SessionExpired => {
                (
                    StatusCode::UNAUTHORIZED,
                    "SESSION_EXPIRED",
                    "Session has expired".to_string(),
                    None,
                )
            }
            AppError::CrunchyrollError(msg) => {
                tracing::error!("Crunchyroll API error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    "CRUNCHYROLL_ERROR",
                    "Streaming service error".to_string(),
                    None,
                )
            }
            AppError::RedisError(msg) => {
                tracing::error!("Redis error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CACHE_ERROR",
                    "Cache service error".to_string(),
                    None,
                )
            }
        };

        let response = ErrorResponse {
            code: code.to_string(),
            message,
            details,
            request_id: None, // TODO: Extract from request extensions
        };

        (status, Json(response)).into_response()
    }
}

/// Convert from anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to specific error types
        if let Some(db_err) = err.downcast_ref::<surrealdb::Error>() {
            return AppError::DatabaseError(db_err.to_string());
        }
        
        if let Some(redis_err) = err.downcast_ref::<redis::RedisError>() {
            return AppError::RedisError(redis_err.to_string());
        }
        
        // Default to internal server error
        AppError::InternalServer(err.to_string())
    }
}

/// Convert from Redis errors
impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::RedisError(err.to_string())
    }
}

/// Result type alias for handlers
pub type AppResult<T> = Result<T, AppError>;

/// Helper function to validate request data
pub fn validate_request<T>(data: T, validators: Vec<impl Fn(&T) -> Option<ValidationError>>) 
    -> Result<T, AppError> {
    let mut errors = Vec::new();
    
    for validator in validators {
        if let Some(error) = validator(&data) {
            errors.push(error);
        }
    }
    
    if errors.is_empty() {
        Ok(data)
    } else {
        Err(AppError::ValidationError(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: "Resource not found".to_string(),
            details: None,
            request_id: Some("test-123".to_string()),
        };
        
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("NOT_FOUND"));
        assert!(json.contains("test-123"));
    }

    #[test]
    fn test_app_error_conversion() {
        let error = AppError::NotFound("Anime not found".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}