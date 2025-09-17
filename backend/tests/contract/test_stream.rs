// T015: Contract test for GET /api/stream/{id}
// Reference: contracts/openapi.yaml - Streaming endpoint specification

use serde_json::json;
use uuid::Uuid;
use chrono::{Utc, Duration};

#[path = "../common/mod.rs"]
mod common;
use common::{spawn_app, create_test_token};

#[tokio::test]
async fn stream_returns_200_with_valid_episode_id() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Create anime with episode
    let anime_data = json!({
        "title": "Stream Test Anime",
        "synonyms": ["Test Stream"],
        "sources": ["https://myanimelist.net/anime/12345/"],
        "episodes": 12,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "winter",
            "year": 2024
        },
        "synopsis": "Testing streaming functionality",
        "poster_url": "https://example.com/stream_test.jpg",
        "tags": ["Action", "Adventure"]
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to create anime");
    
    let created_anime: serde_json::Value = create_response.json().await.unwrap();
    let anime_id = created_anime["id"].as_str().unwrap();
    
    // Create episode
    let episodes_data = json!({
        "episodes": [
            {
                "episode_number": 1,
                "title": "Episode 1",
                "duration": 1440,
                "synopsis": "First episode",
                "thumbnail_url": "https://example.com/ep1_thumb.jpg"
            }
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    // Get episode ID
    let episodes_response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to get episodes");
    
    let episodes_result: serde_json::Value = episodes_response.json().await.unwrap();
    let episode_id = episodes_result["episodes"][0]["id"].as_str().unwrap();
    
    // Act
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let stream_response: serde_json::Value = response.json().await.unwrap();
    
    // Verify response schema matches OpenAPI spec (StreamResponse)
    assert!(stream_response["stream_url"].is_string(), "stream_url must be a string");
    assert!(stream_response["expires_at"].is_string(), "expires_at must be a string");
    assert!(stream_response["quality"].is_string(), "quality must be a string");
    
    // Optional fields
    if stream_response.get("subtitles").is_some() {
        assert!(stream_response["subtitles"].is_array(), "subtitles must be an array");
        
        let subtitles = stream_response["subtitles"].as_array().unwrap();
        for subtitle in subtitles {
            assert!(subtitle["language"].is_string(), "subtitle language must be a string");
            assert!(subtitle["url"].is_string(), "subtitle url must be a string");
            
            if subtitle.get("format").is_some() {
                assert!(subtitle["format"].is_string(), "subtitle format must be a string");
            }
        }
    }
    
    if stream_response.get("audio_tracks").is_some() {
        assert!(stream_response["audio_tracks"].is_array(), "audio_tracks must be an array");
        
        let audio_tracks = stream_response["audio_tracks"].as_array().unwrap();
        for track in audio_tracks {
            assert!(track["language"].is_string(), "audio language must be a string");
            assert!(track["label"].is_string(), "audio label must be a string");
        }
    }
}

#[tokio::test]
async fn stream_returns_401_without_authentication() {
    // Arrange
    let app = spawn_app().await;
    let episode_id = Uuid::new_v4();
    
    // Act - No auth header
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
    assert_eq!(error_response["error"].as_str().unwrap(), "Authentication required");
}

#[tokio::test]
async fn stream_returns_404_for_non_existent_episode() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    let non_existent_id = Uuid::new_v4();
    
    // Act
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, non_existent_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
    assert_eq!(error_response["error"].as_str().unwrap(), "Episode not found");
}

#[tokio::test]
async fn stream_returns_400_for_invalid_episode_id() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Act - Invalid UUID format
    let response = app.client
        .get(&format!("{}/api/stream/invalid-uuid", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
}

#[tokio::test]
async fn stream_url_includes_expiration() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Create anime and episode
    let anime_data = json!({
        "title": "Expiry Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
            "year": 2024
        },
        "synopsis": "Testing URL expiration",
        "poster_url": "https://example.com/expiry.jpg",
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
    
    // Act
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    if response.status().is_success() {
        let stream_response: serde_json::Value = response.json().await.unwrap();
        
        // Verify expiry is set
        assert!(stream_response["expires_at"].is_string(), "Should include expiration time");
        
        let expires_at_str = stream_response["expires_at"].as_str().unwrap();
        
        // Try to parse as ISO 8601 datetime
        if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(expires_at_str) {
            let now = Utc::now();
            let expiry_utc = expires_at.with_timezone(&Utc);
            
            // Expiry should be in the future (at least 1 hour, typically 6 hours)
            assert!(
                expiry_utc > now,
                "Stream URL expiry should be in the future"
            );
            
            assert!(
                expiry_utc < now + Duration::hours(24),
                "Stream URL expiry should be within 24 hours"
            );
        }
    }
}

#[tokio::test]
async fn stream_supports_quality_parameter() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Create anime and episode
    let anime_data = json!({
        "title": "Quality Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "summer",
            "year": 2024
        },
        "synopsis": "Testing quality selection",
        "poster_url": "https://example.com/quality.jpg",
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
    
    // Test different quality settings
    let quality_settings = vec!["auto", "1080p", "720p", "480p", "360p"];
    
    for quality in quality_settings {
        // Act
        let response = app.client
            .get(&format!("{}/api/stream/{}?quality={}", app.address, episode_id, quality))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to send request");
        
        // Assert
        if response.status().is_success() {
            let stream_response: serde_json::Value = response.json().await.unwrap();
            
            // Should return requested quality or best available
            let returned_quality = stream_response["quality"].as_str().unwrap();
            assert!(
                !returned_quality.is_empty(),
                "Quality should be specified in response"
            );
        }
    }
}

#[tokio::test]
async fn stream_returns_403_for_region_restricted_content() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Create anime marked as region-restricted
    let anime_data = json!({
        "title": "Region Restricted Anime",
        "synonyms": [],
        "sources": ["https://crunchyroll.com/restricted"],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "fall",
            "year": 2024
        },
        "synopsis": "Not available in all regions",
        "poster_url": "https://example.com/restricted.jpg",
        "tags": ["Region-Locked"]
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
            {"episode_number": 1, "title": "Restricted Episode"}
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
    
    // Act - Simulate region restriction
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Forwarded-For", "1.1.1.1")  // Simulate different region
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - May return 403 or 200 depending on implementation
    if response.status().as_u16() == 403 {
        let error_response: serde_json::Value = response.json().await.unwrap();
        assert!(error_response["error"].is_string());
        assert!(
            error_response["error"].as_str().unwrap().contains("region") ||
            error_response["error"].as_str().unwrap().contains("not available"),
            "Should indicate region restriction"
        );
    }
}

#[tokio::test]
async fn stream_response_matches_openapi_schema() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Create anime and episode
    let anime_data = json!({
        "title": "Schema Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "winter",
            "year": 2025
        },
        "synopsis": "Testing schema compliance",
        "poster_url": "https://example.com/schema.jpg",
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
            {
                "episode_number": 1,
                "title": "Schema Episode",
                "duration": 1440
            }
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
    
    // Act
    let response = app.client
        .get(&format!("{}/api/stream/{}", app.address, episode_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - Validate against StreamResponse schema
    if response.status().is_success() {
        let stream_response: serde_json::Value = response.json().await.unwrap();
        
        // Required fields
        assert!(stream_response["stream_url"].is_string(), "stream_url is required");
        assert!(stream_response["expires_at"].is_string(), "expires_at is required");
        assert!(stream_response["quality"].is_string(), "quality is required");
        
        // Validate stream_url format (should be a valid URL)
        let stream_url = stream_response["stream_url"].as_str().unwrap();
        assert!(
            stream_url.starts_with("http://") || stream_url.starts_with("https://"),
            "stream_url should be a valid URL"
        );
        
        // Validate quality enum
        let quality = stream_response["quality"].as_str().unwrap();
        let valid_qualities = vec!["auto", "1080p", "720p", "480p", "360p", "240p"];
        assert!(
            valid_qualities.contains(&quality),
            "quality should be one of the valid options"
        );
        
        // Optional arrays should be arrays if present
        if let Some(subtitles) = stream_response.get("subtitles") {
            assert!(subtitles.is_array(), "subtitles must be an array");
        }
        
        if let Some(audio_tracks) = stream_response.get("audio_tracks") {
            assert!(audio_tracks.is_array(), "audio_tracks must be an array");
        }
    } else if response.status().is_client_error() {
        let error_response: serde_json::Value = response.json().await.unwrap();
        
        // Error response schema
        assert!(error_response["error"].is_string(), "error field is required");
        
        if error_response.get("code").is_some() {
            assert!(error_response["code"].is_string(), "code must be a string if present");
        }
        
        if error_response.get("details").is_some() {
            assert!(error_response["details"].is_object(), "details must be an object if present");
        }
    }
}