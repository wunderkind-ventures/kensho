// Middleware modules
pub mod auth;
pub mod cors;
pub mod error;
pub mod json_extractor;
pub mod logging;
pub mod rate_limit;

// Re-export commonly used types
pub use auth::{AuthUser, OptionalAuthUser};
pub use cors::{cors_layer, cors_layer_permissive, get_cors_layer};
pub use error::{AppError, AppResult, ErrorResponse};
pub use logging::{logging_middleware, create_trace_layer, init_logging, RequestId};
pub use rate_limit::{RateLimiter, RateLimitConfig, rate_limit_middleware};