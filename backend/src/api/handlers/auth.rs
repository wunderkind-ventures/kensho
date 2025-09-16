// T038, T039, T040: Authentication handlers
// Reference: contracts/openapi.yaml lines 145-230

use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::db::connection::AppState;
use crate::middleware::json_extractor::ValidatedJson;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    refresh_token: Option<String>,
}

// T038: POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> impl IntoResponse {
    let mut auth = state.auth.lock().await;
    
    match auth.login(&req.email, &req.password).await {
        Ok(session_response) => {
            let response = LoginResponse {
                token: session_response.token,
                expires_at: session_response.expires_at,
                refresh_token: session_response.refresh_token,
            };
            
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": format!("Authentication failed: {}", e)
                }))
            ).into_response()
        }
    }
}

// T039: POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let mut auth = state.auth.lock().await;
    
    // Extract token from Authorization header
    let token = match headers.get("authorization") {
        Some(value) => {
            let value_str = value.to_str().unwrap_or("");
            if value_str.starts_with("Bearer ") {
                &value_str[7..]
            } else {
                return (StatusCode::UNAUTHORIZED, Json(json!({
                    "error": "Invalid authorization header"
                }))).into_response();
            }
        }
        None => {
            return (StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Missing authorization header"
            }))).into_response();
        }
    };
    
    match auth.logout(token).await {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(json!({
                    "message": "Logged out successfully"
                }))
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Logout failed: {}", e)
                }))
            ).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

// T040: POST /api/auth/refresh
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    let mut auth = state.auth.lock().await;
    
    match auth.refresh_session(&req.refresh_token).await {
        Ok(session_response) => {
            let response = LoginResponse {
                token: session_response.token,
                expires_at: session_response.expires_at,
                refresh_token: session_response.refresh_token,
            };
            
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": format!("Token refresh failed: {}", e)
                }))
            ).into_response()
        }
    }
}