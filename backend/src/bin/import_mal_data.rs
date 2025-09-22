use anyhow::{Result, Context};
use kensho_backend::{
    models::{
        AnimeOfflineDatabase, DeadEntriesDatabase,
        Tag, TagCategory,
        Anime,
        Episode
    },
    services::database_v2::DatabaseService,
};
use uuid::Uuid;
use chrono::Utc;
use tokio::time::{sleep, Duration};
use std::collections::{HashMap, HashSet};

const BATCH_SIZE: usize = 100;
const DELAY_MS: u64 = 50;

#[derive(Debug)]
struct ImportStats {
    total_entries: usize,
    dead_filtered: usize,
    imported_anime: usize,
    imported_tags: usize,
    imported_episodes: usize,
    tag_relationships: usize,
    anime_relationships: usize,
    errors: usize,
}

impl Default for ImportStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            dead_filtered: 0,
            imported_anime: 0,
            imported_tags: 0,
            imported_episodes: 0,
            tag_relationships: 0,
            anime_relationships: 0,
            errors: 0,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting comprehensive MAL data import to SurrealDB...\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let import_limit = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(5000)
    } else {
        5000
    };

    let skip_relationships = args.contains(&"--skip-relationships".to_string());
    let skip_episodes = args.contains(&"--skip-episodes".to_string());

    println!("📊 Configuration:");
    println!("  Import limit: {} entries", import_limit);
    println!("  Skip relationships: {}", skip_relationships);
    println!("  Skip episodes: {}", skip_episodes);
    println!();

    // Load data files
    println!("📂 Loading data files...");
    let anime_db = AnimeOfflineDatabase::load_from_file("../data/anime-offline-database.json")
        .context("Failed to load anime database")?;
    
    let dead_db = DeadEntriesDatabase::load_from_file("../data/myanimelist.json")
        .context("Failed to load dead entries database")?;

    let db_stats = anime_db.get_stats();
    let dead_stats = dead_db.get_stats();

    println!("  📈 Anime database: {} entries", db_stats.total_entries);
    println!("  💀 Dead entries: {} entries", dead_stats.total_count);
    println!();

    // Initialize database connection
    println!("🔌 Connecting to SurrealDB...");
    let db_url = std::env::var("SURREAL_URL").unwrap_or_else(|_| "ws://localhost:8000".to_string());
    let db = DatabaseService::new(&db_url).await?;
    db.initialize_schema().await?;
    println!("  ✅ Connected and schema initialized");

    // Check existing data
    let existing_count = db.get_anime_count().await?;
    if existing_count > 0 {
        println!("  ⚠️  Database already contains {} anime entries", existing_count);
        print!("  Continue anyway? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("  Aborting import.");
            return Ok(());
        }
    }
    println!();

    // Start import process
    let mut stats = ImportStats::default();
    stats.total_entries = anime_db.data.len().min(import_limit);

    // Filter out dead entries
    println!("🔍 Filtering out dead entries...");
    let limited_entries: Vec<_> = anime_db.data.iter().take(import_limit).cloned().collect();
    let live_entries = dead_db.filter_anime_entries(limited_entries);
    stats.dead_filtered = stats.total_entries - live_entries.len();
    println!("  💀 Filtered {} dead entries", stats.dead_filtered);
    println!("  ✅ {} live entries to import", live_entries.len());
    println!();

    // Phase 1: Import tags first
    if !skip_relationships {
        println!("🏷️  Phase 1: Creating tags...");
        stats.imported_tags = create_tags(&db, &live_entries).await?;
        println!("  ✅ Created {} unique tags", stats.imported_tags);
        println!();
    }

    // Phase 2: Import anime entries
    println!("📺 Phase 2: Importing anime entries...");
    let mut anime_id_map = HashMap::new();
    
    for (batch_idx, batch) in live_entries.chunks(BATCH_SIZE).enumerate() {
        println!("  📦 Processing batch {} ({}/{} entries)...", 
                batch_idx + 1, 
                batch_idx * BATCH_SIZE + batch.len(),
                live_entries.len());
        
        for entry in batch {
            match import_anime_entry(&db, entry).await {
                Ok(anime) => {
                    // Store MAL ID mapping for relationships
                    if let Some(mal_id) = entry.get_mal_id() {
                        anime_id_map.insert(mal_id, anime.id);
                    }
                    stats.imported_anime += 1;
                }
                Err(e) => {
                    eprintln!("    ❌ Failed to import '{}': {}", entry.title, e);
                    stats.errors += 1;
                }
            }
        }
        
        // Progress update and rate limiting
        if batch_idx % 10 == 0 {
            println!("    📈 Imported {} anime so far...", stats.imported_anime);
        }
        sleep(Duration::from_millis(DELAY_MS)).await;
    }
    
    println!("  ✅ Imported {} anime entries", stats.imported_anime);
    println!();

    // Phase 3: Create anime relationships
    if !skip_relationships {
        println!("🔗 Phase 3: Creating anime relationships...");
        stats.anime_relationships = create_anime_relationships(&db, &live_entries, &anime_id_map).await?;
        println!("  ✅ Created {} anime relationships", stats.anime_relationships);
        println!();

        // Phase 4: Create tag relationships
        println!("🏷️  Phase 4: Creating tag relationships...");
        stats.tag_relationships = create_tag_relationships(&db, &live_entries, &anime_id_map).await?;
        println!("  ✅ Created {} tag relationships", stats.tag_relationships);
        println!();
    }

    // Phase 5: Create episode records
    if !skip_episodes {
        println!("📺 Phase 5: Creating episode records...");
        stats.imported_episodes = create_episode_records(&db, &live_entries, &anime_id_map).await?;
        println!("  ✅ Created {} episode records", stats.imported_episodes);
        println!();
    }

    // Final verification
    println!("🔍 Final verification...");
    let final_anime_count = db.get_anime_count().await?;
    println!("  📊 Total anime in database: {}", final_anime_count);
    println!();

    // Print final summary
    print_import_summary(&stats);

    Ok(())
}

async fn import_anime_entry(db: &DatabaseService, entry: &kensho_backend::models::anime_offline_db::AnimeOfflineEntry) -> Result<Anime> {
    // Convert the offline entry to Kensho anime model
    let anime = entry.to_anime_model();
    
    // Create in database
    db.create_anime(&anime).await
        .context("Failed to create anime in database")
}

async fn create_tags(db: &DatabaseService, entries: &[kensho_backend::models::anime_offline_db::AnimeOfflineEntry]) -> Result<usize> {
    let mut all_tags = HashSet::new();
    
    // Collect all unique tags from entries
    for entry in entries {
        for tag in &entry.tags {
            all_tags.insert(tag.clone());
        }
        
        // Also add studio names as tags
        for studio in &entry.studios {
            if !studio.is_empty() {
                all_tags.insert(format!("Studio: {}", studio));
            }
        }
    }
    
    println!("    🔍 Found {} unique tags", all_tags.len());
    
    // Create tag records
    let mut created = 0;
    for tag_name in all_tags {
        let category = classify_tag(&tag_name);
        let tag = Tag {
            id: Uuid::new_v4(),
            name: tag_name.clone(),
            category: category.clone(),
            description: Some(generate_tag_description(&tag_name, &category)),
            created_at: Utc::now(),
        };
        
        // Try to create the tag (ignore errors for duplicates)
        if let Err(_) = db.create_tag(&tag).await {
            // Tag might already exist, skip
            continue;
        }
        
        created += 1;
        if created % 100 == 0 {
            println!("    📝 Created {} tags...", created);
        }
    }
    
    Ok(created)
}

async fn create_anime_relationships(
    db: &DatabaseService,
    entries: &[kensho_backend::models::anime_offline_db::AnimeOfflineEntry],
    anime_id_map: &HashMap<String, Uuid>
) -> Result<usize> {
    let mut relationships_created = 0;
    
    for entry in entries {
        let Some(source_mal_id) = entry.get_mal_id() else { continue };
        let Some(&source_anime_id) = anime_id_map.get(&source_mal_id) else { continue };
        
        // Process related anime URLs
        for related_url in &entry.related_anime {
            if let Some(related_mal_id) = extract_mal_id_from_url(related_url) {
                if let Some(&related_anime_id) = anime_id_map.get(&related_mal_id) {
                    // Create bidirectional relationship
                    if let Ok(_) = db.create_anime_relationship(source_anime_id, related_anime_id, "related").await {
                        relationships_created += 1;
                    }
                }
            }
        }
    }
    
    Ok(relationships_created)
}

async fn create_tag_relationships(
    db: &DatabaseService,
    entries: &[kensho_backend::models::anime_offline_db::AnimeOfflineEntry],
    anime_id_map: &HashMap<String, Uuid>
) -> Result<usize> {
    let mut relationships_created = 0;
    
    for entry in entries {
        let Some(mal_id) = entry.get_mal_id() else { continue };
        let Some(&anime_id) = anime_id_map.get(&mal_id) else { continue };
        
        // Create tag relationships
        for tag_name in &entry.tags {
            if let Ok(tag_id) = db.find_tag_by_name(tag_name).await {
                let relevance = calculate_tag_relevance(tag_name, entry);
                if let Ok(_) = db.create_anime_tag_relationship(anime_id, tag_id, relevance).await {
                    relationships_created += 1;
                }
            }
        }
        
        // Create studio tag relationships
        for studio in &entry.studios {
            if !studio.is_empty() {
                let studio_tag_name = format!("Studio: {}", studio);
                if let Ok(tag_id) = db.find_tag_by_name(&studio_tag_name).await {
                    if let Ok(_) = db.create_anime_tag_relationship(anime_id, tag_id, 1.0).await {
                        relationships_created += 1;
                    }
                }
            }
        }
    }
    
    Ok(relationships_created)
}

async fn create_episode_records(
    db: &DatabaseService,
    entries: &[kensho_backend::models::anime_offline_db::AnimeOfflineEntry],
    anime_id_map: &HashMap<String, Uuid>
) -> Result<usize> {
    let mut episodes_created = 0;
    
    for entry in entries {
        let Some(mal_id) = entry.get_mal_id() else { continue };
        let Some(&anime_id) = anime_id_map.get(&mal_id) else { continue };
        
        let episode_count = entry.episodes.max(0) as u32;
        if episode_count == 0 { continue; }
        
        // Create episode records based on episode count
        for episode_num in 1..=episode_count.min(500) { // Cap at 500 episodes for sanity
            let episode = Episode {
                id: Uuid::new_v4(),
                anime_id,
                episode_number: episode_num,
                title: Some(format!("Episode {}", episode_num)),
                duration: entry.duration.as_ref().map(|d| d.in_seconds() as u32),
                air_date: None, // Could be calculated from season data
                synopsis: None,
                thumbnail_url: Some(entry.thumbnail.clone()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            
            if let Ok(_) = db.create_episode(&episode).await {
                episodes_created += 1;
            }
        }
    }
    
    Ok(episodes_created)
}

// Helper functions

fn classify_tag(tag_name: &str) -> TagCategory {
    let tag_lower = tag_name.to_lowercase();
    
    // Studio tags
    if tag_lower.starts_with("studio:") {
        return TagCategory::Content;
    }
    
    // Common genres
    let genres = ["action", "adventure", "comedy", "drama", "fantasy", "horror", 
                  "mystery", "romance", "sci-fi", "thriller", "slice of life",
                  "sports", "supernatural", "psychological", "historical"];
    if genres.iter().any(|&genre| tag_lower.contains(genre)) {
        return TagCategory::Genre;
    }
    
    // Demographics
    let demographics = ["shounen", "shoujo", "seinen", "josei", "kids", "children"];
    if demographics.iter().any(|&demo| tag_lower.contains(demo)) {
        return TagCategory::Demographic;
    }
    
    // Themes
    let themes = ["school", "military", "magic", "mecha", "music", "game", "work"];
    if themes.iter().any(|&theme| tag_lower.contains(theme)) {
        return TagCategory::Theme;
    }
    
    // Default to Genre for unknown tags
    TagCategory::Genre
}

fn generate_tag_description(tag_name: &str, category: &TagCategory) -> String {
    if tag_name.starts_with("Studio: ") {
        format!("Animation studio: {}", &tag_name[8..])
    } else {
        match category {
            TagCategory::Genre => format!("Genre: {}", tag_name),
            TagCategory::Theme => format!("Theme: {}", tag_name),
            TagCategory::Demographic => format!("Target demographic: {}", tag_name),
            TagCategory::Content => format!("Content tag: {}", tag_name),
        }
    }
}

fn extract_mal_id_from_url(url: &str) -> Option<String> {
    if url.contains("myanimelist.net") {
        url.split('/').last().map(|s| s.to_string())
    } else {
        None
    }
}

fn calculate_tag_relevance(tag_name: &str, entry: &kensho_backend::models::anime_offline_db::AnimeOfflineEntry) -> f32 {
    // Simple relevance calculation - could be more sophisticated
    let tag_count = entry.tags.len() as f32;
    if tag_count == 0.0 { return 1.0; }
    
    // Base relevance inversely proportional to number of tags
    let base_relevance = (10.0 / tag_count.max(1.0)).min(1.0);
    
    // Boost relevance for certain important tags
    let tag_lower = tag_name.to_lowercase();
    if ["action", "drama", "comedy", "romance"].contains(&tag_lower.as_str()) {
        base_relevance * 1.2
    } else {
        base_relevance
    }.min(1.0)
}

fn print_import_summary(stats: &ImportStats) {
    println!("🎉 Import completed successfully!");
    println!();
    println!("📊 Final Statistics:");
    println!("  📺 Anime imported: {}", stats.imported_anime);
    println!("  🏷️  Tags created: {}", stats.imported_tags);
    println!("  📹 Episodes created: {}", stats.imported_episodes);
    println!("  🔗 Anime relationships: {}", stats.anime_relationships);
    println!("  🏷️  Tag relationships: {}", stats.tag_relationships);
    println!("  💀 Dead entries filtered: {}", stats.dead_filtered);
    println!("  ❌ Errors encountered: {}", stats.errors);
    println!();
    
    let success_rate = if stats.total_entries > 0 {
        (stats.imported_anime as f64 / stats.total_entries as f64) * 100.0
    } else {
        0.0
    };
    println!("✅ Success rate: {:.1}%", success_rate);
    
    if stats.errors > 0 {
        println!("⚠️  {} errors encountered during import", stats.errors);
    }
}
