use anyhow::{Result, Context};
use kensho_backend::models::{AnimeOfflineDatabase, DeadEntriesDatabase};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing MAL/Anime Offline Database Types\n");

    // Test loading the anime offline database
    let anime_db_path = "../data/anime-offline-database.json";
    match test_anime_offline_db(anime_db_path).await {
        Ok(_) => println!("✅ Anime offline database test passed"),
        Err(e) => println!("❌ Anime offline database test failed: {}", e),
    }

    println!();

    // Test loading the dead entries database  
    let dead_entries_path = "../data/myanimelist.json";
    match test_dead_entries_db(dead_entries_path).await {
        Ok(_) => println!("✅ Dead entries database test passed"),
        Err(e) => println!("❌ Dead entries database test failed: {}", e),
    }

    Ok(())
}

async fn test_anime_offline_db(path: &str) -> Result<()> {
    println!("Loading anime offline database from: {}", path);
    
    let db = AnimeOfflineDatabase::load_from_file(path)
        .context("Failed to load anime offline database")?;
    
    println!("📊 Database Info:");
    println!("  Schema: {}", db.schema);
    println!("  Repository: {}", db.repository);
    println!("  Last Update: {}", db.last_update);
    println!("  Score Range: {:.1} - {:.1}", db.score_range.min_inclusive, db.score_range.max_inclusive);
    
    // Get statistics
    let stats = db.get_stats();
    println!("  Total Entries: {}", stats.total_entries);
    println!("  TV Series: {}", stats.tv_series);
    println!("  Movies: {}", stats.movies);
    println!("  Finished: {}", stats.finished);
    println!("  Ongoing: {}", stats.ongoing);
    println!("  With Scores: {}", stats.with_scores);
    println!("  Average Score: {:.2}", stats.average_score);

    // Show a few sample entries
    println!("\n🎬 Sample Entries:");
    for (i, entry) in db.data.iter().take(5).enumerate() {
        println!("  {}. {} ({})", i + 1, entry.title, entry.anime_type.to_anime_type());
        println!("     Episodes: {}, Status: {}", entry.episodes, entry.status.to_anime_status());
        if let Some(mal_id) = entry.get_mal_id() {
            println!("     MAL ID: {}", mal_id);
        }
        if let Some(score) = &entry.score {
            println!("     Score: {:.2}", score.best_score());
        }
        println!("     Studios: {}", entry.studios.join(", "));
        println!();
    }

    // Test filtering
    let high_rated = db.filter_by_min_score(8.5);
    println!("🌟 High-rated anime (>= 8.5): {}", high_rated.len());
    
    if let Some(top_anime) = high_rated.first() {
        println!("  Top example: {} (Score: {:.2})", 
                 top_anime.title, 
                 top_anime.score.as_ref().unwrap().best_score());
    }

    // Test conversion to Kensho models (just a small sample)
    println!("\n🔄 Testing conversion to Kensho models...");
    let sample_entries: Vec<_> = db.data.iter().take(3).collect();
    let kensho_models: Vec<_> = sample_entries.iter().map(|entry| entry.to_anime_model()).collect();
    
    println!("  Converted {} entries to Kensho format", kensho_models.len());
    for model in kensho_models.iter().take(2) {
        println!("    - {} (ID: {})", model.title, model.id);
    }

    Ok(())
}

async fn test_dead_entries_db(path: &str) -> Result<()> {
    println!("Loading dead entries database from: {}", path);
    
    let db = DeadEntriesDatabase::load_from_file(path)
        .context("Failed to load dead entries database")?;
    
    println!("📊 Dead Entries Info:");
    println!("  Schema: {}", db.schema);
    println!("  Repository: {}", db.repository);
    println!("  Last Update: {}", db.last_update);
    
    // Get statistics
    let stats = db.get_stats();
    println!("  Total Dead Entries: {}", stats.total_count);
    println!("  Numeric IDs: {}", stats.numeric_count);
    println!("  Non-numeric IDs: {}", stats.non_numeric_count);
    println!("  ID Range: {} - {}", stats.min_id, stats.max_id);

    // Show some sample dead entries
    println!("\n💀 Sample Dead Entries:");
    for (i, entry) in db.dead_entries.iter().take(10).enumerate() {
        print!("  {}", entry);
        if (i + 1) % 5 == 0 {
            println!();
        } else {
            print!(", ");
        }
    }
    println!();

    // Test lookup functionality
    println!("\n🔍 Testing lookup functionality:");
    let test_ids = vec!["10", "12345", "99999"];
    for test_id in test_ids {
        let is_dead = db.is_dead_entry(test_id);
        println!("  ID {} is {}", test_id, if is_dead { "💀 DEAD" } else { "✅ alive" });
    }

    // Test numeric ID lookup
    let test_numeric_ids = vec![10u32, 12345, 99999];
    for test_id in test_numeric_ids {
        let is_dead = db.is_dead_entry_id(test_id);
        println!("  ID {} is {}", test_id, if is_dead { "💀 DEAD" } else { "✅ alive" });
    }

    Ok(())
}
