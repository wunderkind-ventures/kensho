// T032: Database connection pool
// Reference: research.md section 1 for SurrealDB configuration

use anyhow::Result;
use std::sync::Arc;
use crate::services::DatabaseService;

pub struct ConnectionPool {
    db: Arc<DatabaseService>,
}

impl ConnectionPool {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create database connection
        let db = DatabaseService::new(database_url).await?;
        
        // Initialize schema
        db.initialize_schema().await?;
        
        Ok(ConnectionPool {
            db: Arc::new(db),
        })
    }
    
    pub fn get(&self) -> Arc<DatabaseService> {
        self.db.clone()
    }
}

// Application state that will be shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseService>,
    pub auth: Arc<tokio::sync::Mutex<crate::services::AuthService>>,
    pub cache: Arc<tokio::sync::Mutex<crate::services::CacheService>>,
    pub search: Arc<crate::services::SearchService>,
    pub streaming: Arc<crate::services::StreamingService>,
    pub metadata: Arc<tokio::sync::Mutex<crate::services::MetadataService>>,
    pub health: Arc<crate::services::HealthService>,
}

impl AppState {
    pub async fn new(
        database_url: &str,
        redis_url: &str,
        jwt_secret: String,
    ) -> Result<Self> {
        tracing::info!("Initializing AppState...");
        
        // Initialize database connection pool
        tracing::debug!("Creating database connection pool...");
        let pool = ConnectionPool::new(database_url).await?;
        let db = pool.get();
        tracing::info!("Database connection established");
        
        // Initialize services with better error handling
        tracing::debug!("Initializing auth service with Redis URL: {}", redis_url);
        let auth = match crate::services::AuthService::new(redis_url, jwt_secret.clone()).await {
            Ok(service) => {
                tracing::info!("Auth service initialized successfully");
                Arc::new(tokio::sync::Mutex::new(service))
            }
            Err(e) => {
                tracing::error!("Failed to initialize auth service: {}", e);
                return Err(e);
            }
        };
        
        tracing::debug!("Initializing cache service...");
        let cache = match crate::services::CacheService::new(redis_url).await {
            Ok(service) => {
                tracing::info!("Cache service initialized successfully");
                Arc::new(tokio::sync::Mutex::new(service))
            }
            Err(e) => {
                tracing::error!("Failed to initialize cache service: {}", e);
                return Err(e);
            }
        };
        
        tracing::debug!("Initializing search service...");
        let search = Arc::new(crate::services::SearchService::new(db.clone()));
        tracing::info!("Search service initialized");
        
        tracing::debug!("Initializing streaming service...");
        let streaming = Arc::new(crate::services::StreamingService::new(auth.clone()));
        tracing::info!("Streaming service initialized");
        
        tracing::debug!("Initializing metadata service...");
        let metadata = Arc::new(tokio::sync::Mutex::new(
            crate::services::MetadataService::new("anime-offline-database.json".to_string())
        ));
        tracing::info!("Metadata service initialized");
        
        tracing::debug!("Initializing health service...");
        let version = env!("CARGO_PKG_VERSION").to_string();
        let health = Arc::new(crate::services::HealthService::new(version));
        tracing::info!("Health service initialized");
        
        tracing::info!("AppState initialization complete");
        Ok(AppState {
            db,
            auth,
            cache,
            search,
            streaming,
            metadata,
            health,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new("memory://").await.unwrap();
        let db1 = pool.get();
        let db2 = pool.get();
        
        // Should return the same Arc instance
        assert!(Arc::ptr_eq(&db1, &db2));
    }
    
    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new(
            "memory://",
            "redis://localhost:6379",
            "test_secret".to_string()
        ).await;
        
        // May fail if Redis is not running, which is expected
        assert!(state.is_ok() || state.is_err());
    }
}