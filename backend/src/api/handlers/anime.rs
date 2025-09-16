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
use crate::db::connection::AppState;
use crate::models::{AnimeDetail, RelatedAnime};

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