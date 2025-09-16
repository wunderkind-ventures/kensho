// T060: Frontend to backend log streaming endpoint
// Reference: plan.md line 70 "Frontend logs → backend"
// Reference: research.md section 10

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::db::connection::AppState;
use crate::middleware::auth::OptionalAuthUser;
use crate::middleware::json_extractor::ValidatedJson;

/// Log entry from frontend
#[derive(Debug, Deserialize, Serialize)]
pub struct FrontendLogEntry {
    /// Log level (debug, info, warn, error)
    pub level: String,
    /// Log message
    pub message: String,
    /// Timestamp from frontend
    pub timestamp: String,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// User agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Page URL where log originated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_url: Option<String>,
}

/// Batch of log entries
#[derive(Debug, Deserialize)]
pub struct FrontendLogBatch {
    pub entries: Vec<FrontendLogEntry>,
}

/// POST /api/logs/frontend - Receive logs from frontend
pub async fn receive_frontend_logs(
    State(_state): State<AppState>,
    OptionalAuthUser { session }: OptionalAuthUser,
    ValidatedJson(batch): ValidatedJson<FrontendLogBatch>,
) -> impl IntoResponse {
    // Extract user info if authenticated
    let user_id = session.as_ref().map(|s| s.user_id.clone());
    
    // Process each log entry
    for entry in batch.entries {
        // Map frontend log levels to tracing levels
        match entry.level.to_lowercase().as_str() {
            "error" => {
                tracing::error!(
                    target: "frontend",
                    user_id = ?user_id,
                    message = %entry.message,
                    timestamp = %entry.timestamp,
                    context = ?entry.context,
                    user_agent = ?entry.user_agent,
                    page_url = ?entry.page_url,
                    "Frontend error"
                );
            }
            "warn" | "warning" => {
                tracing::warn!(
                    target: "frontend",
                    user_id = ?user_id,
                    message = %entry.message,
                    timestamp = %entry.timestamp,
                    context = ?entry.context,
                    user_agent = ?entry.user_agent,
                    page_url = ?entry.page_url,
                    "Frontend warning"
                );
            }
            "info" => {
                tracing::info!(
                    target: "frontend",
                    user_id = ?user_id,
                    message = %entry.message,
                    timestamp = %entry.timestamp,
                    context = ?entry.context,
                    user_agent = ?entry.user_agent,
                    page_url = ?entry.page_url,
                    "Frontend info"
                );
            }
            "debug" => {
                tracing::debug!(
                    target: "frontend",
                    user_id = ?user_id,
                    message = %entry.message,
                    timestamp = %entry.timestamp,
                    context = ?entry.context,
                    user_agent = ?entry.user_agent,
                    page_url = ?entry.page_url,
                    "Frontend debug"
                );
            }
            _ => {
                tracing::trace!(
                    target: "frontend",
                    user_id = ?user_id,
                    message = %entry.message,
                    timestamp = %entry.timestamp,
                    context = ?entry.context,
                    user_agent = ?entry.user_agent,
                    page_url = ?entry.page_url,
                    "Frontend trace"
                );
            }
        }
    }
    
    (StatusCode::NO_CONTENT).into_response()
}

/// Frontend error report for critical errors
#[derive(Debug, Deserialize)]
pub struct FrontendErrorReport {
    pub error: String,
    pub stack_trace: Option<String>,
    pub user_agent: String,
    pub page_url: String,
    pub timestamp: String,
    pub session_id: Option<String>,
}

/// POST /api/logs/error - Report critical frontend errors
pub async fn report_frontend_error(
    State(_state): State<AppState>,
    OptionalAuthUser { session }: OptionalAuthUser,
    ValidatedJson(report): ValidatedJson<FrontendErrorReport>,
) -> impl IntoResponse {
    let user_id = session.as_ref().map(|s| s.user_id.clone());
    
    // Log critical error with full details
    tracing::error!(
        target: "frontend_critical",
        user_id = ?user_id,
        error = %report.error,
        stack_trace = ?report.stack_trace,
        user_agent = %report.user_agent,
        page_url = %report.page_url,
        timestamp = %report.timestamp,
        session_id = ?report.session_id,
        "Critical frontend error reported"
    );
    
    // Could also store in database for analysis
    // TODO: Store in error tracking table
    
    (StatusCode::NO_CONTENT).into_response()
}

/// Performance metrics from frontend
#[derive(Debug, Deserialize)]
pub struct FrontendPerformanceMetrics {
    pub page_url: String,
    pub load_time_ms: u32,
    pub dom_ready_ms: u32,
    pub first_contentful_paint_ms: Option<u32>,
    pub time_to_interactive_ms: Option<u32>,
    pub user_agent: String,
}

/// POST /api/logs/performance - Report frontend performance metrics
pub async fn report_performance_metrics(
    State(_state): State<AppState>,
    OptionalAuthUser { session }: OptionalAuthUser,
    ValidatedJson(metrics): ValidatedJson<FrontendPerformanceMetrics>,
) -> impl IntoResponse {
    let user_id = session.as_ref().map(|s| s.user_id.clone());
    
    // Log performance metrics
    tracing::info!(
        target: "frontend_performance",
        user_id = ?user_id,
        page_url = %metrics.page_url,
        load_time_ms = %metrics.load_time_ms,
        dom_ready_ms = %metrics.dom_ready_ms,
        first_contentful_paint_ms = ?metrics.first_contentful_paint_ms,
        time_to_interactive_ms = ?metrics.time_to_interactive_ms,
        user_agent = %metrics.user_agent,
        "Frontend performance metrics"
    );
    
    // Check against performance budgets
    if metrics.load_time_ms > 2000 {
        tracing::warn!(
            target: "frontend_performance",
            page_url = %metrics.page_url,
            load_time_ms = %metrics.load_time_ms,
            "Page load time exceeds 2s budget"
        );
    }
    
    (StatusCode::NO_CONTENT).into_response()
}