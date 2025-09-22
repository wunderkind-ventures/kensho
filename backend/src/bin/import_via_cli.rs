use std::fs;
use std::process::Command;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting data import via SurrealDB CLI...");
    
    // First ensure SurrealDB is running
    println!("🔄 Starting SurrealDB if not already running...");
    let output = Command::new("docker")
        .args(&["start", "kensho-surrealdb"])
        .output()?;
    
    if !output.status.success() {
        println!("⚠️  Could not start container, trying docker-compose...");
        Command::new("docker-compose")
            .args(&["up", "-d", "surrealdb"])
            .current_dir("/Users/kennethsylvain/WKV/Enterprise/kensho")
            .output()?;
    }
    
    // Wait for SurrealDB to be ready
    println!("⏳ Waiting for SurrealDB to be ready...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Read and process the JSON file
    let json_path = "../data/anime-offline-database.json";
    println!("📖 Reading JSON from: {}", json_path);
    let json_content = fs::read_to_string(json_path)?;
    let data: Value = serde_json::from_str(&json_content)?;
    let anime_list = data["data"].as_array()
        .ok_or("Expected 'data' to be an array")?;
    
    println!("📊 Found {} anime entries", anime_list.len());
    
    // Generate import statements
    let mut statements = Vec::new();
    statements.push("USE NS kensho DB anime;".to_string());
    
    // Import first 500 entries
    let limit = std::cmp::min(500, anime_list.len());
    println!("💾 Importing {} entries...", limit);
    
    for anime in anime_list.iter().take(limit) {
        let id = anime["sources"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|s| s.as_str())
            .and_then(|s| s.split('/').last())
            .unwrap_or("unknown");
        
        let title = anime["title"].as_str().unwrap_or("Unknown")
            .replace("'", "''").replace("\\", "\\\\");
        let anime_type = anime["type"].as_str().unwrap_or("UNKNOWN");
        let episodes = anime["episodes"].as_u64().unwrap_or(0) as i32;
        let status = anime["status"].as_str().unwrap_or("UNKNOWN");
        let season = anime["animeSeason"]["season"].as_str().unwrap_or("UNDEFINED");
        let year = anime["animeSeason"]["year"].as_u64().unwrap_or(0) as i32;
        let picture = anime["picture"].as_str().unwrap_or("");
        let thumbnail = anime["thumbnail"].as_str().unwrap_or("");
        
        statements.push(format!(
            "CREATE anime:{} SET title = '{}', anime_type = '{}', episodes = {}, status = '{}', anime_season = {{ season: '{}', year: {} }}, picture = '{}', thumbnail = '{}';",
            id.replace("-", "_"), title, anime_type, episodes, status, season, year, picture, thumbnail
        ));
    }
    
    // Write to temporary file
    let sql_file = "/tmp/import_anime.sql";
    fs::write(sql_file, statements.join("\n"))?;
    println!("📝 Written {} statements to {}", statements.len(), sql_file);
    
    // Execute import using surreal CLI
    println!("🔧 Executing import using surreal CLI...");
    let mut output = Command::new("docker")
        .args(&[
            "exec", "-i", "kensho-surrealdb",
            "surreal", "sql",
            "--conn", "http://localhost:8000",
            "--user", "root",
            "--pass", "root",
            "--ns", "kensho",
            "--db", "anime"
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    
    // Send the SQL content
    if let Some(mut stdin) = output.stdin.take() {
        use std::io::Write;
        stdin.write_all(statements.join("\n").as_bytes())?;
        drop(stdin); // Explicitly close stdin
    }
    
    // Wait for completion
    let result = output.wait_with_output()?;
    
    if result.status.success() {
        println!("✅ Import completed successfully!");
        
        // Verify the count
        println!("\n🔍 Verifying import...");
        let count_output = Command::new("docker")
            .args(&[
                "exec", "kensho-surrealdb",
                "surreal", "sql",
                "--conn", "http://localhost:8000",
                "--user", "root",
                "--pass", "root",
                "--ns", "kensho",
                "--db", "anime",
                "--query", "SELECT count() FROM anime GROUP ALL"
            ])
            .output()?;
        
        let count_result = String::from_utf8_lossy(&count_output.stdout);
        println!("📊 Result: {}", count_result);
    } else {
        println!("❌ Import failed!");
        println!("Error: {}", String::from_utf8_lossy(&result.stderr));
    }
    
    Ok(())
}