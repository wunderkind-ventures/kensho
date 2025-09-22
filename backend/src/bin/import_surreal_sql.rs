use std::fs;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting SurrealQL import process...");
    
    // Read the JSON file
    let json_path = "../data/anime-offline-database.json";
    println!("📖 Reading JSON from: {}", json_path);
    let json_content = fs::read_to_string(json_path)?;
    
    // Parse JSON
    let data: Value = serde_json::from_str(&json_content)?;
    let anime_list = data["data"].as_array()
        .ok_or("Expected 'data' to be an array")?;
    
    println!("📊 Found {} anime entries to import", anime_list.len());
    
    // Create SurrealQL statements
    let mut sql_statements = Vec::new();
    
    // Add namespace and database selection
    sql_statements.push("USE NS kensho DB anime;".to_string());
    
    // Process first 100 entries as a test
    let limit = std::cmp::min(100, anime_list.len());
    println!("💾 Generating SQL for first {} entries...", limit);
    
    for anime in anime_list.iter().take(limit) {
        let id = anime["sources"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|s| s.as_str())
            .map(|s| s.split('/').last().unwrap_or("unknown"))
            .unwrap_or("unknown");
        
        let title = anime["title"].as_str().unwrap_or("Unknown")
            .replace("'", "''");
        
        let anime_type = anime["type"].as_str().unwrap_or("UNKNOWN");
        let episodes = anime["episodes"].as_u64().unwrap_or(0) as i32;
        let status = anime["status"].as_str().unwrap_or("UNKNOWN");
        
        let season = anime["animeSeason"]["season"].as_str().unwrap_or("UNDEFINED");
        let year = anime["animeSeason"]["year"].as_u64().unwrap_or(0) as i32;
        
        let picture = anime["picture"].as_str().unwrap_or("");
        let thumbnail = anime["thumbnail"].as_str().unwrap_or("");
        
        let synonyms: Vec<String> = anime["synonyms"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.replace("'", "''"))
                .collect())
            .unwrap_or_default();
        
        let tags: Vec<String> = anime["tags"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.replace("'", "''"))
                .collect())
            .unwrap_or_default();
        
        // Create the INSERT statement
        let sql = format!(
            r#"CREATE anime:{} CONTENT {{
    title: '{}',
    anime_type: '{}',
    episodes: {},
    status: '{}',
    anime_season: {{
        season: '{}',
        year: {}
    }},
    picture: '{}',
    thumbnail: '{}',
    synonyms: [{}],
    tags: [{}],
    sources: ['{}']
}};"#,
            id.replace("-", "_"),
            title,
            anime_type,
            episodes,
            status,
            season,
            year,
            picture,
            thumbnail,
            synonyms.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "),
            tags.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "),
            format!("https://myanimelist.net/anime/{}", id)
        );
        
        sql_statements.push(sql);
    }
    
    // Write to SQL file
    let sql_file = "../data/import_anime.surql";
    let full_sql = sql_statements.join("\n\n");
    fs::write(sql_file, &full_sql)?;
    
    println!("✅ Generated SQL file: {}", sql_file);
    println!("📝 Total statements: {}", sql_statements.len());
    
    // Now execute using curl
    println!("\n🔄 Executing import via HTTP API...");
    
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8000/sql")
        .header("Accept", "application/json")
        .header("NS", "kensho")
        .header("DB", "anime")
        .basic_auth("root", Some("root"))
        .body(full_sql)
        .send()
        .await?;
    
    if response.status().is_success() {
        let result: Value = response.json().await?;
        println!("✅ Import response: {}", serde_json::to_string_pretty(&result)?);
        
        // Verify the count
        println!("\n🔍 Verifying import...");
        let count_response = client
            .post("http://localhost:8000/sql")
            .header("Accept", "application/json")
            .header("NS", "kensho")
            .header("DB", "anime")
            .basic_auth("root", Some("root"))
            .body("SELECT count() FROM anime GROUP ALL")
            .send()
            .await?;
        
        if count_response.status().is_success() {
            let count_result: Value = count_response.json().await?;
            println!("📊 Count result: {}", serde_json::to_string_pretty(&count_result)?);
        }
    } else {
        println!("❌ Import failed: {}", response.status());
        println!("Response: {}", response.text().await?);
    }
    
    Ok(())
}