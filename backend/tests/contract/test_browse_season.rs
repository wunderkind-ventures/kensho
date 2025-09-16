// T010: Contract test for GET /api/browse/season/{year}/{season}
// Reference: contracts/openapi.yaml lines 79-117
// Reference: data-model.md "Browse by Season" query

use axum::http::StatusCode;
use serde_json::Value;

mod common;
use common::*;

#[tokio::test]
async fn test_browse_season_success() {
    let app = setup_test_app().await;
    
    // Seed anime for Fall 2024
    seed_seasonal_anime(&app.db, 2024, "fall", 15).await;
    
    let response = app
        .client
        .get("/api/browse/season/2024/fall")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Verify response structure
    assert!(body["anime"].is_array());
    assert!(body["pagination"].is_object());
    
    let anime_list = body["anime"].as_array().expect("Anime should be array");
    assert_eq!(anime_list.len(), 15);
    
    // Verify AnimeSummary schema for each item
    for anime in anime_list {
        assert!(anime["id"].is_string());
        assert!(anime["title"].is_string());
        assert!(anime["poster_url"].is_string());
        assert!(anime["episodes"].is_number());
        assert!(anime["status"].is_string());
        assert!(anime["anime_type"].is_string());
    }
    
    // Verify pagination
    let pagination = &body["pagination"];
    assert!(pagination["page"].is_number());
    assert!(pagination["limit"].is_number());
    assert!(pagination["total_pages"].is_number());
    assert!(pagination["total_items"].is_number());
}

#[tokio::test]
async fn test_browse_season_sorted_by_rating() {
    let app = setup_test_app().await;
    
    // Seed anime with different ratings
    seed_seasonal_anime_with_ratings(&app.db, 2024, "spring").await;
    
    let response = app
        .client
        .get("/api/browse/season/2024/spring")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let anime_list = body["anime"].as_array().expect("Anime should be array");
    
    // Verify sorted by IMDb rating (highest first)
    let mut prev_rating = 10.0;
    for anime in anime_list {
        if let Some(rating) = anime["imdb_rating"].as_f64() {
            assert!(rating <= prev_rating, "Results should be sorted by rating DESC");
            prev_rating = rating;
        }
    }
}

#[tokio::test]
async fn test_browse_season_with_pagination() {
    let app = setup_test_app().await;
    
    // Seed 100 anime for summer 2024
    seed_seasonal_anime(&app.db, 2024, "summer", 100).await;
    
    // Request first page
    let response = app
        .client
        .get("/api/browse/season/2024/summer?page=1&limit=20")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let anime_list = body["anime"].as_array().expect("Anime should be array");
    
    assert_eq!(anime_list.len(), 20);
    assert_eq!(body["pagination"]["page"].as_u64(), Some(1));
    assert_eq!(body["pagination"]["limit"].as_u64(), Some(20));
    assert_eq!(body["pagination"]["total_items"].as_u64(), Some(100));
    assert_eq!(body["pagination"]["total_pages"].as_u64(), Some(5));
}

#[tokio::test]
async fn test_browse_all_seasons() {
    let app = setup_test_app().await;
    
    let seasons = ["spring", "summer", "fall", "winter"];
    
    for season in &seasons {
        let response = app
            .client
            .get(&format!("/api/browse/season/2024/{}", season))
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(
            response.status(), 
            StatusCode::OK,
            "Should accept season: {}", 
            season
        );
    }
}

#[tokio::test]
async fn test_browse_invalid_season() {
    let app = setup_test_app().await;
    
    let response = app
        .client
        .get("/api/browse/season/2024/invalid")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn test_browse_invalid_year() {
    let app = setup_test_app().await;
    
    // Year too old
    let response = app
        .client
        .get("/api/browse/season/1899/fall")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Year too far in future
    let response = app
        .client
        .get("/api/browse/season/2031/fall")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_browse_empty_season() {
    let app = setup_test_app().await;
    
    // Don't seed any data for winter 2023
    let response = app
        .client
        .get("/api/browse/season/2023/winter")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let anime_list = body["anime"].as_array().expect("Anime should be array");
    
    assert!(anime_list.is_empty());
    assert_eq!(body["pagination"]["total_items"].as_u64(), Some(0));
}

// Helper functions that will be implemented later
async fn seed_seasonal_anime(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>, 
    year: u16, 
    season: &str, 
    count: usize
) {
    panic!("Not implemented - test should fail");
}

async fn seed_seasonal_anime_with_ratings(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    year: u16,
    season: &str
) {
    panic!("Not implemented - test should fail");
}