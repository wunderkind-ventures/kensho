// T036: GET /api/browse/season handler
// Reference: contracts/openapi.yaml lines 79-117

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde_json::json;
use crate::db::connection::AppState;

pub async fn browse_season(
    Path((year, season)): Path<(u16, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Validate season
    let valid_seasons = ["spring", "summer", "fall", "winter"];
    if !valid_seasons.contains(&season.to_lowercase().as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid season. Must be one of: spring, summer, fall, winter"
            }))
        ).into_response();
    }
    
    // Search by season
    match state.search.search_by_season(year, &season).await {
        Ok(results) => {
            (
                StatusCode::OK,
                Json(json!({
                    "year": year,
                    "season": season,
                    "anime": results,
                    "total": results.len()
                }))
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to browse season: {}", e)
                }))
            ).into_response()
        }
    }
}