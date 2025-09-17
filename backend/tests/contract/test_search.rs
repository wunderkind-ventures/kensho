// T009: Contract test GET /api/search
// Reference: contracts/openapi.yaml lines 46-77

use serde_json::json;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn search_returns_200_with_results() {
    // Arrange
    let app = spawn_app().await;
    
    // Create test anime to search for
    let anime_data = json!({
        "title": "Steins;Gate",
        "synonyms": ["シュタインズ・ゲート"],
        "sources": ["https://myanimelist.net/anime/9253/"],
        "episodes": 24,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
            "year": 2011
        },
        "synopsis": "A self-proclaimed mad scientist and his friends discover time travel",
        "poster_url": "https://example.com/steinsgate.jpg",
        "tags": ["Sci-Fi", "Thriller"]
    });
    
    let _create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to create anime");
    
    // Act
    let response = app.client
        .get(&format!("{}/api/search?q=Steins", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let search_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(search_results["results"].is_array());
    assert!(search_results["total"].is_number());
    
    let results = search_results["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Should find at least one result");
    
    // Verify first result matches our anime
    let first_result = &results[0];
    assert_eq!(first_result["title"].as_str().unwrap(), "Steins;Gate");
}

#[tokio::test]
async fn search_returns_empty_results_for_no_match() {
    // Arrange
    let app = spawn_app().await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/search?q=NonExistentAnime123456", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let search_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    let results = search_results["results"].as_array().unwrap();
    assert_eq!(results.len(), 0, "Should return empty results");
    assert_eq!(search_results["total"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn search_returns_400_without_query_param() {
    // Arrange
    let app = spawn_app().await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/search", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn search_response_matches_openapi_schema() {
    // Arrange
    let app = spawn_app().await;
    
    // Create multiple anime for search
    for i in 1..=3 {
        let anime_data = json!({
            "title": format!("Test Anime {}", i),
            "synonyms": [],
            "sources": [],
            "episodes": 12,
            "status": "FINISHED",
            "anime_type": "TV",
            "anime_season": {
                "season": "spring",
                "year": 2023
            },
            "synopsis": "Test anime for schema validation",
            "poster_url": format!("https://example.com/anime{}.jpg", i),
            "tags": []
        });
        
        let _create = app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await;
    }
    
    // Act
    let response = app.client
        .get(&format!("{}/api/search?q=Test", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - Check response structure matches OpenAPI spec (SearchResults schema)
    let search_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    // Required fields from SearchResults schema
    assert!(search_results["results"].is_array(), "results must be an array");
    assert!(search_results["total"].is_number(), "total must be a number");
    
    // Check AnimeSummary schema for each result
    let results = search_results["results"].as_array().unwrap();
    for result in results {
        assert!(result["id"].is_string(), "id must be a string");
        assert!(result["title"].is_string(), "title must be a string");
        assert!(result["poster_url"].is_string(), "poster_url must be a string");
        assert!(result["episodes"].is_number(), "episodes must be a number");
        assert!(result["status"].is_string(), "status must be a string");
        assert!(result["anime_type"].is_string(), "anime_type must be a string");
        // Optional field
        if result.get("imdb_rating").is_some() {
            assert!(result["imdb_rating"].is_number(), "imdb_rating must be a number if present");
        }
    }
}

#[tokio::test]
async fn search_filters_by_title_and_synonyms() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime with Japanese synonym
    let anime_data = json!({
        "title": "Death Note",
        "synonyms": ["デスノート", "DN"],
        "sources": [],
        "episodes": 37,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "fall",
            "year": 2006
        },
        "synopsis": "A genius student finds a notebook that kills",
        "poster_url": "https://example.com/deathnote.jpg",
        "tags": ["Psychological", "Supernatural"]
    });
    
    let _create = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await;
    
    // Act - Search by synonym
    let response = app.client
        .get(&format!("{}/api/search?q=デスノート", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    let search_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    let results = search_results["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Should find anime by synonym");
    assert_eq!(results[0]["title"].as_str().unwrap(), "Death Note");
}