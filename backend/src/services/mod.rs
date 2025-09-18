pub mod metadata;
pub mod auth;
pub mod streaming;
// pub mod database; // Old implementation with v2 issues
pub mod database_v2; // Fixed SurrealDB v2 implementation
pub mod database_simplified; // Keep as fallback
pub mod cache;
pub mod search;
pub mod health;
pub mod resilient;
pub mod data_loader;
// pub mod crunchyroll_wrapper; // No longer needed - using crunchyroll-rs directly

pub use metadata::MetadataService;
pub use auth::AuthService;
pub use streaming::StreamingService;
pub use database_v2::DatabaseService; // Use fixed v2 implementation
pub use cache::CacheService;
pub use search::SearchService;
pub use health::HealthService;
pub use resilient::{ResilientClient, ResilientHttpClient, ResilienceConfig, ResilienceManager};