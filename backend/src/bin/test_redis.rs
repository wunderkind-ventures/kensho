use redis::AsyncCommands;

#[tokio::main]
async fn main() {
    println!("Testing Redis connection...");
    
    // Test the connection string we're using
    let url = "redis://:kensho_redis_pass@localhost:6379";
    println!("Trying: {}", url);
    
    match redis::Client::open(url) {
        Ok(client) => {
            println!("  Client created successfully");
            match client.get_connection_manager().await {
                Ok(mut conn) => {
                    println!("  Connection established!");
                    
                    // Try a simple SET command
                    let result: redis::RedisResult<()> = conn.set_ex("test_key", "test_value", 10).await;
                    match result {
                        Ok(_) => println!("  SET command successful"),
                        Err(e) => println!("  SET command failed: {}", e),
                    }
                    
                    // Try a simple GET command
                    let result: redis::RedisResult<Option<String>> = conn.get("test_key").await;
                    match result {
                        Ok(Some(val)) => println!("  GET command successful: {}", val),
                        Ok(None) => println!("  GET command returned None"),
                        Err(e) => println!("  GET command failed: {}", e),
                    }
                    
                    println!("\n✅ Redis connection and operations successful!");
                }
                Err(e) => {
                    println!("  ❌ Connection failed: {}", e);
                    println!("  Error details: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Client creation failed: {}", e);
            println!("  Error details: {:?}", e);
        }
    }
}