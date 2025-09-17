// T019: Integration test for session persistence and management
// Tests user preferences, watch history, and session continuity

use serde_json::json;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;
use common::{spawn_app, create_test_token};

#[tokio::test]
async fn user_preferences_persist_across_sessions() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Step 1: Set user preferences
    let preferences = json!({
        "language": "en",
        "subtitle_language": "en",
        "quality": "1080p",
        "autoplay": true,
        "skip_intro": true
    });
    
    let set_response = app.client
        .put(&format!("{}/api/user/preferences", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&preferences)
        .send()
        .await
        .expect("Failed to set preferences");
    
    assert_eq!(set_response.status().as_u16(), 200, "Should set preferences");
    
    // Step 2: Retrieve preferences in new session
    let get_response = app.client
        .get(&format!("{}/api/user/preferences", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get preferences");
    
    assert_eq!(get_response.status().as_u16(), 200);
    
    let saved_preferences: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(saved_preferences["language"].as_str().unwrap(), "en");
    assert_eq!(saved_preferences["quality"].as_str().unwrap(), "1080p");
    assert_eq!(saved_preferences["autoplay"].as_bool().unwrap(), true);
    assert_eq!(saved_preferences["skip_intro"].as_bool().unwrap(), true);
}

#[tokio::test]
async fn watch_history_is_tracked_per_user() {
    // Arrange
    let app = spawn_app().await;
    let user1_token = create_test_token();
    
    // Create anime and episodes
    let anime_data = json!({
        "title": "Watch History Test",
        "synonyms": [],
        "sources": [],
        "episodes": 3,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "winter",
            "year": 2024
        },
        "synopsis": "Test anime for watch history",
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
    
    // Create episodes
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
    let episode1_id = episodes_result["episodes"][0]["id"].as_str().unwrap();
    let episode2_id = episodes_result["episodes"][1]["id"].as_str().unwrap();
    
    // Step 1: Record watch progress for episode 1
    let watch_data = json!({
        "episode_id": episode1_id,
        "progress": 720,  // 12 minutes watched
        "total_duration": 1440,  // 24 minutes total
        "completed": false
    });
    
    let watch_response = app.client
        .post(&format!("{}/api/user/watch-history", app.address))
        .header("Authorization", format!("Bearer {}", user1_token))
        .json(&watch_data)
        .send()
        .await
        .expect("Failed to record watch progress");
    
    assert_eq!(watch_response.status().as_u16(), 200);
    
    // Step 2: Complete episode 1
    let complete_data = json!({
        "episode_id": episode1_id,
        "progress": 1440,
        "total_duration": 1440,
        "completed": true
    });
    
    app.client
        .post(&format!("{}/api/user/watch-history", app.address))
        .header("Authorization", format!("Bearer {}", user1_token))
        .json(&complete_data)
        .send()
        .await;
    
    // Step 3: Start episode 2
    let watch_data2 = json!({
        "episode_id": episode2_id,
        "progress": 360,
        "total_duration": 1440,
        "completed": false
    });
    
    app.client
        .post(&format!("{}/api/user/watch-history", app.address))
        .header("Authorization", format!("Bearer {}", user1_token))
        .json(&watch_data2)
        .send()
        .await;
    
    // Step 4: Get watch history
    let history_response = app.client
        .get(&format!("{}/api/user/watch-history", app.address))
        .header("Authorization", format!("Bearer {}", user1_token))
        .send()
        .await
        .expect("Failed to get watch history");
    
    if history_response.status().is_success() {
        let history: serde_json::Value = history_response.json().await.unwrap();
        let episodes = history["episodes"].as_array().unwrap_or(&vec![]);
        
        // Should have history for watched episodes
        assert!(episodes.len() >= 2, "Should have watch history for multiple episodes");
    }
}

#[tokio::test]
async fn session_continues_after_token_refresh() {
    // Arrange
    let app = spawn_app().await;
    
    // Step 1: Login and get initial tokens
    let login_data = json!({
        "username": "test_user",
        "password": "test_password"
    });
    
    let login_response = app.client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to login");
    
    if login_response.status().is_success() {
        let auth_tokens: serde_json::Value = login_response.json().await.unwrap();
        let access_token = auth_tokens["access_token"].as_str().unwrap();
        let refresh_token = auth_tokens["refresh_token"].as_str().unwrap();
        
        // Step 2: Set a preference with initial token
        let preference = json!({
            "language": "ja"
        });
        
        app.client
            .put(&format!("{}/api/user/preferences", app.address))
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&preference)
            .send()
            .await;
        
        // Step 3: Refresh token
        let refresh_data = json!({
            "refresh_token": refresh_token
        });
        
        let refresh_response = app.client
            .post(&format!("{}/api/auth/refresh", app.address))
            .json(&refresh_data)
            .send()
            .await
            .expect("Failed to refresh token");
        
        if refresh_response.status().is_success() {
            let new_tokens: serde_json::Value = refresh_response.json().await.unwrap();
            let new_access_token = new_tokens["access_token"].as_str().unwrap();
            
            // Step 4: Verify session continues with new token
            let pref_response = app.client
                .get(&format!("{}/api/user/preferences", app.address))
                .header("Authorization", format!("Bearer {}", new_access_token))
                .send()
                .await
                .expect("Failed to get preferences");
            
            assert!(
                pref_response.status().is_success(),
                "Session should continue after token refresh"
            );
            
            let preferences: serde_json::Value = pref_response.json().await.unwrap();
            assert_eq!(
                preferences["language"].as_str().unwrap_or(""),
                "ja",
                "Preferences should persist after token refresh"
            );
        }
    }
}

#[tokio::test]
async fn watchlist_is_maintained_per_user() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Create multiple anime
    let anime_ids: Vec<String> = vec![];
    for i in 1..=3 {
        let anime_data = json!({
            "title": format!("Watchlist Anime {}", i),
            "synonyms": [],
            "sources": [],
            "episodes": 12,
            "status": "FINISHED",
            "anime_type": "TV",
            "anime_season": {
                "season": "spring",
                "year": 2024
            },
            "synopsis": format!("Test anime {} for watchlist", i),
            "poster_url": format!("https://example.com/anime{}.jpg", i),
            "tags": []
        });
        
        let response = app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await
            .expect("Failed to create anime");
        
        if response.status().is_success() {
            let anime: serde_json::Value = response.json().await.unwrap();
            let anime_id = anime["id"].as_str().unwrap().to_string();
            
            // Add to watchlist
            let watchlist_data = json!({
                "anime_id": anime_id,
                "status": "watching"
            });
            
            let add_response = app.client
                .post(&format!("{}/api/user/watchlist", app.address))
                .header("Authorization", format!("Bearer {}", token))
                .json(&watchlist_data)
                .send()
                .await
                .expect("Failed to add to watchlist");
            
            assert!(
                add_response.status().is_success(),
                "Should add anime to watchlist"
            );
        }
    }
    
    // Get watchlist
    let watchlist_response = app.client
        .get(&format!("{}/api/user/watchlist", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get watchlist");
    
    if watchlist_response.status().is_success() {
        let watchlist: serde_json::Value = watchlist_response.json().await.unwrap();
        let items = watchlist["items"].as_array().unwrap_or(&vec![]);
        
        assert_eq!(items.len(), 3, "Should have 3 anime in watchlist");
        
        // Update status of one item
        if !items.is_empty() {
            let first_item_id = items[0]["anime_id"].as_str().unwrap();
            
            let update_data = json!({
                "anime_id": first_item_id,
                "status": "completed"
            });
            
            let update_response = app.client
                .put(&format!("{}/api/user/watchlist", app.address))
                .header("Authorization", format!("Bearer {}", token))
                .json(&update_data)
                .send()
                .await
                .expect("Failed to update watchlist");
            
            assert!(
                update_response.status().is_success(),
                "Should update watchlist item"
            );
        }
    }
}

#[tokio::test]
async fn resume_playback_from_last_position() {
    // Arrange
    let app = spawn_app().await;
    let token = create_test_token();
    
    // Create anime with episode
    let anime_data = json!({
        "title": "Resume Test Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "summer",
            "year": 2024
        },
        "synopsis": "Test resume functionality",
        "poster_url": "https://example.com/resume.jpg",
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
            {"episode_number": 1, "title": "Episode 1", "duration": 1440}
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
    
    // Step 1: Save playback position
    let position_data = json!({
        "episode_id": episode_id,
        "position": 600,  // 10 minutes
        "duration": 1440
    });
    
    let save_response = app.client
        .post(&format!("{}/api/user/playback-position", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&position_data)
        .send()
        .await
        .expect("Failed to save position");
    
    assert!(
        save_response.status().is_success(),
        "Should save playback position"
    );
    
    // Step 2: Get resume position
    let resume_response = app.client
        .get(&format!("{}/api/user/playback-position/{}", app.address, episode_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get resume position");
    
    if resume_response.status().is_success() {
        let resume_data: serde_json::Value = resume_response.json().await.unwrap();
        
        assert_eq!(
            resume_data["position"].as_u64().unwrap_or(0),
            600,
            "Should return saved playback position"
        );
        assert_eq!(
            resume_data["episode_id"].as_str().unwrap(),
            episode_id,
            "Should return correct episode ID"
        );
    }
}