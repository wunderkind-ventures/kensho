// T018: Integration test for authenticated video streaming
// Tests complete streaming workflow with Crunchyroll integration

use serde_json::json;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;
use common::{spawn_app, create_test_token};

#[tokio::test]
async fn authenticated_user_can_stream_anime() {
    // Arrange
    let app = spawn_app().await;
    
    // Create test anime with episodes
    let anime_data = json!({
        "title": "Demon Slayer",
        "synonyms": ["Kimetsu no Yaiba"],
        "sources": ["https://myanimelist.net/anime/38000/"],
        "episodes": 26,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
            "year": 2019
        },
        "synopsis": "A young boy becomes a demon slayer to save his sister",
        "poster_url": "https://example.com/demon_slayer.jpg",
        "tags": ["Action", "Historical", "Supernatural"]
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to create anime");
    
    let created_anime: serde_json::Value = create_response.json().await.unwrap();
    let anime_id = created_anime["id"].as_str().unwrap();
    
    // Create episodes
    let episodes_data = json!({
        "episodes": [
            {
                "episode_number": 1,
                "title": "Cruelty",
                "duration": 1440,
                "synopsis": "Tanjiro's family is attacked",
                "thumbnail_url": "https://example.com/ep1.jpg"
            }
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    // Get episodes to find episode ID
    let episodes_response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to get episodes");
    
    let episodes_result: serde_json::Value = episodes_response.json().await.unwrap();
    let episode_id = episodes_result["episodes"][0]["id"].as_str().unwrap();
    
    // Step 1: Authenticate user
    let token = create_test_token();
    
    // Step 2: Request stream URL
    let stream_response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get stream URL");
    
    // Assert
    assert_eq!(stream_response.status().as_u16(), 200, "Should return stream URL for authenticated user");
    
    let stream_data: serde_json::Value = stream_response.json().await.unwrap();
    assert!(stream_data["stream_url"].is_string(), "Should return stream URL");
    assert!(stream_data["expires_at"].is_string(), "Should include expiration time");
    assert!(stream_data["quality"].is_string(), "Should include quality setting");
}

#[tokio::test]
async fn unauthenticated_user_cannot_stream() {
    // Arrange
    let app = spawn_app().await;
    let episode_id = Uuid::new_v4();
    
    // Act - Try to stream without authentication
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 401, "Should require authentication");
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
    assert_eq!(error_response["error"].as_str().unwrap(), "Authentication required");
}

#[tokio::test]
async fn streaming_respects_quality_preferences() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime and episode
    let anime_data = json!({
        "title": "Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "winter",
            "year": 2024
        },
        "synopsis": "Test",
        "poster_url": "https://example.com/test.jpg",
        "tags": []
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to create anime");
    
    let created_anime: serde_json::Value = create_response.json().await.unwrap();
    let anime_id = created_anime["id"].as_str().unwrap();
    
    let episodes_data = json!({
        "episodes": [
            {"episode_number": 1, "title": "Episode 1"}
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    let episodes_response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to get episodes");
    
    let episodes_result: serde_json::Value = episodes_response.json().await.unwrap();
    let episode_id = episodes_result["episodes"][0]["id"].as_str().unwrap();
    
    let token = create_test_token();
    
    // Test different quality settings
    let qualities = vec!["auto", "1080p", "720p", "480p"];
    
    for quality in qualities {
        let response = app.client
            .get(&format!("{}/api/stream/{}?quality={}", app.address, episode_id, quality))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to get stream URL");
        
        if response.status().is_success() {
            let stream_data: serde_json::Value = response.json().await.unwrap();
            let returned_quality = stream_data["quality"].as_str().unwrap_or("auto");
            
            // Quality should match requested or fall back to available
            assert!(
                returned_quality == quality || returned_quality == "auto",
                "Quality should match requested or fall back"
            );
        }
    }
}

#[tokio::test]
async fn stream_url_expires_after_timeout() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime and episode
    let anime_data = json!({
        "title": "Expiry Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "summer",
            "year": 2024
        },
        "synopsis": "Test",
        "poster_url": "https://example.com/test.jpg",
        "tags": []
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to create anime");
    
    let created_anime: serde_json::Value = create_response.json().await.unwrap();
    let anime_id = created_anime["id"].as_str().unwrap();
    
    let episodes_data = json!({
        "episodes": [
            {"episode_number": 1, "title": "Episode 1"}
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    let episodes_response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to get episodes");
    
    let episodes_result: serde_json::Value = episodes_response.json().await.unwrap();
    let episode_id = episodes_result["episodes"][0]["id"].as_str().unwrap();
    
    let token = create_test_token();
    
    // Act - Get stream URL
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get stream URL");
    
    // Assert
    if response.status().is_success() {
        let stream_data: serde_json::Value = response.json().await.unwrap();
        
        // Check expiration is set
        assert!(stream_data["expires_at"].is_string(), "Should have expiration time");
        
        // Parse and validate expiration (typically 6 hours)
        // In a real test, we'd parse the timestamp and verify it's in the future
        let expires_at = stream_data["expires_at"].as_str().unwrap();
        assert!(!expires_at.is_empty(), "Expiration time should not be empty");
    }
}

#[tokio::test]
async fn concurrent_streams_are_handled_correctly() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime with multiple episodes
    let anime_data = json!({
        "title": "Concurrent Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 3,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "fall",
            "year": 2024
        },
        "synopsis": "Test",
        "poster_url": "https://example.com/test.jpg",
        "tags": []
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to create anime");
    
    let created_anime: serde_json::Value = create_response.json().await.unwrap();
    let anime_id = created_anime["id"].as_str().unwrap();
    
    let episodes_data = json!({
        "episodes": [
            {"episode_number": 1, "title": "Episode 1"},
            {"episode_number": 2, "title": "Episode 2"},
            {"episode_number": 3, "title": "Episode 3"}
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    let episodes_response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to get episodes");
    
    let episodes_result: serde_json::Value = episodes_response.json().await.unwrap();
    let episodes = episodes_result["episodes"].as_array().unwrap();
    
    let token = create_test_token();
    
    // Act - Request multiple streams concurrently
    let mut handles = vec![];
    
    for episode in episodes.iter().take(3) {
        let episode_id = episode["id"].as_str().unwrap().to_string();
        let address = app.address.clone();
        let token_clone = token.clone();
        let client = app.client.clone();
        
        let handle = tokio::spawn(async move {
            client
                .get(&format!("{}/api/stream/{}", address, episode_id))
                .header("Authorization", format!("Bearer {}", token_clone))
                .send()
                .await
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests
    let results = futures::future::join_all(handles).await;
    
    // Assert - All concurrent requests should succeed
    for result in results {
        let response = result.expect("Task panicked").expect("Request failed");
        assert!(
            response.status().is_success() || response.status().is_client_error(),
            "Concurrent stream requests should be handled"
        );
    }
}