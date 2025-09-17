use kensho_backend::services::metadata::{MetadataService, OfflineAnimeEntry};
use kensho_backend::services::database_simplified::DatabaseService;
use kensho_backend::models::{Anime, Episode};
use serde_json;
use std::fs;
use uuid::Uuid;

#[tokio::test]
async fn test_import_sample_data() {
    println!("Starting anime metadata ingestion test");
    
    // Read the JSON file
    let content = fs::read_to_string(".data/anime-offline-database.json")
        .expect("Failed to read anime database file");
    
    println!("Parsing JSON data...");
    let json: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse JSON");
    
    // Extract the data array
    let data_array = json["data"]
        .as_array()
        .expect("Expected 'data' field to be an array");
    
    println!("Found {} anime entries", data_array.len());
    
    // Initialize services
    let metadata_service = MetadataService::new(".data/anime-offline-database.json".to_string());
    
    // Use in-memory database for testing
    let db = DatabaseService::new("memory://").await
        .expect("Failed to create database service");
    db.initialize_schema().await
        .expect("Failed to initialize schema");
    
    // Process first 10 entries as a test
    let limit = 10;
    let entries_to_process = data_array.iter().take(limit);
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (index, entry_value) in entries_to_process.enumerate() {
        // Parse the entry
        match serde_json::from_value::<OfflineAnimeEntry>(entry_value.clone()) {
            Ok(entry) => {
                // Convert to our Anime model
                match metadata_service.convert_to_anime(entry.clone()) {
                    Ok(anime) => {
                        // Save to database
                        match db.create_anime(&anime).await {
                            Ok(_) => {
                                success_count += 1;
                                println!("Imported: {}", anime.title);
                                
                                // Create placeholder episodes
                                for ep_num in 1..=anime.episodes.min(5) { // Limit episodes for testing
                                    let episode = Episode {
                                        id: Uuid::new_v4(),
                                        anime_id: anime.id,
                                        episode_number: ep_num,
                                        title: Some(format!("Episode {}", ep_num)),
                                        duration: None,
                                        air_date: None,
                                        synopsis: None,
                                        thumbnail_url: None,
                                        created_at: chrono::Utc::now(),
                                        updated_at: chrono::Utc::now(),
                                    };
                                    let _ = db.create_episode(&episode).await;
                                }
                            }
                            Err(e) => {
                                error_count += 1;
                                eprintln!("Failed to save anime {}: {}", entry.title, e);
                            }
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("Failed to convert entry {}: {}", entry.title, e);
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Failed to parse entry at index {}: {}", index, e);
            }
        }
    }
    
    // Final report
    println!("\nIngestion complete!");
    println!("Successfully imported: {} anime", success_count);
    if error_count > 0 {
        println!("Failed to import: {} anime", error_count);
    }
    
    // Verify data
    let count = db.get_anime_count().await
        .expect("Failed to get anime count");
    println!("Total anime in database: {}", count);
    
    // Test fetching some data
    let all_anime = db.get_all_anime().await
        .expect("Failed to fetch anime");
    println!("\nImported anime:");
    for anime in all_anime.iter().take(5) {
        println!("- {} ({} episodes)", anime.title, anime.episodes);
    }
    
    assert_eq!(success_count, limit as i32);
    assert_eq!(count, limit);
}