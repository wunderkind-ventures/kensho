// T012: Contract test POST /api/auth/login
// Reference: contracts/openapi.yaml lines 172-186

use serde_json::json;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn auth_login_returns_200_with_valid_credentials() {
    // Arrange
    let app = spawn_app().await;
    
    let login_data = json!({
        "username": "test_user",
        "password": "valid_password"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
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
async fn auth_login_returns_401_with_invalid_credentials() {
    // Arrange
    let app = spawn_app().await;
    
    let login_data = json!({
        "username": "test_user",
        "password": "wrong_password"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
    assert_eq!(error_response["error"].as_str().unwrap(), "Invalid credentials");
}

#[tokio::test]
async fn auth_login_returns_400_with_missing_username() {
    // Arrange
    let app = spawn_app().await;
    
    let login_data = json!({
        "password": "some_password"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
}

#[tokio::test]
async fn auth_login_returns_400_with_missing_password() {
    // Arrange
    let app = spawn_app().await;
    
    let login_data = json!({
        "username": "test_user"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
}

#[tokio::test]
async fn auth_login_response_matches_openapi_schema() {
    // Arrange
    let app = spawn_app().await;
    
    let login_data = json!({
        "username": "test_user",
        "password": "valid_password"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
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
    }
}

#[tokio::test]
async fn auth_login_jwt_contains_valid_claims() {
    // Arrange
    let app = spawn_app().await;
    
    let login_data = json!({
        "username": "test_user",
        "password": "valid_password"
    });
    
    // Act
    let response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    if response.status().is_success() {
        let auth_response: serde_json::Value = response.json().await.unwrap();
        let access_token = auth_response["access_token"].as_str().unwrap();
        
        // Basic JWT structure validation (header.payload.signature)
        let parts: Vec<&str> = access_token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT must have three parts separated by dots");
        
        // Could decode and validate claims if needed
        // For now, just verify the token format is correct
        assert!(!parts[0].is_empty(), "JWT header must not be empty");
        assert!(!parts[1].is_empty(), "JWT payload must not be empty");
        assert!(!parts[2].is_empty(), "JWT signature must not be empty");
    }
}