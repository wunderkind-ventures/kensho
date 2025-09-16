// T009: Contract test for GET /api/search
// Reference: contracts/openapi.yaml lines 46-77
// Reference: data-model.md "Search by Title or Synonym" query

use axum::http::StatusCode;
use serde_json::Value;

mod common;
use common::*;

#[tokio::test]
async fn test_search_anime_by_title() {
    let app = setup_test_app().await;
    
    // Seed test data with searchable anime
    seed_searchable_anime(&app.db).await;
    
    let response = app
        .client
        .get("/api/search?q=Attack%20on%20Titan")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Verify response structure matches OpenAPI spec
    assert!(body["results"].is_array());
    assert!(body["total"].is_number());
    
    let results = body["results"].as_array().expect("Results should be array");
    assert!(!results.is_empty(), "Should find at least one result");
    
    // Verify AnimeSummary schema
    let first_result = &results[0];
    assert!(first_result["id"].is_string());
    assert!(first_result["title"].is_string());
    assert!(first_result["poster_url"].is_string());
    assert!(first_result["episodes"].is_number());
    assert!(first_result["status"].is_string());
}

#[tokio::test]
async fn test_search_anime_by_synonym() {
    let app = setup_test_app().await;
    
    // Seed anime with Japanese title as synonym
    seed_anime_with_synonyms(&app.db).await;
    
    let response = app
        .client
        .get("/api/search?q=Shingeki%20no%20Kyojin")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let results = body["results"].as_array().expect("Results should be array");
    
    // Should find anime by synonym
    assert!(!results.is_empty(), "Should find anime by synonym");
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let app = setup_test_app().await;
    
    seed_searchable_anime(&app.db).await;
    
    // Test with different cases
    for query in &["spy x family", "SPY X FAMILY", "Spy X Family"] {
        let response = app
            .client
            .get(&format!("/api/search?q={}", urlencoding::encode(query)))
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body: Value = response.json().await.expect("Failed to parse JSON");
        let results = body["results"].as_array().expect("Results should be array");
        assert!(!results.is_empty(), "Search should be case-insensitive for: {}", query);
    }
}

#[tokio::test]
async fn test_search_with_limit() {
    let app = setup_test_app().await;
    
    // Seed many anime
    seed_many_anime(&app.db, 50).await;
    
    let response = app
        .client
        .get("/api/search?q=anime&limit=10")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let results = body["results"].as_array().expect("Results should be array");
    
    assert_eq!(results.len(), 10, "Should respect limit parameter");
}

#[tokio::test]
async fn test_search_empty_query() {
    let app = setup_test_app().await;
    
    let response = app
        .client
        .get("/api/search?q=")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn test_search_query_too_short() {
    let app = setup_test_app().await;
    
    let response = app
        .client
        .get("/api/search?q=a")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["error"].as_str(), Some("Invalid search parameters"));
}

#[tokio::test]
async fn test_search_no_results() {
    let app = setup_test_app().await;
    
    let response = app
        .client
        .get("/api/search?q=nonexistentanime12345")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let results = body["results"].as_array().expect("Results should be array");
    
    assert!(results.is_empty(), "Should return empty results for non-existent anime");
    assert_eq!(body["total"].as_u64(), Some(0));
}

// Helper functions that will be implemented later
async fn seed_searchable_anime(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) {
    panic!("Not implemented - test should fail");
}

async fn seed_anime_with_synonyms(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) {
    panic!("Not implemented - test should fail");
}

async fn seed_many_anime(db: &surrealdb::Surreal<surrealdb::engine::any::Any>, count: usize) {
    panic!("Not implemented - test should fail");
}