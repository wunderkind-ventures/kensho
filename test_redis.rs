use redis::AsyncCommands;

#[tokio::main]
async fn main() {
    println!("Testing Redis connection...");
    
    // Test different connection string formats
    let urls = vec![
        "redis://:kensho_redis_pass@localhost:6379",
        "redis://default:kensho_redis_pass@localhost:6379",
        "redis://localhost:6379",
    ];
    
    for url in urls {
        println!("\nTrying: {}", url);
        match redis::Client::open(url) {
            Ok(client) => {
                println!("  Client created successfully");
                match client.get_connection_manager().await {
                    Ok(mut conn) => {
                        println!("  Connection established!");
                        // Try a simple command
                        let result: redis::RedisResult<String> = conn.set_ex("test_key", "test_value", 10).await;
                        match result {
                            Ok(_) => println!("  SET command successful"),
                            Err(e) => println!("  SET command failed: {}", e),
                        }
                        
                        let result: redis::RedisResult<Option<String>> = conn.get("test_key").await;
                        match result {
                            Ok(Some(val)) => println!("  GET command successful: {}", val),
                            Ok(None) => println!("  GET command returned None"),
                            Err(e) => println!("  GET command failed: {}", e),
                        }
                    }
                    Err(e) => println!("  Connection failed: {}", e),
                }
            }
            Err(e) => println!("  Client creation failed: {}", e),
        }
    }
}