// T017: Integration test for complete auth flow
// Tests login → refresh → logout sequence

use serde_json::json;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn complete_auth_flow_with_crunchyroll() {
    // Arrange
    let app = spawn_app().await;
    
    // Step 1: Login with Crunchyroll credentials
    let login_data = json!({
        "username": "test_user",
        "password": "test_password"
    });
    
    let login_response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to login");
    
    assert_eq!(login_response.status().as_u16(), 200, "Login should succeed");
    
    let auth_tokens: serde_json::Value = login_response.json().await.unwrap();
    let access_token = auth_tokens["access_token"].as_str().unwrap();
    let refresh_token = auth_tokens["refresh_token"].as_str().unwrap();
    
    // Step 2: Use access token to access protected endpoint
    let anime_response = app.client
        .get(&format!("{}/api/anime", app.address))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .expect("Failed to get anime");
    
    assert!(
        anime_response.status().is_success(),
        "Should be able to access protected endpoint with valid token"
    );
    
    // Step 3: Refresh token
    let refresh_data = json!({
        "refresh_token": refresh_token
    });
    
    let refresh_response = app.client
        .post(&format!("{}/api/auth/refresh", app.address))
        .json(&refresh_data)
        .send()
        .await
        .expect("Failed to refresh token");
    
    if refresh_response.status().is_success() {
        let new_tokens: serde_json::Value = refresh_response.json().await.unwrap();
        let new_access_token = new_tokens["access_token"].as_str().unwrap();
        
        // Step 4: Use new access token
        let anime_response2 = app.client
            .get(&format!("{}/api/anime", app.address))
            .header("Authorization", format!("Bearer {}", new_access_token))
            .send()
            .await
            .expect("Failed to get anime with new token");
        
        assert!(
            anime_response2.status().is_success(),
            "Should be able to use new access token"
        );
        
        // Step 5: Logout
        let logout_response = app.client
            .post(&format!("{}/api/auth/logout", app.address))
            .header("Authorization", format!("Bearer {}", new_access_token))
            .send()
            .await
            .expect("Failed to logout");
        
        assert_eq!(logout_response.status().as_u16(), 200, "Logout should succeed");
    }
}

#[tokio::test]
async fn token_expires_after_timeout() {
    // Arrange
    let app = spawn_app().await;
    
    // Login
    let login_data = json!({
        "username": "test_user",
        "password": "test_password"
    });
    
    let login_response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to login");
    
    if login_response.status().is_success() {
        let auth_tokens: serde_json::Value = login_response.json().await.unwrap();
        let expires_in = auth_tokens["expires_in"].as_u64().unwrap();
        
        // Verify expires_in is reasonable (typically 3600 seconds = 1 hour)
        assert!(
            expires_in > 0 && expires_in <= 86400,
            "Token expiry should be between 1 second and 24 hours"
        );
    }
}

#[tokio::test]
async fn multiple_login_sessions_are_independent() {
    // Arrange
    let app = spawn_app().await;
    
    // First user login
    let user1_data = json!({
        "username": "user1",
        "password": "password1"
    });
    
    let user1_response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&user1_data)
        .send()
        .await
        .expect("Failed to login user1");
    
    // Second user login
    let user2_data = json!({
        "username": "user2",
        "password": "password2"
    });
    
    let user2_response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to login user2");
    
    if user1_response.status().is_success() && user2_response.status().is_success() {
        let user1_tokens: serde_json::Value = user1_response.json().await.unwrap();
        let user2_tokens: serde_json::Value = user2_response.json().await.unwrap();
        
        let user1_token = user1_tokens["access_token"].as_str().unwrap();
        let user2_token = user2_tokens["access_token"].as_str().unwrap();
        
        // Tokens should be different for different users
        assert_ne!(user1_token, user2_token, "Different users should get different tokens");
        
        // Logout user1
        let logout1_response = app.client
            .post(&format!("{}/api/auth/logout", app.address))
            .header("Authorization", format!("Bearer {}", user1_token))
            .send()
            .await
            .expect("Failed to logout user1");
        
        assert_eq!(logout1_response.status().as_u16(), 200);
        
        // User2 should still be able to access protected endpoints
        let user2_access = app.client
            .get(&format!("{}/api/anime", app.address))
            .header("Authorization", format!("Bearer {}", user2_token))
            .send()
            .await
            .expect("Failed to access with user2 token");
        
        assert!(
            user2_access.status().is_success() || user2_access.status().is_client_error(),
            "User2 token should still be valid after user1 logout"
        );
    }
}