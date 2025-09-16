// T017: Integration test "Login with valid/invalid credentials"
// Reference: quickstart.md "Test 3: Authentication Flow"

use serde_json::json;
mod common;
use common::*;

#[tokio::test]
async fn test_login_with_valid_credentials() {
    let app = setup_integration_test().await;
    
    let login_data = json!({
        "username": "test@example.com",
        "password": "valid_password"
    });
    
    let response = app.client
        .post(&format!("{}/api/auth/login", app.base_url))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to login");
    
    assert_eq!(response.status(), 200);
    panic!("Not fully implemented - test should fail");
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    panic!("Not implemented - test should fail");
}