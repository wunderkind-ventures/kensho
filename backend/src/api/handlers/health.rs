// T066: Health check API endpoints
// Reference: plan.md Phase 4 - Production Hardening

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::db::connection::AppState;
use crate::services::health::{HealthService, HealthStatus};
use std::sync::Arc;

/// GET /health/live - Kubernetes liveness probe
/// Returns 200 if the application is alive
pub async fn liveness(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.health.check_liveness().await;
    
    (
        StatusCode::OK,
        Json(response)
    )
}

/// GET /health/ready - Kubernetes readiness probe
/// Returns 200 if ready to serve traffic, 503 if not
pub async fn readiness(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.health.check_readiness().await;
    
    let status_code = if response.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (
        status_code,
        Json(response)
    )
}

/// GET /health - Complete health check
/// Returns detailed health status of all components
pub async fn health(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.health.check_health().await;
    
    let status_code = match response.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };
    
    (
        status_code,
        Json(response)
    )
}

/// GET /health/components - Individual component health
/// Returns health status for each monitored component
pub async fn component_health(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.health.check_health().await;
    
    (
        StatusCode::OK,
        Json(response.checks)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::health::LivenessResponse;

    #[tokio::test]
    async fn test_liveness_endpoint() {
        // Liveness should always return 200 OK
        // This test would require setting up AppState
    }

    #[tokio::test]
    async fn test_readiness_endpoint() {
        // Readiness should return 503 if components are unhealthy
        // This test would require mocking unhealthy components
    }
}