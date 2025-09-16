// T014: Contract test for POST /api/auth/refresh
// Reference: contracts/openapi.yaml lines 197-231

use axum::http::StatusCode;
use serde_json::{json, Value};
use chrono::{DateTime, Utc};

mod common;
use common::*;

#[tokio::test]
async fn test_refresh_token_success() {
    let app = setup_test_app().await;
    
    // First login to get tokens
    let (_, refresh_token) = create_test_session_with_refresh(&app).await;
    
    let refresh_request = json!({
        "refresh_token": refresh_token
    });
    
    let response = app
        .client
        .post("/api/auth/refresh")
        .json(&refresh_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Verify response schema
    assert!(body["token"].is_string());
    assert!(body["expires_at"].is_string());
    
    // New token should be different
    let new_token = body["token"].as_str().unwrap();
    assert!(!new_token.is_empty());
    
    // Verify expiry is in future
    let expires_at = body["expires_at"].as_str().unwrap();
    let expiry_time: DateTime<Utc> = expires_at.parse().expect("Invalid datetime format");
    assert!(expiry_time > Utc::now());
}

#[tokio::test]
async fn test_refresh_with_invalid_token() {
    let app = setup_test_app().await;
    
    let refresh_request = json!({
        "refresh_token": "invalid_refresh_token"
    });
    
    let response = app
        .client
        .post("/api/auth/refresh")
        .json(&refresh_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["error"].is_string());
    assert_eq!(body["message"].as_str(), Some("Invalid refresh token"));
}

#[tokio::test]
async fn test_refresh_with_expired_token() {
    let app = setup_test_app().await;
    
    // Create an expired refresh token
    let expired_refresh = create_expired_refresh_token().await;
    
    let refresh_request = json!({
        "refresh_token": expired_refresh
    });
    
    let response = app
        .client
        .post("/api/auth/refresh")
        .json(&refresh_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_missing_token() {
    let app = setup_test_app().await;
    
    let refresh_request = json!({});
    
    let response = app
        .client
        .post("/api/auth/refresh")
        .json(&refresh_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_refresh_updates_session() {
    let app = setup_test_app().await;
    
    let (old_token, refresh_token) = create_test_session_with_refresh(&app).await;
    
    let refresh_request = json!({
        "refresh_token": refresh_token
    });
    
    let response = app
        .client
        .post("/api/auth/refresh")
        .json(&refresh_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let new_token = body["token"].as_str().unwrap();
    
    // Old token should be invalidated
    let old_session = get_session_from_redis(&app.redis, &old_token).await;
    assert!(old_session.is_none(), "Old session should be invalidated");
    
    // New token should have session
    let new_session = get_session_from_redis(&app.redis, new_token).await;
    assert!(new_session.is_some(), "New session should exist");
}

// Helper functions
async fn create_test_session_with_refresh(app: &TestApp) -> (String, String) {
    panic!("Not implemented - test should fail");
}

async fn create_expired_refresh_token() -> String {
    panic!("Not implemented - test should fail");
}

async fn get_session_from_redis(redis: &redis::Client, token: &str) -> Option<String> {
    panic!("Not implemented - test should fail");
}