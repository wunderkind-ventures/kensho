// T037: GET /api/anime/{id}/episodes handler
// Reference: contracts/openapi.yaml lines 119-143

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use uuid::Uuid;
use serde_json::json;
use crate::db::connection::AppState;
use crate::models::EpisodeListResponse;

pub async fn get_episodes(
    Path(anime_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Check if anime exists
    match state.db.get_anime(anime_id).await {
        Ok(Some(_anime)) => {
            // Get episodes for this anime
            match state.db.get_anime_episodes(anime_id).await {
                Ok(episodes) => {
                    let response = EpisodeListResponse {
                        total: episodes.len(),
                        episodes: episodes.into_iter().map(|e| e.into()).collect(),
                    };
                    
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": format!("Failed to fetch episodes: {}", e)
                        }))
                    ).into_response()
                }
            }
        }
        Ok(None) => {
            (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Anime not found"
                }))
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to fetch anime: {}", e)
                }))
            ).into_response()
        }
    }
}