use kensho_backend::{
    models::{Anime, AnimeStatus, AnimeType, AnimeSeason, Season, ImdbData, Tag, TagCategory},
    services::database_v2::DatabaseService,
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
    synonyms: Vec<String>,
    studios: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AnimeSeasonRaw {
    season: String,
    year: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting anime data import to SurrealDB...");

    // Connect to SurrealDB
    let db = DatabaseService::new("ws://localhost:8000").await?;
    db.initialize_schema().await?;
    println!("Connected to SurrealDB");

    // Check if data already exists
    let existing_count = db.get_anime_count().await?;
    if existing_count > 0 {
        println!("Database already contains {} anime entries", existing_count);
        print!("Do you want to continue and add more? (y/n): ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborting import.");
            return Ok(());
        }
    }

    // Read the anime database file
    let file = File::open("data/anime-offline-database.json")?;
    let reader = BufReader::new(file);
    let database: AnimeOfflineDatabase = serde_json::from_reader(reader)?;
    println!("Loaded {} anime entries", database.data.len());

    // Track imported count
    let mut imported = 0;
    let mut skipped = 0;
    let mut errors = 0;

    // Import anime (limit to first 2000 for reasonable import time)
    let import_limit = std::env::var("IMPORT_LIMIT")
        .unwrap_or_else(|_| "2000".to_string())
        .parse::<usize>()
        .unwrap_or(2000);
    
    println!("Importing up to {} anime entries...", import_limit);
    
    for (index, entry) in database.data.iter().take(import_limit).enumerate() {
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
        match db.create_anime(&anime).await {
            Ok(_) => {
                imported += 1;
                if imported % 100 == 0 {
                    println!("Imported {} anime...", imported);
                }
            }
            Err(e) => {
                errors += 1;
                if errors <= 5 {
                    eprintln!("Failed to import '{}': {}", entry.title, e);
                }
            }
        }
        
        // Add slight delay to avoid overwhelming the database
        if index % 50 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    println!("\n=== Import Summary ===");
    println!("  Imported: {} anime", imported);
    println!("  Skipped: {} anime (no season/year data)", skipped);
    println!("  Errors: {} anime", errors);
    
    // Verify final count
    let total = db.get_anime_count().await?;
    println!("  Total in database: {} anime", total);
    
    // Create common tags
    println!("\nCreating tags...");
    let tag_categories = vec![
        ("Action", TagCategory::Genre),
        ("Adventure", TagCategory::Genre),
        ("Comedy", TagCategory::Genre),
        ("Drama", TagCategory::Genre),
        ("Fantasy", TagCategory::Genre),
        ("Horror", TagCategory::Genre),
        ("Mystery", TagCategory::Genre),
        ("Romance", TagCategory::Genre),
        ("Sci-Fi", TagCategory::Genre),
        ("Slice of Life", TagCategory::Genre),
        ("Sports", TagCategory::Genre),
        ("Thriller", TagCategory::Genre),
        ("Shounen", TagCategory::Demographic),
        ("Shoujo", TagCategory::Demographic),
        ("Seinen", TagCategory::Demographic),
        ("Josei", TagCategory::Demographic),
    ];
    
    for (name, category) in tag_categories {
        let tag = Tag {
            id: Uuid::new_v4(),
            name: name.to_string(),
            category,
            description: Some(format!("{} tag", name)),
            created_at: Utc::now(),
        };
        
        match db.create_tag(&tag).await {
            Ok(_) => println!("  Created tag: {}", name),
            Err(e) => eprintln!("  Failed to create tag '{}': {}", name, e),
        }
    }
    
    // Test queries
    println!("\nTesting queries...");
    
    // Test seasonal query
    let fall_2020 = db.get_seasonal_anime(2020, "fall").await?;
    println!("Found {} anime for Fall 2020", fall_2020.len());
    
    if fall_2020.len() > 0 {
        println!("Sample: {}", fall_2020[0].title);
        
        // Test similarity query
        let similar = db.get_similar_anime(fall_2020[0].id, 5).await?;
        println!("Found {} similar anime", similar.len());
    }
    
    // Test search
    if let Ok(search_results) = db.search_anime("attack").await {
        println!("Search for 'attack' returned {} results", search_results.len());
    }

    Ok(())
}