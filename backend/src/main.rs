use anyhow::Result;
use std::net::SocketAddr;
use tracing_subscriber;

mod api;
mod db;
mod models;
mod services;
mod middleware;
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "memory://".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://:kensho_redis_pass@localhost:6379".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "development_secret_key_change_in_production".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;
    
    // Initialize application state
    tracing::info!("Creating application state...");
    let state = match db::connection::AppState::new(&database_url, &redis_url, jwt_secret).await {
        Ok(s) => {
            tracing::info!("Application state created successfully");
            s
        }
        Err(e) => {
            tracing::error!("Failed to create application state: {}", e);
            return Err(e);
        }
    };
    
    // Create router
    let app = api::routes::create_router(state);
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}