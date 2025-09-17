pub mod metadata;
pub mod auth;
pub mod streaming;
// pub mod database; // Complex SurrealDB implementation
pub mod database_simplified;
pub mod cache;
pub mod search;
pub mod health;
pub mod resilient;
// pub mod crunchyroll_wrapper; // No longer needed - using crunchyroll-rs directly

pub use metadata::MetadataService;
pub use auth::AuthService;
pub use streaming::StreamingService;
pub use database_simplified::DatabaseService; // Use simplified version for POC
pub use cache::CacheService;
pub use search::SearchService;
pub use health::HealthService;
pub use resilient::{ResilientClient, ResilientHttpClient, ResilienceConfig, ResilienceManager};