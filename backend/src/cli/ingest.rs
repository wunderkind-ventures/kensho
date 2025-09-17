use anyhow::{Result, Context};
use clap::Parser;
use kensho_backend::services::metadata::{MetadataService, OfflineAnimeEntry};
use kensho_backend::services::database_simplified::DatabaseService;
use kensho_backend::models::{Anime, Episode};
use serde_json;
use std::fs;
use uuid::Uuid;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the anime-offline-database.json file
    #[arg(short, long, default_value = ".data/anime-offline-database.json")]
    file: String,
    
    /// Maximum number of anime to import (for testing)
    #[arg(short, long)]
    limit: Option<usize>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }
    
    tracing::info!("Starting anime metadata ingestion");
    tracing::info!("Reading from: {}", args.file);
    
    // Read the JSON file
    let content = fs::read_to_string(&args.file)
        .context("Failed to read anime database file")?;
    
    tracing::info!("Parsing JSON data...");
    let json: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse JSON")?;
    
    // Extract the data array
    let data_array = json["data"]
        .as_array()
        .context("Expected 'data' field to be an array")?;
    
    tracing::info!("Found {} anime entries", data_array.len());
    
    // Initialize services
    let mut metadata_service = MetadataService::new(args.file.clone());
    
    // For now, we'll use the simplified database service
    // In production, this would connect to SurrealDB
    let db = DatabaseService::new("memory://").await?;
    db.initialize_schema().await?;
    
    // Process entries
    let limit = args.limit.unwrap_or(data_array.len());
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
                                if args.verbose {
                                    tracing::debug!("Imported: {}", anime.title);
                                }
                                
                                // Create placeholder episodes
                                for ep_num in 1..=anime.episodes {
                                    let episode = Episode {
                                        id: Uuid::new_v4(),
                                        anime_id: anime.id,
                                        episode_number: ep_num,
                                        title: Some(format!("Episode {}", ep_num)),
                                        duration: None, // Duration would be per-episode, not from anime metadata
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
                                tracing::error!("Failed to save anime {}: {}", entry.title, e);
                            }
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        tracing::error!("Failed to convert entry {}: {}", entry.title, e);
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                tracing::error!("Failed to parse entry at index {}: {}", index, e);
                if args.verbose {
                    tracing::debug!("Entry data: {:?}", entry_value);
                }
            }
        }
        
        // Progress indicator
        if (index + 1) % 100 == 0 {
            tracing::info!("Processed {} / {} entries", index + 1, limit);
        }
    }
    
    // Final report
    tracing::info!("Ingestion complete!");
    tracing::info!("Successfully imported: {} anime", success_count);
    if error_count > 0 {
        tracing::warn!("Failed to import: {} anime", error_count);
    }
    
    // Verify data
    let count = db.get_anime_count().await?;
    tracing::info!("Total anime in database: {}", count);
    
    Ok(())
}