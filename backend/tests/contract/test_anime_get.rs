// T008: Contract test GET /api/anime/{id}
// Reference: contracts/openapi.yaml lines 24-44

use serde_json::json;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn get_anime_returns_200_with_valid_id() {
    // Arrange
    let app = spawn_app().await;
    
    // First create an anime
    let anime_data = json!({
        "title": "Test Anime",
        "synonyms": ["テストアニメ"],
        "sources": ["https://myanimelist.net/anime/1/"],
        "episodes": 12,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
            "year": 2023
        },
        "synopsis": "A test anime for contract testing",
        "poster_url": "https://example.com/poster.jpg",
        "tags": ["Action", "Comedy"]
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(create_response.status().as_u16(), 201);
    let created_anime: serde_json::Value = create_response.json().await.expect("Failed to parse response");
    let anime_id = created_anime["id"].as_str().expect("ID should be string");
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}", app.address, anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let anime_detail: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(anime_detail["id"].as_str().unwrap(), anime_id);
    assert_eq!(anime_detail["title"].as_str().unwrap(), "Test Anime");
    assert_eq!(anime_detail["episodes"].as_u64().unwrap(), 12);
}

#[tokio::test]
async fn get_anime_returns_404_with_invalid_id() {
    // Arrange
    let app = spawn_app().await;
    let non_existent_id = Uuid::new_v4();
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}", app.address, non_existent_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    
    let error_response: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(error_response["error"].as_str().is_some());
}

#[tokio::test]
async fn get_anime_returns_400_with_invalid_uuid() {
    // Arrange
    let app = spawn_app().await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/not-a-valid-uuid", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn get_anime_response_matches_openapi_schema() {
    // Arrange
    let app = spawn_app().await;
    
    // Create test anime
    let anime_data = json!({
        "title": "Schema Test Anime",
        "synonyms": ["Synonym1", "Synonym2"],
        "sources": ["source1", "source2"],
        "episodes": 24,
        "status": "ONGOING",
        "anime_type": "TV",
        "anime_season": {
            "season": "fall",
            "year": 2024
        },
        "synopsis": "Testing OpenAPI schema compliance",
        "poster_url": "https://example.com/schema-test.jpg",
        "tags": ["Drama", "Sci-Fi"]
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to send request");
    
    let created_anime: serde_json::Value = create_response.json().await.expect("Failed to parse response");
    let anime_id = created_anime["id"].as_str().expect("ID should be string");
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}", app.address, anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - Check response structure matches OpenAPI spec
    let anime_detail: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    // Required fields from AnimeDetail schema
    assert!(anime_detail["id"].is_string(), "id must be a string (UUID)");
    assert!(anime_detail["title"].is_string(), "title must be a string");
    assert!(anime_detail["synonyms"].is_array(), "synonyms must be an array");
    assert!(anime_detail["sources"].is_array(), "sources must be an array");
    assert!(anime_detail["episodes"].is_number(), "episodes must be a number");
    assert!(anime_detail["status"].is_string(), "status must be a string");
    assert!(anime_detail["type"].is_string(), "type must be a string");
    assert!(anime_detail["anime_season"].is_object(), "anime_season must be an object");
    assert!(anime_detail["anime_season"]["season"].is_string(), "season must be a string");
    assert!(anime_detail["anime_season"]["year"].is_number(), "year must be a number");
    assert!(anime_detail["synopsis"].is_string(), "synopsis must be a string");
    assert!(anime_detail["poster_url"].is_string(), "poster_url must be a string");
    assert!(anime_detail["tags"].is_array(), "tags must be an array");
    assert!(anime_detail["related_anime"].is_object(), "related_anime must be an object");
}