// T014: Contract test POST /api/auth/refresh
// Reference: contracts/openapi.yaml lines 203-217

use serde_json::json;

#[path = "../common/mod.rs"]
mod common;
use common::{spawn_app, create_test_token};

#[tokio::test]
async fn auth_refresh_returns_200_with_valid_refresh_token() {
    // Arrange
    let app = spawn_app().await;
    
    let refresh_data = json!({
        "refresh_token": "valid_refresh_token_string"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let auth_response: serde_json::Value = response.json().await.unwrap();
    assert!(auth_response["access_token"].is_string(), "access_token must be a string");
    assert!(auth_response["refresh_token"].is_string(), "refresh_token must be a string");
    assert!(auth_response["expires_in"].is_number(), "expires_in must be a number");
    assert!(auth_response["token_type"].is_string(), "token_type must be a string");
    assert_eq!(auth_response["token_type"].as_str().unwrap(), "Bearer");
}

#[tokio::test]
async fn auth_refresh_returns_401_with_invalid_refresh_token() {
    // Arrange
    let app = spawn_app().await;
    
    let refresh_data = json!({
        "refresh_token": "invalid_refresh_token"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
    assert_eq!(error_response["error"].as_str().unwrap(), "Invalid refresh token");
}

#[tokio::test]
async fn auth_refresh_returns_401_with_expired_refresh_token() {
    // Arrange
    let app = spawn_app().await;
    
    let refresh_data = json!({
        "refresh_token": "expired_refresh_token"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
    assert!(
        error_response["error"].as_str().unwrap().contains("expired") ||
        error_response["error"].as_str().unwrap().contains("Expired"),
        "Error message should indicate token expiration"
    );
}

#[tokio::test]
async fn auth_refresh_returns_400_with_missing_refresh_token() {
    // Arrange
    let app = spawn_app().await;
    
    let refresh_data = json!({});
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
}

#[tokio::test]
async fn auth_refresh_returns_new_tokens() {
    // Arrange
    let app = spawn_app().await;
    
    let refresh_data = json!({
        "refresh_token": "valid_refresh_token_1"
    });
    
    // Act - First refresh
    let response1 = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data)
        .send()
        .await
        .expect("Failed to send first request");
    
    let auth_response1: serde_json::Value = response1.json().await.unwrap();
    let access_token1 = auth_response1["access_token"].as_str().unwrap();
    let refresh_token1 = auth_response1["refresh_token"].as_str().unwrap();
    
    // Act - Second refresh with new refresh token
    let refresh_data2 = json!({
        "refresh_token": refresh_token1
    });
    
    let response2 = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data2)
        .send()
        .await
        .expect("Failed to send second request");
    
    if response2.status().is_success() {
        let auth_response2: serde_json::Value = response2.json().await.unwrap();
        let access_token2 = auth_response2["access_token"].as_str().unwrap();
        
        // Assert - New tokens should be different
        // Note: This depends on implementation - some systems return the same refresh token
        assert_ne!(access_token1, access_token2, "New access token should be different");
    }
}

#[tokio::test]
async fn auth_refresh_response_matches_openapi_schema() {
    // Arrange
    let app = spawn_app().await;
    
    let refresh_data = json!({
        "refresh_token": "valid_refresh_token"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - Check response matches OpenAPI schema (AuthResponse)
    if response.status().is_success() {
        let auth_response: serde_json::Value = response.json().await.unwrap();
        
        // Required fields from AuthResponse schema
        assert!(auth_response["access_token"].is_string(), "access_token must be a string");
        assert!(auth_response["refresh_token"].is_string(), "refresh_token must be a string");
        assert!(auth_response["expires_in"].is_number(), "expires_in must be a number");
        assert!(auth_response["token_type"].is_string(), "token_type must be a string");
        
        // Validate token_type enum value
        let token_type = auth_response["token_type"].as_str().unwrap();
        assert_eq!(token_type, "Bearer", "token_type must be 'Bearer'");
        
        // Validate expires_in is a positive integer
        let expires_in = auth_response["expires_in"].as_u64().unwrap();
        assert!(expires_in > 0, "expires_in must be a positive integer");
        
        // Validate JWT format for access_token
        let access_token = auth_response["access_token"].as_str().unwrap();
        let parts: Vec<&str> = access_token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT must have three parts separated by dots");
    }
}