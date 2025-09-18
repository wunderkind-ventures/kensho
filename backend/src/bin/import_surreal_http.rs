use kensho_backend::{
    models::{Anime, AnimeStatus, AnimeType, AnimeSeason, Season, Tag, TagCategory},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
struct SqlQuery {
    query: String,
    vars: Option<serde_json::Value>,
}

async fn execute_query(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8000/sql")
        .basic_auth("root", Some("root"))
        .header("Accept", "application/json")
        .header("NS", "kensho")
        .header("DB", "anime")
        .body(query.to_string())
        .send()
        .await?;
    
    let text = response.text().await?;
    Ok(text)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting anime data import via HTTP API...");

    // Create namespace and database
    println!("Setting up database...");
    let _ = execute_query("DEFINE NAMESPACE IF NOT EXISTS kensho").await?;
    let _ = execute_query("USE NS kensho; DEFINE DATABASE IF NOT EXISTS anime").await?;
    let _ = execute_query("USE NS kensho; USE DB anime; DEFINE TABLE IF NOT EXISTS anime SCHEMAFULL").await?;
    let _ = execute_query("USE NS kensho; USE DB anime; DEFINE TABLE IF NOT EXISTS tag SCHEMAFULL").await?;
    
    println!("Database setup complete");

    // Read the anime database file
    let file = File::open("data/anime-offline-database.json")?;
    let reader = BufReader::new(file);
    let database: AnimeOfflineDatabase = serde_json::from_reader(reader)?;
    println!("Loaded {} anime entries", database.data.len());

    // Track imported count
    let mut imported = 0;
    let mut skipped = 0;

    // Import anime (limit to first 100 for testing)
    let import_limit = std::env::var("IMPORT_LIMIT")
        .unwrap_or_else(|_| "100".to_string())
        .parse::<usize>()
        .unwrap_or(100);
    
    println!("Importing up to {} anime entries...", import_limit);
    
    for entry in database.data.iter().take(import_limit) {
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
            "FINISHED" => "finished",
            "ONGOING" => "ongoing",
            "UPCOMING" => "upcoming",
            _ => "unknown",
        };
        
        // Map anime type
        let anime_type = match entry.anime_type.to_uppercase().as_str() {
            "TV" => "TV",
            "MOVIE" => "Movie",
            "OVA" => "OVA",
            "ONA" => "ONA",
            "SPECIAL" => "Special",
            _ => "Unknown",
        };
        
        // Map season
        let season = match season_raw.season.to_uppercase().as_str() {
            "SPRING" => "spring",
            "SUMMER" => "summer",
            "FALL" | "AUTUMN" => "fall",
            "WINTER" => "winter",
            _ => "spring",
        };
        
        // Create INSERT query
        let anime_id = Uuid::new_v4();
        let query = format!(r#"
            USE NS kensho; USE DB anime;
            CREATE anime:{} CONTENT {{
                id: "{}",
                title: "{}",
                synonyms: {:?},
                sources: {:?},
                episodes: {},
                status: "{}",
                anime_type: "{}",
                anime_season: {{
                    season: "{}",
                    year: {}
                }},
                synopsis: "{}",
                poster_url: "{}",
                imdb: null,
                created_at: time::now(),
                updated_at: time::now()
            }}
        "#,
            anime_id,
            anime_id,
            entry.title.replace('"', r#"\""#).replace('\n', " "),
            entry.synonyms,
            entry.sources,
            entry.episodes.unwrap_or(0),
            status,
            anime_type,
            season,
            year,
            format!("A {} anime with {} episodes. Studios: {}. Tags: {}", 
                entry.anime_type, 
                entry.episodes.unwrap_or(0),
                entry.studios.join(", "),
                entry.tags.join(", ")
            ).replace('"', r#"\""#).replace('\n', " "),
            entry.picture
        );
        
        // Execute insert
        match execute_query(&query).await {
            Ok(_) => {
                imported += 1;
                if imported % 20 == 0 {
                    println!("Imported {} anime...", imported);
                }
            }
            Err(e) => {
                eprintln!("Failed to import '{}': {}", entry.title, e);
            }
        }
    }

    println!("\n=== Import Summary ===");
    println!("  Imported: {} anime", imported);
    println!("  Skipped: {} anime (no season/year data)", skipped);
    
    // Verify count
    let count_result = execute_query("USE NS kensho; USE DB anime; SELECT count() as total FROM anime GROUP ALL").await?;
    println!("  Database response: {}", count_result);
    
    // Test a query
    println!("\nTesting seasonal query...");
    let test_result = execute_query("USE NS kensho; USE DB anime; SELECT * FROM anime WHERE anime_season.year = 2020 AND anime_season.season = 'fall' LIMIT 5").await?;
    println!("Query result preview: {}", &test_result[..test_result.len().min(200)]);

    Ok(())
}