// T011: Contract test GET /api/anime/{id}/episodes
// Reference: contracts/openapi.yaml lines 119-143

use serde_json::json;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn get_episodes_returns_200_with_valid_anime_id() {
    // Arrange
    let app = spawn_app().await;
    
    // Create an anime
    let anime_data = json!({
        "title": "Attack on Titan",
        "synonyms": ["Shingeki no Kyojin"],
        "sources": ["https://myanimelist.net/anime/16498/"],
        "episodes": 25,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
            "year": 2013
        },
        "synopsis": "Humanity fights for survival against giant humanoid Titans",
        "poster_url": "https://example.com/aot.jpg",
        "tags": ["Action", "Drama", "Fantasy"]
    });
    
    let create_response = app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&anime_data)
        .send()
        .await
        .expect("Failed to create anime");
    
    assert_eq!(create_response.status().as_u16(), 201);
    let created_anime: serde_json::Value = create_response.json().await.unwrap();
    let anime_id = created_anime["id"].as_str().unwrap();
    
    // Create episodes
    let episodes_data = json!({
        "episodes": [
            {
                "episode_number": 1,
                "title": "To You, in 2000 Years",
                "duration": 1440,
                "synopsis": "The Colossal Titan appears",
                "thumbnail_url": "https://example.com/aot_ep1.jpg"
            },
            {
                "episode_number": 2,
                "title": "That Day",
                "duration": 1440,
                "synopsis": "The fall of Wall Maria",
                "thumbnail_url": "https://example.com/aot_ep2.jpg"
            },
            {
                "episode_number": 3,
                "title": "A Dim Light Amid Despair",
                "duration": 1440,
                "synopsis": "Training begins",
                "thumbnail_url": "https://example.com/aot_ep3.jpg"
            }
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let episodes_response: serde_json::Value = response.json().await.unwrap();
    assert!(episodes_response["episodes"].is_array());
    assert!(episodes_response["total"].is_number());
    
    let episodes = episodes_response["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 3);
    assert_eq!(episodes[0]["episode_number"].as_u64().unwrap(), 1);
    assert_eq!(episodes[0]["title"].as_str().unwrap(), "To You, in 2000 Years");
}

#[tokio::test]
async fn get_episodes_returns_404_for_non_existent_anime() {
    // Arrange
    let app = spawn_app().await;
    let non_existent_id = Uuid::new_v4();
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, non_existent_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert!(error_response["error"].is_string());
}

#[tokio::test]
async fn get_episodes_returns_empty_list_when_no_episodes() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime without episodes
    let anime_data = json!({
        "title": "Movie Without Episodes",
        "synonyms": [],
        "sources": [],
        "episodes": 1,
        "status": "FINISHED",
        "anime_type": "MOVIE",
        "anime_season": {
            "season": "summer",
            "year": 2023
        },
        "synopsis": "A standalone movie",
        "poster_url": "https://example.com/movie.jpg",
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
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let episodes_response: serde_json::Value = response.json().await.unwrap();
    let episodes = episodes_response["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 0);
    assert_eq!(episodes_response["total"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn get_episodes_response_matches_openapi_schema() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime and episodes
    let anime_data = json!({
        "title": "Schema Test Series",
        "synonyms": [],
        "sources": [],
        "episodes": 12,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "winter",
            "year": 2024
        },
        "synopsis": "Testing schema",
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
            {
                "episode_number": 1,
                "title": "Episode 1",
                "duration": 1440,
                "air_date": "2024-01-07",
                "synopsis": "First episode",
                "thumbnail_url": "https://example.com/ep1.jpg"
            }
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - Check response matches OpenAPI schema (EpisodeListResponse)
    let episodes_response: serde_json::Value = response.json().await.unwrap();
    
    // Required fields
    assert!(episodes_response["episodes"].is_array(), "episodes must be an array");
    assert!(episodes_response["total"].is_number(), "total must be a number");
    
    // Check Episode schema for each item
    let episodes = episodes_response["episodes"].as_array().unwrap();
    for episode in episodes {
        assert!(episode["id"].is_string(), "id must be a string");
        assert!(episode["episode_number"].is_number(), "episode_number must be a number");
        
        // Optional fields
        if episode.get("title").is_some() {
            assert!(episode["title"].is_string(), "title must be a string if present");
        }
        if episode.get("duration").is_some() {
            assert!(episode["duration"].is_number(), "duration must be a number if present");
        }
        if episode.get("air_date").is_some() {
            assert!(episode["air_date"].is_string(), "air_date must be a string if present");
        }
        if episode.get("synopsis").is_some() {
            assert!(episode["synopsis"].is_string(), "synopsis must be a string if present");
        }
        if episode.get("thumbnail_url").is_some() {
            assert!(episode["thumbnail_url"].is_string(), "thumbnail_url must be a string if present");
        }
    }
}

#[tokio::test]
async fn get_episodes_sorted_by_episode_number() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime
    let anime_data = json!({
        "title": "Test Series",
        "synonyms": [],
        "sources": [],
        "episodes": 5,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "spring",
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
    
    // Create episodes in random order
    let episodes_data = json!({
        "episodes": [
            {"episode_number": 3, "title": "Episode 3"},
            {"episode_number": 1, "title": "Episode 1"},
            {"episode_number": 5, "title": "Episode 5"},
            {"episode_number": 2, "title": "Episode 2"},
            {"episode_number": 4, "title": "Episode 4"}
        ]
    });
    
    app.client
        .post(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .json(&episodes_data)
        .send()
        .await;
    
    // Act
    let response = app.client
        .get(&format!("{}/api/anime/{}/episodes", app.address, anime_id))
        .send()
        .await
        .expect("Failed to send request");
    
    // Assert - Episodes should be sorted by episode_number
    let episodes_response: serde_json::Value = response.json().await.unwrap();
    let episodes = episodes_response["episodes"].as_array().unwrap();
    
    assert_eq!(episodes.len(), 5);
    for i in 0..5 {
        assert_eq!(
            episodes[i]["episode_number"].as_u64().unwrap(),
            (i + 1) as u64,
            "Episodes should be sorted by episode_number"
        );
    }
}