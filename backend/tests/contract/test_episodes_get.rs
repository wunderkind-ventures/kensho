// T011: Contract test for GET /api/anime/{id}/episodes
// Reference: contracts/openapi.yaml lines 119-143
// Reference: data-model.md "Get Episodes for Anime" query

use axum::http::StatusCode;
use serde_json::Value;
use uuid::Uuid;

mod common;
use common::*;

#[tokio::test]
async fn test_get_episodes_success() {
    let app = setup_test_app().await;
    let anime_id = Uuid::new_v4();
    
    // Seed anime with 24 episodes
    seed_anime_with_episodes(&app.db, anime_id, 24).await;
    
    let response = app
        .client
        .get(&format!("/api/anime/{}/episodes", anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Verify response structure
    assert!(body["episodes"].is_array());
    assert!(body["total"].is_number());
    
    let episodes = body["episodes"].as_array().expect("Episodes should be array");
    assert_eq!(episodes.len(), 24);
    assert_eq!(body["total"].as_u64(), Some(24));
    
    // Verify Episode schema
    for (i, episode) in episodes.iter().enumerate() {
        assert!(episode["id"].is_string());
        assert_eq!(episode["episode_number"].as_u64(), Some((i + 1) as u64));
        
        // Optional fields
        if episode["title"].is_string() {
            assert!(!episode["title"].as_str().unwrap().is_empty());
        }
        if episode["duration"].is_number() {
            assert!(episode["duration"].as_u64().unwrap() > 0);
        }
    }
}

#[tokio::test]
async fn test_get_episodes_sorted_by_number() {
    let app = setup_test_app().await;
    let anime_id = Uuid::new_v4();
    
    // Seed episodes out of order
    seed_anime_with_shuffled_episodes(&app.db, anime_id, 12).await;
    
    let response = app
        .client
        .get(&format!("/api/anime/{}/episodes", anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let episodes = body["episodes"].as_array().expect("Episodes should be array");
    
    // Verify episodes are sorted by episode_number
    for i in 0..episodes.len() {
        assert_eq!(
            episodes[i]["episode_number"].as_u64(),
            Some((i + 1) as u64),
            "Episodes should be sorted by episode_number"
        );
    }
}

#[tokio::test]
async fn test_get_episodes_with_metadata() {
    let app = setup_test_app().await;
    let anime_id = Uuid::new_v4();
    
    // Seed episodes with full metadata
    seed_episodes_with_metadata(&app.db, anime_id).await;
    
    let response = app
        .client
        .get(&format!("/api/anime/{}/episodes", anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let episodes = body["episodes"].as_array().expect("Episodes should be array");
    
    // Check first episode has all optional fields
    let first = &episodes[0];
    assert!(first["title"].is_string());
    assert!(first["duration"].is_number());
    assert!(first["air_date"].is_string());
    assert!(first["synopsis"].is_string());
    assert!(first["thumbnail_url"].is_string());
}

#[tokio::test]
async fn test_get_episodes_anime_not_found() {
    let app = setup_test_app().await;
    let non_existent_id = Uuid::new_v4();
    
    let response = app
        .client
        .get(&format!("/api/anime/{}/episodes", non_existent_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["error"].is_string());
    assert_eq!(body["message"].as_str(), Some("Anime not found"));
}

#[tokio::test]
async fn test_get_episodes_empty_list() {
    let app = setup_test_app().await;
    let anime_id = Uuid::new_v4();
    
    // Seed anime with no episodes (e.g., upcoming anime)
    seed_anime_without_episodes(&app.db, anime_id).await;
    
    let response = app
        .client
        .get(&format!("/api/anime/{}/episodes", anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let episodes = body["episodes"].as_array().expect("Episodes should be array");
    
    assert!(episodes.is_empty());
    assert_eq!(body["total"].as_u64(), Some(0));
}

#[tokio::test]
async fn test_get_episodes_invalid_uuid() {
    let app = setup_test_app().await;
    
    let response = app
        .client
        .get("/api/anime/invalid-uuid/episodes")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Helper functions that will be implemented later
async fn seed_anime_with_episodes(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    anime_id: Uuid,
    episode_count: usize
) {
    panic!("Not implemented - test should fail");
}

async fn seed_anime_with_shuffled_episodes(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    anime_id: Uuid,
    episode_count: usize
) {
    panic!("Not implemented - test should fail");
}

async fn seed_episodes_with_metadata(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    anime_id: Uuid
) {
    panic!("Not implemented - test should fail");
}

async fn seed_anime_without_episodes(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    anime_id: Uuid
) {
    panic!("Not implemented - test should fail");
}