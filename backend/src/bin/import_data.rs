use kensho_backend::{
    models::{Anime, AnimeStatus, AnimeType, AnimeSeason, Season, ImdbData},
    services::database_simplified::DatabaseService,
};
use chrono::Utc;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use uuid::Uuid;

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
    thumbnail: String,
    duration: Option<Duration>,
    score: Option<Score>,
    synonyms: Vec<String>,
    studios: Vec<String>,
    producers: Vec<String>,
    related_anime: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AnimeSeasonRaw {
    season: String,
    year: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Duration {
    value: i32,
    unit: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Score {
    arithmetic_mean: Option<f32>,
    median: Option<f32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting anime data import...");

    // Connect to database (using simplified in-memory version)
    let db = DatabaseService::new("memory://").await?;
    db.initialize_schema().await?;
    println!("Connected to database");

    // Read the anime database file
    let file = File::open("data/anime-offline-database.json")?;
    let reader = BufReader::new(file);
    let database: AnimeOfflineDatabase = serde_json::from_reader(reader)?;
    println!("Loaded {} anime entries", database.data.len());

    // Track imported count
    let mut imported = 0;
    let mut skipped = 0;

    // Import each anime (limit to first 1000 for testing)
    for (index, entry) in database.data.iter().take(1000).enumerate() {
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

        // Create ImdbData if score exists
        let imdb = entry.score.as_ref().and_then(|s| {
            s.arithmetic_mean.or(s.median).map(|rating| ImdbData {
                id: format!("tt{:07}", index), // Generate fake IMDB ID
                rating,
                votes: 1000, // Default vote count
            })
        });

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
            imdb,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Insert into database
        match db.create_anime(&anime).await {
            Ok(_) => {
                imported += 1;
                if imported % 100 == 0 {
                    println!("Imported {} anime...", imported);
                }
            }
            Err(e) => {
                eprintln!("Failed to import '{}': {}", entry.title, e);
            }
        }
    }

    println!("\nImport complete!");
    println!("  Imported: {} anime", imported);
    println!("  Skipped: {} anime (no season/year data)", skipped);
    
    // Verify count
    let total = db.get_anime_count().await?;
    println!("  Total in database: {}", total);

    Ok(())
}