// T012: Contract test for POST /api/auth/login
// Reference: contracts/openapi.yaml lines 145-182
// Reference: research.md section 5 "Session Flow"

use axum::http::StatusCode;
use serde_json::{json, Value};
use chrono::{DateTime, Utc, Duration};

mod common;
use common::*;

#[tokio::test]
async fn test_login_success() {
    let app = setup_test_app().await;
    
    // Mock Crunchyroll authentication
    mock_crunchyroll_auth_success();
    
    let login_request = json!({
        "username": "test@example.com",
        "password": "valid_password123"
    });
    
    let response = app
        .client
        .post("/api/auth/login")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Verify response matches OpenAPI schema
    assert!(body["token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert!(body["expires_at"].is_string());
    
    // Verify JWT token format
    let token = body["token"].as_str().unwrap();
    assert!(token.split('.').count() == 3, "Token should be valid JWT format");
    
    // Verify expiry is in future (15 minutes as per spec)
    let expires_at = body["expires_at"].as_str().unwrap();
    let expiry_time: DateTime<Utc> = expires_at.parse().expect("Invalid datetime format");
    let now = Utc::now();
    let expected_expiry = now + Duration::minutes(15);
    
    assert!(expiry_time > now, "Expiry should be in future");
    assert!(expiry_time <= expected_expiry + Duration::seconds(5), "Expiry should be ~15 minutes");
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = setup_test_app().await;
    
    // Mock Crunchyroll authentication failure
    mock_crunchyroll_auth_failure();
    
    let login_request = json!({
        "username": "test@example.com",
        "password": "wrong_password"
    });
    
    let response = app
        .client
        .post("/api/auth/login")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    assert!(body["error"].is_string());
    assert_eq!(body["message"].as_str(), Some("Invalid credentials"));
}

#[tokio::test]
async fn test_login_missing_username() {
    let app = setup_test_app().await;
    
    let login_request = json!({
        "password": "password123"
    });
    
    let response = app
        .client
        .post("/api/auth/login")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn test_login_missing_password() {
    let app = setup_test_app().await;
    
    let login_request = json!({
        "username": "test@example.com"
    });
    
    let response = app
        .client
        .post("/api/auth/login")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn test_login_empty_credentials() {
    let app = setup_test_app().await;
    
    let login_request = json!({
        "username": "",
        "password": ""
    });
    
    let response = app
        .client
        .post("/api/auth/login")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_stores_session_in_redis() {
    let app = setup_test_app().await;
    
    mock_crunchyroll_auth_success();
    
    let login_request = json!({
        "username": "test@example.com",
        "password": "valid_password123"
    });
    
    let response = app
        .client
        .post("/api/auth/login")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let token = body["token"].as_str().unwrap();
    
    // Verify session is stored in Redis
    let session = get_session_from_redis(&app.redis, token).await;
    assert!(session.is_some(), "Session should be stored in Redis");
}

#[tokio::test]
async fn test_login_rate_limiting() {
    let app = setup_test_app().await;
    
    // Attempt multiple failed logins
    for _ in 0..6 {
        let login_request = json!({
            "username": "test@example.com",
            "password": "wrong_password"
        });
        
        let _ = app
            .client
            .post("/api/auth/login")
            .json(&login_request)
            .send()
            .await;
    }
    
    // Next attempt should be rate limited
    let login_request = json!({
        "username": "test@example.com",
        "password": "valid_password"
    });
    
    let response = app
        .client
        .post("/api/auth/login")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

// Helper functions that will be implemented later
fn mock_crunchyroll_auth_success() {
    panic!("Not implemented - test should fail");
}

fn mock_crunchyroll_auth_failure() {
    panic!("Not implemented - test should fail");
}

async fn get_session_from_redis(
    redis: &redis::Client,
    token: &str
) -> Option<String> {
    panic!("Not implemented - test should fail");
}