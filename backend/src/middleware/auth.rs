// T055: JWT authentication middleware
// Reference: research.md section 5 "JWT-Based Sessions"
// Reference: spec.md FR-004, FR-005, FR-007

use axum::{
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use crate::db::connection::AppState;
use crate::models::Session;

/// Extractor for authenticated requests
/// Add this to any handler that requires authentication
pub struct AuthUser {
    pub session: Session,
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        // Check for Bearer token format
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Verify the session using the auth service
        let mut auth_service = state.auth.lock().await;
        let session = auth_service
            .verify_session(token)
            .await
            .map_err(|e| {
                tracing::debug!("Session verification failed: {}", e);
                AuthError::InvalidSession
            })?;

        // Check if session is expired
        if session.is_expired() {
            return Err(AuthError::ExpiredSession);
        }

        Ok(AuthUser { session })
    }
}

/// Optional authentication extractor
/// Use this for endpoints that work with or without authentication
pub struct OptionalAuthUser {
    pub session: Option<Session>,
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to extract auth, but don't fail if missing
        match AuthUser::from_request_parts(parts, state).await {
            Ok(auth_user) => Ok(OptionalAuthUser {
                session: Some(auth_user.session),
            }),
            Err(_) => Ok(OptionalAuthUser { session: None }),
        }
    }
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    InvalidSession,
    ExpiredSession,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Missing authentication token",
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Invalid authentication token format",
            ),
            AuthError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired session",
            ),
            AuthError::ExpiredSession => (
                StatusCode::UNAUTHORIZED,
                "Session has expired, please login again",
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

/// Helper function to require specific permissions
/// Can be extended to check for specific user roles or permissions
pub fn require_permission(session: &Session, permission: &str) -> Result<(), AuthError> {
    // For now, just check if user is authenticated
    // In the future, could check session.permissions or user roles
    match permission {
        "stream" => {
            // Check if user has valid Crunchyroll token
            if session.cr_token_key.is_empty() {
                return Err(AuthError::InvalidSession);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_token_extraction() {
        let header_value = "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9";
        let token = header_value.strip_prefix("Bearer ");
        assert!(token.is_some());
        assert_eq!(token.unwrap(), "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9");
    }

    #[test]
    fn test_invalid_bearer_format() {
        let header_value = "Basic dXNlcjpwYXNz";
        let token = header_value.strip_prefix("Bearer ");
        assert!(token.is_none());
    }
}