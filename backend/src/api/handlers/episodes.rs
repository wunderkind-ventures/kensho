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
use serde::{Deserialize, Serialize};
use crate::db::connection::AppState;
use crate::models::{Episode, EpisodeListResponse};

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

// Request DTO for creating episodes
#[derive(Debug, Deserialize)]
pub struct CreateEpisodeRequest {
    pub episode_number: u32,
    pub title: Option<String>,
    pub duration: Option<u32>,
    pub air_date: Option<String>,
    pub synopsis: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEpisodesRequest {
    pub episodes: Vec<CreateEpisodeRequest>,
}

// POST /api/anime/{id}/episodes handler
pub async fn create_episodes(
    Path(anime_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateEpisodesRequest>,
) -> impl IntoResponse {
    // Check if anime exists
    match state.db.get_anime(anime_id).await {
        Ok(Some(_anime)) => {
            let mut created_episodes = Vec::new();
            let mut errors = Vec::new();
            
            for ep_request in payload.episodes {
                let episode = Episode {
                    id: Uuid::new_v4(),
                    anime_id,
                    episode_number: ep_request.episode_number,
                    title: ep_request.title,
                    duration: ep_request.duration,
                    air_date: None, // Would need to parse from string
                    synopsis: ep_request.synopsis,
                    thumbnail_url: ep_request.thumbnail_url,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                
                match state.db.create_episode(&episode).await {
                    Ok(_) => created_episodes.push(episode),
                    Err(e) => errors.push(format!("Episode {}: {}", ep_request.episode_number, e)),
                }
            }
            
            if errors.is_empty() {
                (
                    StatusCode::CREATED,
                    Json(json!({
                        "created": created_episodes.len(),
                        "episodes": created_episodes
                    }))
                ).into_response()
            } else {
                (
                    StatusCode::PARTIAL_CONTENT,
                    Json(json!({
                        "created": created_episodes.len(),
                        "episodes": created_episodes,
                        "errors": errors
                    }))
                ).into_response()
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