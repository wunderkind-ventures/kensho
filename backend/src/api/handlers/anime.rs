// T034: GET /api/anime/{id} handler
// Reference: contracts/openapi.yaml lines 24-44

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
use crate::models::{Anime, AnimeDetail, RelatedAnime, AnimeStatus, AnimeType, AnimeSeason, Season};

pub async fn get_anime(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Get anime from database
    match state.db.get_anime(id).await {
        Ok(Some(anime)) => {
            // Get tags for this anime
            let tags = state.db.get_anime_tags(id).await.unwrap_or_default();
            
            // Get related anime (simplified for POC)
            let similar = state.db.get_similar_anime(id, 5).await.unwrap_or_default();
            
            let detail = AnimeDetail {
                anime,
                tags,
                related_anime: RelatedAnime {
                    sequels: vec![],
                    prequels: vec![],
                    related: similar,
                },
            };
            
            (StatusCode::OK, Json(detail)).into_response()
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

// Request DTO for creating anime
#[derive(Debug, Deserialize)]
pub struct CreateAnimeRequest {
    pub title: String,
    pub synonyms: Vec<String>,
    pub sources: Vec<String>,
    pub episodes: u32,
    pub status: String,
    pub anime_type: String,
    pub anime_season: AnimeSeason,
    pub synopsis: String,
    pub poster_url: String,
    pub tags: Vec<String>,
}

// POST /api/anime handler
pub async fn create_anime(
    State(state): State<AppState>,
    Json(payload): Json<CreateAnimeRequest>,
) -> impl IntoResponse {
    // Parse anime type
    let anime_type = match payload.anime_type.as_str() {
        "TV" => AnimeType::TV,
        "MOVIE" => AnimeType::Movie,
        "OVA" => AnimeType::OVA,
        "ONA" => AnimeType::ONA,
        "SPECIAL" => AnimeType::Special,
        _ => AnimeType::Unknown,
    };
    
    // Parse status
    let status = match payload.status.as_str() {
        "FINISHED" => AnimeStatus::Finished,
        "ONGOING" => AnimeStatus::Ongoing,
        "UPCOMING" => AnimeStatus::Upcoming,
        _ => AnimeStatus::Unknown,
    };
    
    // Create anime model
    let anime = Anime {
        id: Uuid::new_v4(),
        title: payload.title,
        synonyms: payload.synonyms,
        sources: payload.sources,
        episodes: payload.episodes,
        status,
        anime_type,
        anime_season: payload.anime_season,
        synopsis: payload.synopsis,
        poster_url: payload.poster_url,
        imdb: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // Save to database
    match state.db.create_anime(&anime).await {
        Ok(_) => {
            // Also create tags if provided
            for tag_name in payload.tags {
                // Create tag
                let tag = crate::models::Tag::new(tag_name, crate::models::TagCategory::Genre);
                if let Ok(created_tag) = state.db.create_tag(&tag).await {
                    // Link tag to anime
                    let _ = state.db.link_anime_tag(anime.id, created_tag.id, Some(1.0)).await;
                }
            }
            
            (StatusCode::CREATED, Json(anime)).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to create anime: {}", e)
                }))
            ).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_get_anime_not_found() {
        let state = AppState::new("memory://", "redis://localhost", "secret".to_string())
            .await
            .unwrap();
        
        let app = crate::api::routes::create_router(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/anime/{}", Uuid::new_v4()))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}