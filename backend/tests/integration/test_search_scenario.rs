// T016: Integration test "Search for SPY x FAMILY" scenario
// Reference: quickstart.md "Test 1: Search and Discovery" section
// Reference: spec.md Acceptance Scenario 1

use serde_json::json;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn user_can_search_and_discover_anime() {
    // Arrange - Set up test environment with realistic data
    let app = spawn_app().await;
    
    // Create SPY x FAMILY anime
    let spy_family_data = json!({
        "title": "SPY x FAMILY",
        "synonyms": ["SPY×FAMILY", "スパイファミリー"],
        "sources": ["https://myanimelist.net/anime/50265/"],
        "episodes": 12,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
            "year": 2022
        },
        "synopsis": "A spy, an assassin, and a telepath form a fake family",
        "poster_url": "https://example.com/spyxfamily.jpg",
        "tags": ["Action", "Comedy", "Shounen"]
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&spy_family_data)
        .send()
        .await
        .expect("Failed to create SPY x FAMILY");
    
    assert_eq!(create_response.status().as_u16(), 201);
    let created_anime: serde_json::Value = create_response.json().await.unwrap();
    let anime_id = created_anime["id"].as_str().unwrap();
    
    // Create episodes for SPY x FAMILY
    let episodes_data = json!({
        "episodes": [
            {
                "episode_number": 1,
                "title": "Operation Strix",
                "duration": 1440,
                "synopsis": "Twilight receives his mission"
            },
            {
                "episode_number": 2,
                "title": "Secure a Wife",
                "duration": 1440,
                "synopsis": "Loid meets Yor"
            },
            {
                "episode_number": 3,
                "title": "Prepare for the Interview",
                "duration": 1440,
                "synopsis": "The family prepares for Eden Academy"
            }
        ]
    });
    
    let episodes_response = app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    // Act - User searches for "SPY x FAMILY"
    let search_response = app.client
        .get(&format!("{}/api/search?q=SPY", app.address))
        .send()
        .await
        .expect("Failed to search");
    
    // Assert - User finds the anime
    assert_eq!(search_response.status().as_u16(), 200);
    
    let search_results: serde_json::Value = search_response.json().await.unwrap();
    let results = search_results["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Should find SPY x FAMILY");
    assert_eq!(results[0]["title"].as_str().unwrap(), "SPY x FAMILY");
    
    // Act - User clicks on the anime to view details
    let detail_response = app.client
        .get(&format!("{}/api/anime/{}", app.address, anime_id))
        .send()
        .await
        .expect("Failed to get anime details");
    
    // Assert - User sees full details
    assert_eq!(detail_response.status().as_u16(), 200);
    
    let anime_detail: serde_json::Value = detail_response.json().await.unwrap();
    assert_eq!(anime_detail["title"].as_str().unwrap(), "SPY x FAMILY");
    assert_eq!(anime_detail["episodes"].as_u64().unwrap(), 12);
    assert!(anime_detail["synopsis"].as_str().unwrap().contains("fake family"));
    
    // Act - User checks episodes
    let episodes_response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to get episodes");
    
    // Assert - User sees episode list
    assert_eq!(episodes_response.status().as_u16(), 200);
    
    let episodes_list: serde_json::Value = episodes_response.json().await.unwrap();
    let episodes = episodes_list["episodes"].as_array().unwrap();
    assert!(episodes.len() >= 3, "Should have at least 3 episodes");
    assert_eq!(episodes[0]["title"].as_str().unwrap(), "Operation Strix");
}

#[tokio::test]
async fn user_can_search_by_japanese_title() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime with Japanese title
    let anime_data = json!({
        "title": "Your Name",
        "synonyms": ["君の名は。", "Kimi no Na wa"],
        "sources": ["https://myanimelist.net/anime/32281/"],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "MOVIE",
        "anime_season": {
            "season": "summer",
            "year": 2016
        },
        "synopsis": "Two teenagers share a profound, magical connection",
        "poster_url": "https://example.com/yourname.jpg",
        "tags": ["Romance", "Supernatural", "School"]
    });
    
    let _create = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await;
    
    // Act - Search by Japanese title
    let response = app.client
        .get(&format!("{}/api/search?q=君の名は", app.address))
        .send()
        .await
        .expect("Failed to search");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let search_results: serde_json::Value = response.json().await.unwrap();
    let results = search_results["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Should find anime by Japanese title");
    assert_eq!(results[0]["title"].as_str().unwrap(), "Your Name");
}

#[tokio::test]
async fn user_can_discover_anime_by_partial_title() {
    // Arrange
    let app = spawn_app().await;
    
    // Create multiple related anime
    let attack_on_titan_titles = vec![
        ("Attack on Titan", 2013, "spring"),
        ("Attack on Titan Season 2", 2017, "spring"),
        ("Attack on Titan Season 3", 2018, "summer"),
        ("Attack on Titan Final Season", 2020, "winter"),
    ];
    
    for (title, year, season) in attack_on_titan_titles {
        let anime_data = json!({
            "title": title,
            "synonyms": ["Shingeki no Kyojin"],
            "sources": [],
            "episodes": 12,
            "status": "FINISHED",
            "anime_type": "TV",
            "anime_season": {
                "season": season,
                "year": year
            },
            "synopsis": "Humanity fights against titans",
            "poster_url": format!("https://example.com/{}.jpg", title.to_lowercase().replace(" ", "_")),
            "tags": ["Action", "Drama", "Fantasy"]
        });
        
        let _create = app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await;
    }
    
    // Act - Search with partial title
    let response = app.client
        .get(&format!("{}/api/search?q=Attack", app.address))
        .send()
        .await
        .expect("Failed to search");
    
    // Assert - Should find all Attack on Titan series
    assert_eq!(response.status().as_u16(), 200);
    
    let search_results: serde_json::Value = response.json().await.unwrap();
    let results = search_results["results"].as_array().unwrap();
    assert_eq!(results.len(), 4, "Should find all 4 Attack on Titan series");
    
    // Verify all results are Attack on Titan related
    for result in results {
        assert!(
            result["title"].as_str().unwrap().contains("Attack on Titan"),
            "All results should be Attack on Titan series"
        );
    }
}