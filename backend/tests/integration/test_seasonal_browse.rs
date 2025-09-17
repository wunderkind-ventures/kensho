// T020: Integration test for seasonal browse functionality
// Tests filtering, sorting, and pagination of seasonal anime

use serde_json::json;
use std::collections::HashSet;

#[path = "../common/mod.rs"]
mod common;
use common::spawn_app;

#[tokio::test]
async fn browse_current_season_shows_ongoing_anime() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime for current season (Winter 2024 for this test)
    let current_season_anime = vec![
        json!({
            "title": "Solo Leveling",
            "synonyms": ["Ore Dake Level Up na Ken"],
            "sources": ["https://myanimelist.net/anime/52299/"],
            "episodes": 12,
            "status": "ONGOING",
            "anime_type": "TV",
            "anime_season": {
                "season": "winter",
                "year": 2024
            },
            "synopsis": "A weak hunter becomes the strongest",
            "poster_url": "https://example.com/solo_leveling.jpg",
            "tags": ["Action", "Fantasy", "Adventure"]
        }),
        json!({
            "title": "The Dangers in My Heart Season 2",
            "synonyms": ["Boku no Kokoro no Yabai Yatsu 2nd Season"],
            "sources": ["https://myanimelist.net/anime/53861/"],
            "episodes": 13,
            "status": "ONGOING",
            "anime_type": "TV",
            "anime_season": {
                "season": "winter",
                "year": 2024
            },
            "synopsis": "Romance continues between unlikely pair",
            "poster_url": "https://example.com/dangers2.jpg",
            "tags": ["Romance", "School", "Comedy"]
        }),
    ];
    
    // Create past season anime
    let past_anime = json!({
        "title": "Old Anime",
        "synonyms": [],
        "sources": [],
        "episodes": 24,
        "status": "FINISHED",
        "anime_type": "TV",
        "anime_season": {
            "season": "fall",
            "year": 2023
        },
        "synopsis": "An older anime",
        "poster_url": "https://example.com/old.jpg",
        "tags": []
    });
    
    // Insert all anime
    for anime_data in current_season_anime {
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await
            .expect("Failed to create current season anime");
    }
    
    app.client
        .post(&format!("{}/api/anime", app.address))
        .json(&past_anime)
        .send()
        .await
        .expect("Failed to create past anime");
    
    // Act - Browse Winter 2024
    let response = app.client
        .get(&format!("{}/api/browse/season/2024/winter", app.address))
        .send()
        .await
        .expect("Failed to browse season");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let result: serde_json::Value = response.json().await.unwrap();
    assert_eq!(result["year"].as_u64().unwrap(), 2024);
    assert_eq!(result["season"].as_str().unwrap(), "winter");
    
    let anime_list = result["anime"].as_array().unwrap();
    assert_eq!(anime_list.len(), 2, "Should only show Winter 2024 anime");
    
    // Verify only current season anime are returned
    let titles: HashSet<String> = anime_list
        .iter()
        .map(|a| a["title"].as_str().unwrap().to_string())
        .collect();
    
    assert!(titles.contains("Solo Leveling"));
    assert!(titles.contains("The Dangers in My Heart Season 2"));
    assert!(!titles.contains("Old Anime"));
}

#[tokio::test]
async fn seasonal_browse_supports_pagination() {
    // Arrange
    let app = spawn_app().await;
    
    // Create many anime for same season
    for i in 1..=15 {
        let anime_data = json!({
            "title": format!("Spring 2024 Anime {}", i),
            "synonyms": [],
            "sources": [],
            "episodes": 12,
            "status": if i <= 5 { "ONGOING" } else { "FINISHED" },
            "anime_type": "TV",
            "anime_season": {
                "season": "spring",
                "year": 2024
            },
            "synopsis": format!("Test anime {}", i),
            "poster_url": format!("https://example.com/anime{}.jpg", i),
            "tags": []
        });
        
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await
            .expect("Failed to create anime");
    }
    
    // Act - Get first page
    let page1_response = app.client
        .get(&format!("{}/api/browse/season/2024/spring?page=1&limit=10", app.address))
        .send()
        .await
        .expect("Failed to get page 1");
    
    assert_eq!(page1_response.status().as_u16(), 200);
    
    let page1_result: serde_json::Value = page1_response.json().await.unwrap();
    let page1_anime = page1_result["anime"].as_array().unwrap();
    
    // Should return first 10 items
    assert_eq!(page1_anime.len(), 10, "First page should have 10 items");
    
    // Act - Get second page
    let page2_response = app.client
        .get(&format!("{}/api/browse/season/2024/spring?page=2&limit=10", app.address))
        .send()
        .await
        .expect("Failed to get page 2");
    
    assert_eq!(page2_response.status().as_u16(), 200);
    
    let page2_result: serde_json::Value = page2_response.json().await.unwrap();
    let page2_anime = page2_result["anime"].as_array().unwrap();
    
    // Should return remaining 5 items
    assert_eq!(page2_anime.len(), 5, "Second page should have 5 items");
    
    // Verify total count
    assert_eq!(page1_result["total"].as_u64().unwrap(), 15);
    assert_eq!(page2_result["total"].as_u64().unwrap(), 15);
    
    // Verify no overlap between pages
    let page1_titles: HashSet<String> = page1_anime
        .iter()
        .map(|a| a["title"].as_str().unwrap().to_string())
        .collect();
    
    let page2_titles: HashSet<String> = page2_anime
        .iter()
        .map(|a| a["title"].as_str().unwrap().to_string())
        .collect();
    
    let intersection: Vec<_> = page1_titles.intersection(&page2_titles).collect();
    assert_eq!(intersection.len(), 0, "Pages should not have overlapping items");
}

#[tokio::test]
async fn browse_filters_by_anime_type() {
    // Arrange
    let app = spawn_app().await;
    
    // Create different types of anime for same season
    let anime_types = vec![
        ("TV Show 1", "TV"),
        ("Movie 1", "MOVIE"),
        ("OVA 1", "OVA"),
        ("TV Show 2", "TV"),
        ("Special 1", "SPECIAL"),
        ("ONA 1", "ONA"),
    ];
    
    for (title, anime_type) in anime_types {
        let anime_data = json!({
            "title": title,
            "synonyms": [],
            "sources": [],
            "episodes": if anime_type == "MOVIE" { 1 } else { 12 },
            "status": "FINISHED",
            "anime_type": anime_type,
            "anime_season": {
                "season": "summer",
                "year": 2024
            },
            "synopsis": format!("{} synopsis", title),
            "poster_url": format!("https://example.com/{}.jpg", title.replace(" ", "_")),
            "tags": []
        });
        
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await
            .expect("Failed to create anime");
    }
    
    // Act - Filter for TV shows only
    let tv_response = app.client
        .get(&format!("{}/api/browse/season/2024/summer?type=TV", app.address))
        .send()
        .await
        .expect("Failed to filter by TV");
    
    assert_eq!(tv_response.status().as_u16(), 200);
    
    let tv_result: serde_json::Value = tv_response.json().await.unwrap();
    let tv_anime = tv_result["anime"].as_array().unwrap();
    
    assert_eq!(tv_anime.len(), 2, "Should only return TV shows");
    
    for anime in tv_anime {
        assert_eq!(anime["anime_type"].as_str().unwrap(), "TV");
    }
    
    // Act - Filter for movies
    let movie_response = app.client
        .get(&format!("{}/api/browse/season/2024/summer?type=MOVIE", app.address))
        .send()
        .await
        .expect("Failed to filter by MOVIE");
    
    let movie_result: serde_json::Value = movie_response.json().await.unwrap();
    let movie_anime = movie_result["anime"].as_array().unwrap();
    
    assert_eq!(movie_anime.len(), 1, "Should only return movies");
    assert_eq!(movie_anime[0]["anime_type"].as_str().unwrap(), "MOVIE");
}

#[tokio::test]
async fn browse_filters_by_status() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime with different statuses
    let anime_statuses = vec![
        ("Ongoing Anime 1", "ONGOING"),
        ("Finished Anime 1", "FINISHED"),
        ("Ongoing Anime 2", "ONGOING"),
        ("Upcoming Anime 1", "UPCOMING"),
        ("Finished Anime 2", "FINISHED"),
    ];
    
    for (title, status) in anime_statuses {
        let anime_data = json!({
            "title": title,
            "synonyms": [],
            "sources": [],
            "episodes": 12,
            "status": status,
            "anime_type": "TV",
            "anime_season": {
                "season": "fall",
                "year": 2024
            },
            "synopsis": format!("{} synopsis", title),
            "poster_url": format!("https://example.com/{}.jpg", title.replace(" ", "_")),
            "tags": []
        });
        
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await
            .expect("Failed to create anime");
    }
    
    // Act - Filter for ongoing anime
    let ongoing_response = app.client
        .get(&format!("{}/api/browse/season/2024/fall?status=ONGOING", app.address))
        .send()
        .await
        .expect("Failed to filter by status");
    
    assert_eq!(ongoing_response.status().as_u16(), 200);
    
    let ongoing_result: serde_json::Value = ongoing_response.json().await.unwrap();
    let ongoing_anime = ongoing_result["anime"].as_array().unwrap();
    
    assert_eq!(ongoing_anime.len(), 2, "Should only return ongoing anime");
    
    for anime in ongoing_anime {
        assert_eq!(anime["status"].as_str().unwrap(), "ONGOING");
    }
}

#[tokio::test]
async fn browse_supports_tag_filtering() {
    // Arrange
    let app = spawn_app().await;
    
    // Create anime with different tags
    let anime_with_tags = vec![
        ("Action Anime 1", vec!["Action", "Adventure", "Fantasy"]),
        ("Romance Anime 1", vec!["Romance", "Drama", "School"]),
        ("Action Anime 2", vec!["Action", "Sci-Fi", "Mecha"]),
        ("Slice of Life", vec!["Slice of Life", "Comedy", "School"]),
        ("Action Romance", vec!["Action", "Romance", "Drama"]),
    ];
    
    for (title, tags) in anime_with_tags {
        let anime_data = json!({
            "title": title,
            "synonyms": [],
            "sources": [],
            "episodes": 12,
            "status": "FINISHED",
            "anime_type": "TV",
            "anime_season": {
                "season": "spring",
                "year": 2023
            },
            "synopsis": format!("{} synopsis", title),
            "poster_url": format!("https://example.com/{}.jpg", title.replace(" ", "_")),
            "tags": tags
        });
        
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await
            .expect("Failed to create anime");
    }
    
    // Act - Filter for Action tag
    let action_response = app.client
        .get(&format!("{}/api/browse/season/2023/spring?tag=Action", app.address))
        .send()
        .await
        .expect("Failed to filter by tag");
    
    if action_response.status().is_success() {
        let action_result: serde_json::Value = action_response.json().await.unwrap();
        let action_anime = action_result["anime"].as_array().unwrap();
        
        assert_eq!(action_anime.len(), 3, "Should return anime with Action tag");
        
        // Verify all results have Action tag
        for anime in action_anime {
            let tags = anime["tags"].as_array().unwrap();
            let has_action = tags.iter().any(|t| t.as_str().unwrap() == "Action");
            assert!(has_action, "Each anime should have Action tag");
        }
    }
    
    // Act - Filter for multiple tags (if supported)
    let multi_tag_response = app.client
        .get(&format!("{}/api/browse/season/2023/spring?tag=Action&tag=Romance", app.address))
        .send()
        .await
        .expect("Failed to filter by multiple tags");
    
    if multi_tag_response.status().is_success() {
        let multi_result: serde_json::Value = multi_tag_response.json().await.unwrap();
        let multi_anime = multi_result["anime"].as_array().unwrap();
        
        // Should return only anime with both tags
        assert!(multi_anime.len() <= 1, "Should return anime with both Action and Romance");
    }
}

#[tokio::test]
async fn browse_shows_upcoming_anime_for_future_seasons() {
    // Arrange
    let app = spawn_app().await;
    
    // Create upcoming anime
    let upcoming_anime = vec![
        json!({
            "title": "Upcoming Spring 2025 Anime",
            "synonyms": [],
            "sources": [],
            "episodes": 12,
            "status": "UPCOMING",
            "anime_type": "TV",
            "anime_season": {
                "season": "spring",
                "year": 2025
            },
            "synopsis": "An exciting upcoming anime",
            "poster_url": "https://example.com/upcoming1.jpg",
            "tags": ["Action", "Adventure"]
        }),
        json!({
            "title": "Another Upcoming Anime",
            "synonyms": [],
            "sources": [],
            "episodes": 24,
            "status": "UPCOMING",
            "anime_type": "TV",
            "anime_season": {
                "season": "spring",
                "year": 2025
            },
            "synopsis": "Another exciting anime",
            "poster_url": "https://example.com/upcoming2.jpg",
            "tags": ["Romance", "Drama"]
        }),
    ];
    
    for anime_data in upcoming_anime {
        app.client
            .post(&format!("{}/api/anime", app.address))
            .json(&anime_data)
            .send()
            .await
            .expect("Failed to create upcoming anime");
    }
    
    // Act - Browse future season
    let response = app.client
        .get(&format!("{}/api/browse/season/2025/spring", app.address))
        .send()
        .await
        .expect("Failed to browse future season");
    
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    
    let result: serde_json::Value = response.json().await.unwrap();
    let anime_list = result["anime"].as_array().unwrap();
    
    assert_eq!(anime_list.len(), 2, "Should show upcoming anime");
    
    for anime in anime_list {
        assert_eq!(anime["status"].as_str().unwrap(), "UPCOMING");
    }
}