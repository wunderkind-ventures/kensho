use kensho_backend::models::{Session, SessionResponse};

#[tokio::main]
async fn main() {
    println!("Testing JWT token generation...\n");
    
    let jwt_secret = "test_secret_key_for_jwt_signing";
    let user_id = "test_user_123";
    let cr_token_key = "cr_token:test_user_123";
    
    // Create a new session
    match Session::new(user_id.to_string(), cr_token_key.to_string(), jwt_secret) {
        Ok(session) => {
            println!("✅ Session created successfully!");
            println!("  Session ID: {}", session.id);
            println!("  User ID: {}", session.user_id);
            println!("  Expires at: {}", session.expires_at);
            println!("  JWT Token (first 50 chars): {}...", &session.jwt_token[..50.min(session.jwt_token.len())]);
            
            // Verify the token
            match Session::verify_token(&session.jwt_token, jwt_secret) {
                Ok(claims) => {
                    println!("\n✅ Token verification successful!");
                    println!("  Subject (user_id): {}", claims.sub);
                    println!("  Session ID: {}", claims.session_id);
                    println!("  Expiry: {}", claims.exp);
                    println!("  CR Token Key: {}", claims.cr_token_key);
                }
                Err(e) => {
                    println!("\n❌ Token verification failed: {}", e);
                }
            }
            
            // Test refresh
            let mut session_mut = session;
            let original_token = session_mut.jwt_token.clone();
            match session_mut.refresh(jwt_secret) {
                Ok(new_token) => {
                    println!("\n✅ Token refresh successful!");
                    println!("  Tokens are different: {}", original_token != new_token);
                }
                Err(e) => {
                    println!("\n❌ Token refresh failed: {}", e);
                }
            }
            
            // Test response conversion
            let response = session_mut.to_response();
            println!("\n✅ Session response created:");
            println!("  Has token: {}", !response.token.is_empty());
            println!("  Has refresh token: {}", response.refresh_token.is_some());
            println!("  Expires at: {}", response.expires_at);
        }
        Err(e) => {
            println!("❌ Failed to create session: {}", e);
        }
    }
}