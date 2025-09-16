// T008: Contract test for GET /api/anime/{id}
// Reference: contracts/openapi.yaml lines 24-44
// Reference: data-model.md "Get Anime with All Relationships" query

use axum::http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

mod common;
use common::*;

#[tokio::test]
async fn test_get_anime_by_id_success() {
    let app = setup_test_app().await;
    let anime_id = Uuid::new_v4();
    
    // Seed test data
    seed_test_anime(&app.db, anime_id).await;
    
    let response = app
        .client
        .get(&format!("/api/anime/{}", anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Verify response matches OpenAPI schema
    assert!(body["id"].is_string());
    assert!(body["title"].is_string());
    assert!(body["poster_url"].is_string());
    assert!(body["episodes"].is_number());
    assert!(body["status"].is_string());
    
    // Verify optional fields
    if body["imdb"].is_object() {
        assert!(body["imdb"]["rating"].is_number());
        assert!(body["imdb"]["votes"].is_number());
    }
    
    // Verify relationships
    assert!(body["tags"].is_array());
    assert!(body["related_anime"].is_object());
}

#[tokio::test]
async fn test_get_anime_by_id_not_found() {
    let app = setup_test_app().await;
    let non_existent_id = Uuid::new_v4();
    
    let response = app
        .client
        .get(&format!("/api/anime/{}", non_existent_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["error"].is_string());
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn test_get_anime_by_id_invalid_uuid() {
    let app = setup_test_app().await;
    
    let response = app
        .client
        .get("/api/anime/invalid-uuid")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_anime_with_all_relationships() {
    let app = setup_test_app().await;
    let anime_id = Uuid::new_v4();
    
    // Seed anime with relationships
    seed_anime_with_relationships(&app.db, anime_id).await;
    
    let response = app
        .client
        .get(&format!("/api/anime/{}", anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Verify tags are included
    let tags = body["tags"].as_array().expect("Tags should be array");
    assert!(!tags.is_empty());
    
    // Verify related anime
    let related = &body["related_anime"];
    assert!(related["sequels"].is_array());
    assert!(related["prequels"].is_array());
    assert!(related["related"].is_array());
}

// Helper functions that will be implemented in common module
async fn seed_test_anime(db: &surrealdb::Surreal<surrealdb::engine::any::Any>, id: Uuid) {
    // This will fail until implementation
    panic!("Not implemented - test should fail");
}

async fn seed_anime_with_relationships(db: &surrealdb::Surreal<surrealdb::engine::any::Any>, id: Uuid) {
    // This will fail until implementation
    panic!("Not implemented - test should fail");
}