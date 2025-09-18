use crate::models::{Anime, AnimeStatus, AnimeType, AnimeSeason, Season, ImdbData};
use crate::services::database_v2::DatabaseService;
use chrono::Utc;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Deserialize)]
struct AnimeOfflineDatabase {
    data: Vec<AnimeEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnimeEntry {
    sources: Vec<String>,
    title: String,
    #[serde(rename = "type")]
    anime_type: String,
    episodes: Option<i32>,
    status: String,
    anime_season: Option<AnimeSeasonRaw>,
    picture: String,
    synonyms: Vec<String>,
    studios: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AnimeSeasonRaw {
    season: String,
    year: Option<i32>,
}

pub async fn load_initial_data(db: &DatabaseService) -> Result<()> {
    // Check if data is already loaded
    let count = db.get_anime_count().await?;
    if count > 0 {
        tracing::info!("Database already contains {} anime, skipping import", count);
        return Ok(());
    }
    
    tracing::info!("Loading initial anime data...");
    
    // Try to load the anime database file
    let file_path = "data/anime-offline-database.json";
    if !std::path::Path::new(file_path).exists() {
        tracing::warn!("Anime database file not found at {}, skipping data load", file_path);
        return Ok(());
    }
    
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let database: AnimeOfflineDatabase = serde_json::from_reader(reader)?;
    
    tracing::info!("Found {} anime entries to load", database.data.len());
    
    let mut imported = 0;
    let mut skipped = 0;
    
    // Import first 500 for quick loading
    for (index, entry) in database.data.iter().take(500).enumerate() {
        // Skip entries without season data or year
        let Some(season_raw) = &entry.anime_season else {
            skipped += 1;
            continue;
        };
        
        let Some(year) = season_raw.year else {
            skipped += 1;
            continue;
        };
        
        // Map status
        let status = match entry.status.as_str() {
            "FINISHED" => AnimeStatus::Finished,
            "ONGOING" => AnimeStatus::Ongoing,
            "UPCOMING" => AnimeStatus::Upcoming,
            _ => AnimeStatus::Unknown,
        };
        
        // Map anime type
        let anime_type = match entry.anime_type.to_uppercase().as_str() {
            "TV" => AnimeType::TV,
            "MOVIE" => AnimeType::Movie,
            "OVA" => AnimeType::OVA,
            "ONA" => AnimeType::ONA,
            "SPECIAL" => AnimeType::Special,
            _ => AnimeType::Unknown,
        };
        
        // Map season
        let season = match season_raw.season.to_uppercase().as_str() {
            "SPRING" => Season::Spring,
            "SUMMER" => Season::Summer,
            "FALL" | "AUTUMN" => Season::Fall,
            "WINTER" => Season::Winter,
            _ => Season::Spring,
        };
        
        // Create anime record
        let anime = Anime {
            id: Uuid::new_v4(),
            title: entry.title.clone(),
            synonyms: entry.synonyms.clone(),
            sources: entry.sources.clone(),
            episodes: entry.episodes.unwrap_or(0).max(0) as u32,
            status,
            anime_type,
            anime_season: AnimeSeason {
                season,
                year: year as u16,
            },
            synopsis: format!("A {} anime with {} episodes. Studios: {}. Tags: {}", 
                entry.anime_type, 
                entry.episodes.unwrap_or(0),
                entry.studios.join(", "),
                entry.tags.join(", ")
            ),
            poster_url: entry.picture.clone(),
            imdb: None,  // No IMDB data in this dataset
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Insert into database
        if let Ok(_) = db.create_anime(&anime).await {
            imported += 1;
            if imported % 100 == 0 {
                tracing::debug!("Imported {} anime...", imported);
            }
        }
    }
    
    tracing::info!("Data import complete: imported {}, skipped {}", imported, skipped);
    
    Ok(())
}