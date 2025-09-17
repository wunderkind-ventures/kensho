// T033: Setup Axum router and middleware
// Reference: contracts/openapi.yaml for API endpoints

use axum::{
    Router,
    routing::{get, post},
    middleware as axum_middleware,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tower_http::{
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
};
use crate::db::connection::AppState;
use crate::middleware::{
    get_cors_layer,
    logging_middleware,
    create_trace_layer,
};
use serde_json::json;

pub fn create_router(state: AppState) -> Router {
    // API routes
    let api_routes = Router::new()
        // Anime endpoints
        .route("/anime", post(crate::api::handlers::anime::create_anime))
        .route("/anime/:id", get(crate::api::handlers::anime::get_anime))
        .route("/anime/:id/episodes", get(crate::api::handlers::episodes::get_episodes))
        .route("/anime/:id/episodes", post(crate::api::handlers::episodes::create_episodes))
        
        // Search and browse
        .route("/search", get(crate::api::handlers::search::search))
        .route("/browse/season/:year/:season", get(crate::api::handlers::browse::browse_season))
        
        // Authentication
        .route("/auth/login", post(crate::api::handlers::auth::login))
        .route("/auth/logout", post(crate::api::handlers::auth::logout))
        .route("/auth/refresh", post(crate::api::handlers::auth::refresh))
        
        // Streaming
        .route("/stream/:anime_id/:episode", get(crate::api::handlers::stream::get_stream))
        
        // Frontend logging endpoints
        .route("/logs/frontend", post(crate::api::handlers::logs::receive_frontend_logs))
        .route("/logs/error", post(crate::api::handlers::logs::report_frontend_error))
        .route("/logs/performance", post(crate::api::handlers::logs::report_performance_metrics))
        
        // Legacy health check
        .route("/health", get(health_check))
        
        // Production health checks (T066)
        .route("/health/live", get(crate::api::handlers::health::liveness))
        .route("/health/ready", get(crate::api::handlers::health::readiness))
        .route("/health/full", get(crate::api::handlers::health::health))
        .route("/health/components", get(crate::api::handlers::health::component_health))
        
        .with_state(state);
    
    // Main router with middleware
    Router::new()
        .nest("/api", api_routes)
        // Add fallback for 404 handling
        .fallback(handle_404)
        // Add custom logging middleware
        .layer(axum_middleware::from_fn(logging_middleware))
        // Add middleware layers
        .layer(get_cors_layer())
        .layer(CompressionLayer::new())
        .layer(create_trace_layer())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "Not Found",
            "message": "The requested resource was not found",
            "code": 404
        }))
    )
}

