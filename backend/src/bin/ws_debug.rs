use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing raw WebSocket connection to SurrealDB...");
    
    let url = "ws://localhost:8000/rpc";
    println!("Connecting to {}...", url);
    
    match connect_async(url).await {
        Ok((mut ws_stream, response)) => {
            println!("✅ WebSocket connected!");
            println!("HTTP Response: {:?}", response);
            
            // Send a ping message
            let ping = r#"{"id":1,"method":"ping"}"#;
            println!("\nSending: {}", ping);
            ws_stream.send(Message::Text(ping.to_string())).await?;
            
            // Read response
            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Received: {}", text);
                    }
                    Ok(other) => {
                        println!("Received non-text message: {:?}", other);
                    }
                    Err(e) => {
                        println!("Error receiving message: {}", e);
                    }
                }
            }
            
            // Try authentication
            let auth = r#"{"id":2,"method":"signin","params":[{"user":"root","pass":"root"}]}"#;
            println!("\nSending auth: {}", auth);
            ws_stream.send(Message::Text(auth.to_string())).await?;
            
            // Read auth response
            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Auth response: {}", text);
                    }
                    Err(e) => {
                        println!("Auth error: {}", e);
                    }
                    _ => {}
                }
            }
            
            ws_stream.close(None).await?;
            println!("\n✅ WebSocket test completed successfully!");
        }
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            println!("Error details: {:?}", e);
        }
    }
    
    Ok(())
}