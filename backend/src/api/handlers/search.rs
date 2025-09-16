// T035: GET /api/search handler
// Reference: contracts/openapi.yaml lines 46-77

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use crate::db::connection::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_limit() -> usize {
    20
}

pub async fn search(
    Query(params): Query<SearchParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Perform search
    match state.search.search_anime(&params.q).await {
        Ok(mut results) => {
            // Apply pagination
            let total = results.len();
            results = results
                .into_iter()
                .skip(params.offset)
                .take(params.limit)
                .collect();
            
            (
                StatusCode::OK,
                Json(json!({
                    "results": results,
                    "total": total,
                    "offset": params.offset,
                    "limit": params.limit
                }))
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Search failed: {}", e)
                }))
            ).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 20);
    }
}