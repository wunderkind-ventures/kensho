// Middleware modules
pub mod auth;
pub mod cors;
pub mod error;
pub mod json_extractor;
pub mod logging;
pub mod rate_limit;

// Re-export commonly used types
pub use cors::get_cors_layer;
pub use logging::{logging_middleware, create_trace_layer};
