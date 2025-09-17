// Common test utilities for integration and contract tests

use kensho_backend::db::connection::AppState;
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub state: AppState,
}

pub async fn spawn_app() -> TestApp {
    // Use a unique database for each test
    let db_name = format!("test_{}", Uuid::new_v4().to_string().replace("-", ""));
    let database_url = format!("memory://{}", db_name);
    let redis_url = "redis://:kensho_redis_pass@localhost:6379".to_string();
    let jwt_secret = "test_secret_key_for_testing_only".to_string();
    
    // Create application state
    let state = AppState::new(&database_url, &redis_url, jwt_secret)
        .await
        .expect("Failed to create application state");
    
    // Initialize database schema
    state.db.initialize_schema()
        .await
        .expect("Failed to initialize database schema");
    
    // Build the application
    let app = kensho_backend::api::routes::create_router(state.clone());
    
    // Start a test server on a random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    
    // Spawn the server in the background
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Failed to start server");
    });
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Wait for server to be ready
    for _ in 0..10 {
        if client.get(&format!("{}/api/health", address))
            .send()
            .await
            .is_ok() 
        {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    TestApp {
        address,
        client,
        state,
    }
}

// Helper function to create test authentication token
pub fn create_test_token(state: &AppState) -> String {
    use jsonwebtoken::{encode, Header, EncodingKey};
    use kensho_backend::models::Claims;
    use chrono::{Utc, Duration};
    
    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    ).expect("Failed to create test token")
}