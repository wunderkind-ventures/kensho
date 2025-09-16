use kensho_backend::services::AuthService;
use kensho_backend::models::SessionResponse;

#[tokio::main]
async fn main() {
    println!("=== Testing Authentication Service ===\n");
    
    // Initialize with environment variables
    dotenvy::dotenv().ok();
    
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://:kensho_redis_pass@localhost:6379".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "development_secret_key_change_in_production".to_string());
    
    println!("Configuration:");
    println!("  Redis URL: {}", redis_url);
    println!("  JWT Secret: {}\n", if jwt_secret.is_empty() { "(empty)" } else { "(set)" });
    
    // Test 1: Create auth service
    println!("Test 1: Creating auth service...");
    let mut auth_service = match AuthService::new(&redis_url, jwt_secret.clone()).await {
        Ok(service) => {
            println!("✅ Auth service created successfully\n");
            service
        }
        Err(e) => {
            println!("❌ Failed to create auth service: {}", e);
            println!("\nTroubleshooting:");
            println!("1. Make sure Redis is running: docker-compose up -d redis");
            println!("2. Check Redis connection: docker exec kensho-redis redis-cli -a kensho_redis_pass ping");
            println!("3. Check Redis logs: docker logs kensho-redis");
            return;
        }
    };
    
    // Test 2: Mock login
    println!("Test 2: Testing mock login (test@example.com/password)...");
    match auth_service.login("test@example.com", "password").await {
        Ok(session_response) => {
            println!("✅ Login successful!");
            println!("  Token (first 50 chars): {}...", &session_response.token[..50.min(session_response.token.len())]);
            println!("  Expires at: {}", session_response.expires_at);
            println!("  Has refresh token: {}\n", session_response.refresh_token.is_some());
            
            // Test 3: Verify the session
            println!("Test 3: Verifying session with token...");
            match auth_service.verify_session(&session_response.token).await {
                Ok(session) => {
                    println!("✅ Session verified successfully!");
                    println!("  Session ID: {}", session.id);
                    println!("  User ID: {}", session.user_id);
                    println!("  Is expired: {}\n", session.is_expired());
                    
                    // Test 4: Refresh token
                    if let Some(refresh_token) = &session_response.refresh_token {
                        println!("Test 4: Refreshing session...");
                        match auth_service.refresh_session(refresh_token).await {
                            Ok(new_session) => {
                                println!("✅ Session refreshed successfully!");
                                println!("  New token different: {}", new_session.token != session_response.token);
                                println!("  New expiry: {}\n", new_session.expires_at);
                            }
                            Err(e) => {
                                println!("❌ Failed to refresh session: {}\n", e);
                            }
                        }
                    }
                    
                    // Test 5: Logout
                    println!("Test 5: Logging out...");
                    match auth_service.logout(&session_response.token).await {
                        Ok(_) => {
                            println!("✅ Logout successful!\n");
                            
                            // Test 6: Verify session is invalid after logout
                            println!("Test 6: Verifying session is invalid after logout...");
                            match auth_service.verify_session(&session_response.token).await {
                                Ok(_) => {
                                    println!("❌ Session still valid after logout (should be invalid)");
                                }
                                Err(_) => {
                                    println!("✅ Session properly invalidated after logout");
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to logout: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to verify session: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Login failed: {}", e);
            println!("\nNote: This test uses mock credentials (test@example.com/password)");
            println!("Real Crunchyroll authentication would require valid credentials.");
        }
    }
    
    println!("\n=== Test Complete ===");
}