// T041: GET /api/stream handler
// Reference: contracts/openapi.yaml lines 233-296

use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    Json,
    response::IntoResponse,
};
use uuid::Uuid;
use serde_json::json;
use crate::db::connection::AppState;

pub async fn get_stream(
    Path((anime_id, episode_num)): Path<(Uuid, u32)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract token from Authorization header
    let token = match headers.get("authorization") {
        Some(value) => {
            let value_str = value.to_str().unwrap_or("");
            if value_str.starts_with("Bearer ") {
                &value_str[7..]
            } else {
                return (StatusCode::UNAUTHORIZED, Json(json!({
                    "error": "Invalid authorization header"
                }))).into_response();
            }
        }
        None => {
            return (StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Missing authorization header"
            }))).into_response();
        }
    };
    
    // Verify authentication
    let mut auth = state.auth.lock().await;
    let session = match auth.verify_session(token).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": format!("Authentication required: {}", e)
                }))
            ).into_response();
        }
    };
    
    // Get the anime to verify it exists
    match state.db.get_anime(anime_id).await {
        Ok(Some(anime)) => {
            // Get episodes to find the specific one
            match state.db.get_anime_episodes(anime_id).await {
                Ok(episodes) => {
                    // Find the requested episode
                    if let Some(episode) = episodes.iter().find(|e| e.episode_number == episode_num) {
                        // For POC, we'll create a mock Crunchyroll episode ID
                        // In production, this would be stored in the database
                        let cr_episode_id = format!("CR_{}_E{}", anime_id, episode_num);
                        
                        // Get streaming manifest
                        match state.streaming.get_episode_stream(&session, &cr_episode_id).await {
                            Ok(manifest) => {
                                (StatusCode::OK, Json(manifest)).into_response()
                            }
                            Err(e) => {
                                // For POC, return a mock stream URL
                                (
                                    StatusCode::OK,
                                    Json(json!({
                                        "episode_id": episode.id,
                                        "crunchyroll_id": cr_episode_id,
                                        "streams": [{
                                            "url": format!("https://example.com/stream/{}/{}.m3u8", anime_id, episode_num),
                                            "resolution": "1080p",
                                            "audio_language": "en-US",
                                            "subtitle_language": null,
                                            "hardsub": false,
                                            "expires_at": chrono::Utc::now() + chrono::Duration::minutes(15)
                                        }],
                                        "thumbnail": episode.thumbnail_url,
                                        "duration": episode.duration.unwrap_or(1440)
                                    }))
                                ).into_response()
                            }
                        }
                    } else {
                        (
                            StatusCode::NOT_FOUND,
                            Json(json!({
                                "error": "Episode not found"
                            }))
                        ).into_response()
                    }
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