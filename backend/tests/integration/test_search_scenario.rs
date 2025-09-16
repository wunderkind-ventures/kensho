// T016: Integration test "Search for SPY x FAMILY" scenario
// Reference: quickstart.md "Test 1: Search and Discovery" section
// Reference: spec.md Acceptance Scenario 1

use serde_json::Value;
use uuid::Uuid;

mod common;
use common::*;

#[tokio::test]
async fn test_spy_family_search_scenario() {
    // Setup: Initialize app with test data
    let app = setup_integration_test().await;
    
    // Seed SPY x FAMILY anime data
    let spy_family_id = seed_spy_family_anime(&app.db).await;
    
    // Step 1: User searches for "SPY x FAMILY"
    let search_response = app
        .client
        .get("/api/search?q=SPY%20x%20FAMILY")
        .send()
        .await
        .expect("Failed to search");
    
    assert_eq!(search_response.status(), 200);
    
    let search_results: Value = search_response
        .json()
        .await
        .expect("Failed to parse search results");
    
    // Verify: Search results show SPY x FAMILY
    let results = search_results["results"]
        .as_array()
        .expect("Results should be array");
    
    assert!(!results.is_empty(), "Should find SPY x FAMILY");
    
    let spy_family = &results[0];
    assert_eq!(spy_family["title"].as_str(), Some("SPY x FAMILY"));
    assert!(spy_family["poster_url"].is_string());
    
    // Verify IMDb rating is displayed if available
    if spy_family["imdb_rating"].is_number() {
        let rating = spy_family["imdb_rating"].as_f64().unwrap();
        assert!(rating > 0.0 && rating <= 10.0);
    }
    
    // Step 2: User clicks on the series to navigate to IP Hub
    let anime_id = spy_family["id"].as_str().expect("Should have ID");
    
    let ip_hub_response = app
        .client
        .get(&format!("/api/anime/{}", anime_id))
        .send()
        .await
        .expect("Failed to get anime details");
    
    assert_eq!(ip_hub_response.status(), 200);
    
    let anime_details: Value = ip_hub_response
        .json()
        .await
        .expect("Failed to parse anime details");
    
    // Verify: IP Hub shows all required information
    assert_eq!(anime_details["title"].as_str(), Some("SPY x FAMILY"));
    assert!(anime_details["synopsis"].is_string());
    assert!(anime_details["poster_url"].is_string());
    assert_eq!(anime_details["episodes"].as_u64(), Some(25)); // Season 1 episodes
    
    // Verify tags are included
    assert!(anime_details["tags"].is_array());
    
    // Verify related anime if any
    assert!(anime_details["related_anime"].is_object());
}

#[tokio::test]
async fn test_search_case_insensitive_variations() {
    let app = setup_integration_test().await;
    
    seed_spy_family_anime(&app.db).await;
    
    // Test various case variations
    let variations = [
        "spy x family",
        "SPY X FAMILY",
        "Spy X Family",
        "sPy X fAmIlY",
    ];
    
    for query in &variations {
        let response = app
            .client
            .get(&format!("/api/search?q={}", urlencoding::encode(query)))
            .send()
            .await
            .expect("Failed to search");
        
        assert_eq!(response.status(), 200);
        
        let results: Value = response.json().await.expect("Failed to parse");
        let items = results["results"].as_array().expect("Should be array");
        
        assert!(
            !items.is_empty(),
            "Should find SPY x FAMILY with query: {}",
            query
        );
    }
}

#[tokio::test]
async fn test_search_and_navigation_flow() {
    let app = setup_integration_test().await;
    
    // Seed multiple anime
    seed_multiple_anime(&app.db).await;
    
    // Search for action anime
    let search_response = app
        .client
        .get("/api/search?q=action")
        .send()
        .await
        .expect("Failed to search");
    
    assert_eq!(search_response.status(), 200);
    
    let search_results: Value = search_response.json().await.expect("Failed to parse");
    let results = search_results["results"].as_array().expect("Should be array");
    
    assert!(results.len() > 0, "Should find action anime");
    
    // Navigate to each result
    for result in results.iter().take(3) {
        let anime_id = result["id"].as_str().expect("Should have ID");
        
        let detail_response = app
            .client
            .get(&format!("/api/anime/{}", anime_id))
            .send()
            .await
            .expect("Failed to get details");
        
        assert_eq!(detail_response.status(), 200);
    }
}

// Helper functions
async fn setup_integration_test() -> TestApp {
    panic!("Not implemented - test should fail");
}

async fn seed_spy_family_anime(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>
) -> Uuid {
    panic!("Not implemented - test should fail");
}

async fn seed_multiple_anime(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>
) {
    panic!("Not implemented - test should fail");
}

struct TestApp {
    client: reqwest::Client,
    db: surrealdb::Surreal<surrealdb::engine::any::Any>,
}