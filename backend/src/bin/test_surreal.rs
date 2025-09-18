use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing SurrealDB connection...");
    
    // Try to connect
    println!("Connecting to ws://localhost:8000...");
    let db = match Surreal::new::<Ws>("ws://localhost:8000").await {
        Ok(db) => {
            println!("✓ Connected to SurrealDB");
            db
        },
        Err(e) => {
            println!("✗ Failed to connect: {}", e);
            return Err(e.into());
        }
    };
    
    // Try to authenticate
    println!("Authenticating as root...");
    match db.signin(Root {
        username: "root",
        password: "root",
    }).await {
        Ok(_) => println!("✓ Authenticated successfully"),
        Err(e) => {
            println!("✗ Failed to authenticate: {}", e);
            return Err(e.into());
        }
    }
    
    // Try to use namespace
    println!("Using namespace 'test'...");
    match db.use_ns("test").await {
        Ok(_) => println!("✓ Using namespace 'test'"),
        Err(e) => {
            println!("✗ Failed to use namespace: {}", e);
            return Err(e.into());
        }
    }
    
    // Try to use database
    println!("Using database 'test'...");
    match db.use_db("test").await {
        Ok(_) => println!("✓ Using database 'test'"),
        Err(e) => {
            println!("✗ Failed to use database: {}", e);
            return Err(e.into());
        }
    }
    
    // Try a simple query
    println!("Running test query...");
    match db.query("SELECT * FROM test LIMIT 1").await {
        Ok(_) => println!("✓ Query executed successfully"),
        Err(e) => println!("✗ Query failed (this is expected if table doesn't exist): {}", e),
    }
    
    println!("\n✅ All connection tests passed!");
    Ok(())
}