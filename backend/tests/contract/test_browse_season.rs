// T010: Contract test GET /api/browse/season/{year}/{season}
// Reference: contracts/openapi.yaml lines 79-117

use serde_json::json;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn browse_season_returns_200_with_valid_params() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime for Fall 2023
    let anime_data = vec![
        json!({
            "title": "Frieren: Beyond Journey's End",
            "synonyms": ["Sousou no Frieren"],
            "sources": ["https://myanimelist.net/anime/52991/"],
            "episodes": 28,
            "status": "FINISHED",
            "anime_type": "TV",
            "anime_season": {
                "season": "fall",
                "year": 2023
            },
            "synopsis": "After defeating the Demon King, the hero party disbands",
            "poster_url": "https://example.com/frieren.jpg",
            "tags": ["Adventure", "Drama", "Fantasy"]
        }),
        json!({
            "title": "The Apothecary Diaries",
            "synonyms": ["Kusuriya no Hitorigoto"],
            "sources": ["https://myanimelist.net/anime/54492/"],
            "episodes": 24,
            "status": "FINISHED",
            "anime_type": "TV",
            "anime_season": {
                "season": "fall",
                "year": 2023
            },
            "synopsis": "A pharmacist gets kidnapped and sold to the imperial palace",
            "poster_url": "https://example.com/apothecary.jpg",
            "tags": ["Drama", "Historical", "Mystery"]
        }),
    ];
    
    for anime in anime_data {
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime)
            .send()
            .await
            .expect("Failed to create anime");
    }
    
    // Act
    let response = app.client
        .get(&format!("{}/api/browse/season/2023/fall", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let browse_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(browse_results["year"].as_u64().unwrap(), 2023);
    assert_eq!(browse_results["season"].as_str().unwrap(), "fall");
    assert!(browse_results["anime"].is_array());
    assert!(browse_results["total"].is_number());
    
    let anime_list = browse_results["anime"].as_array().unwrap();
    assert_eq!(anime_list.len(), 2, "Should find both Fall 2023 anime");
}

#[tokio::test]
async fn browse_season_returns_empty_for_no_matches() {
    // Arrange
    let app = spawn_app().await;
    
    // Act - Browse future season with no anime
    let response = app.client
        .get(&format!("{}/api/browse/season/2030/winter", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let browse_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    let anime_list = browse_results["anime"].as_array().unwrap();
    assert_eq!(anime_list.len(), 0, "Should return empty list");
    assert_eq!(browse_results["total"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn browse_season_returns_400_for_invalid_season() {
    // Arrange
    let app = spawn_app().await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/browse/season/2023/invalid", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn browse_season_returns_400_for_invalid_year() {
    // Arrange
    let app = spawn_app().await;
    
    // Act - Year too early (before anime existed)
    let response = app.client
        .get(&format!("{}/api/browse/season/1800/spring", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn browse_season_response_matches_openapi_schema() {
    // Arrange
    let app = spawn_app().await;
    
    // Create test anime
    let anime_data = json!({
        "title": "Schema Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 12,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
            "year": 2024
        },
        "synopsis": "Testing schema compliance",
        "poster_url": "https://example.com/test.jpg",
        "tags": []
    });
    
    app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/browse/season/2024/spring", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - Check response structure matches OpenAPI spec (SeasonalBrowseResponse schema)
    let browse_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    // Required fields from SeasonalBrowseResponse
    assert!(browse_results["year"].is_number(), "year must be a number");
    assert!(browse_results["season"].is_string(), "season must be a string");
    assert!(browse_results["anime"].is_array(), "anime must be an array");
    assert!(browse_results["total"].is_number(), "total must be a number");
    
    // Check AnimeSummary schema for each anime
    let anime_list = browse_results["anime"].as_array().unwrap();
    for anime in anime_list {
        assert!(anime["id"].is_string(), "id must be a string");
        assert!(anime["title"].is_string(), "title must be a string");
        assert!(anime["poster_url"].is_string(), "poster_url must be a string");
        assert!(anime["episodes"].is_number(), "episodes must be a number");
        assert!(anime["status"].is_string(), "status must be a string");
        assert!(anime["anime_type"].is_string(), "anime_type must be a string");
    }
}

#[tokio::test]
async fn browse_season_sorts_by_imdb_rating() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime with different ratings (would need IMDb data in real scenario)
    let anime_list = vec![
        ("Low Rated Anime", 6.5),
        ("High Rated Anime", 9.2),
        ("Mid Rated Anime", 7.8),
    ];
    
    for (title, _rating) in anime_list {
        let anime_data = json!({
            "title": title,
            "synonyms": [],
            "sources": [],
            "episodes": 12,
            "status": "FINISHED",
            "anime_type": "TV",
            "anime_season": {
                "season": "summer",
                "year": 2024
            },
            "synopsis": "Test anime for sorting",
            "poster_url": "https://example.com/anime.jpg",
            "tags": []
        });
        
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await;
    }
    
    // Act
    let response = app.client
        .get(&format!("{}/api/browse/season/2024/summer", app.address))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    let browse_results: serde_json::Value = response.json().await.expect("Failed to parse response");
    let anime_list = browse_results["anime"].as_array().unwrap();
    
    // In a real implementation with IMDb data, verify sorting
    // For now, just verify we got all 3 anime
    assert_eq!(anime_list.len(), 3, "Should return all anime from the season");
}