// Setup proper SurrealDB schema for Kenshō project
use reqwest;

async fn execute_query(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8000/sql")
        .basic_auth("root", Some("root"))
        .header("Accept", "application/json")
        .body(query.to_string())
        .send()
        .await?;
    
    let text = response.text().await?;
    Ok(text)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting up SurrealDB schema for Kenshō...\n");

    // Create namespace and database
    println!("1. Creating namespace and database...");
    execute_query("DEFINE NAMESPACE IF NOT EXISTS kensho").await?;
    execute_query("USE NS kensho; DEFINE DATABASE IF NOT EXISTS anime").await?;
    
    // Remove existing tables to start fresh
    println!("2. Cleaning existing tables...");
    execute_query("USE NS kensho; USE DB anime; REMOVE TABLE IF EXISTS anime").await?;
    execute_query("USE NS kensho; USE DB anime; REMOVE TABLE IF EXISTS tag").await?;
    execute_query("USE NS kensho; USE DB anime; REMOVE TABLE IF EXISTS episode").await?;
    execute_query("USE NS kensho; USE DB anime; REMOVE TABLE IF EXISTS user").await?;
    
    // Define anime table with proper schema
    println!("3. Creating anime table with schema...");
    let anime_schema = r#"
        USE NS kensho; USE DB anime;
        
        -- Define anime table as SCHEMAFULL
        DEFINE TABLE anime TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        
        -- Core fields (required)
        DEFINE FIELD id ON anime TYPE string ASSERT $value != NONE;
        DEFINE FIELD title ON anime TYPE string ASSERT $value != NONE;
        DEFINE FIELD status ON anime TYPE string ASSERT $value != NONE;
        DEFINE FIELD anime_type ON anime TYPE string ASSERT $value != NONE;
        DEFINE FIELD episodes ON anime TYPE number DEFAULT 0;
        DEFINE FIELD poster_url ON anime TYPE string ASSERT $value != NONE;
        
        -- Season information (required nested object)
        DEFINE FIELD anime_season ON anime TYPE object ASSERT $value != NONE;
        DEFINE FIELD anime_season.season ON anime TYPE string ASSERT $value != NONE;
        DEFINE FIELD anime_season.year ON anime TYPE number ASSERT $value != NONE;
        
        -- Optional fields
        DEFINE FIELD synopsis ON anime TYPE option<string>;
        DEFINE FIELD synonyms ON anime TYPE array DEFAULT [];
        DEFINE FIELD sources ON anime TYPE array DEFAULT [];
        DEFINE FIELD imdb ON anime TYPE option<object>;
        
        -- Timestamps
        DEFINE FIELD created_at ON anime TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON anime TYPE datetime DEFAULT time::now();
        
        -- Indexes for performance (removed SEARCH ANALYZER ascii as it doesn't exist)
        DEFINE INDEX anime_title ON anime COLUMNS title;
        DEFINE INDEX anime_season_idx ON anime COLUMNS anime_season.year, anime_season.season;
        DEFINE INDEX anime_status_idx ON anime COLUMNS status;
        DEFINE INDEX anime_type_idx ON anime COLUMNS anime_type;
    "#;
    
    let result = execute_query(anime_schema).await?;
    println!("   Anime table created: {}", result.contains("OK"));
    
    // Define tag table
    println!("4. Creating tag table with schema...");
    let tag_schema = r#"
        USE NS kensho; USE DB anime;
        
        DEFINE TABLE tag TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        
        DEFINE FIELD id ON tag TYPE string ASSERT $value != NONE;
        DEFINE FIELD name ON tag TYPE string ASSERT $value != NONE;
        DEFINE FIELD category ON tag TYPE string ASSERT $value != NONE;
        DEFINE FIELD description ON tag TYPE option<string>;
        DEFINE FIELD created_at ON tag TYPE datetime DEFAULT time::now();
        
        DEFINE INDEX tag_name ON tag COLUMNS name UNIQUE;
        DEFINE INDEX tag_category ON tag COLUMNS category;
    "#;
    
    execute_query(tag_schema).await?;
    println!("   Tag table created");
    
    // Define episode table
    println!("5. Creating episode table with schema...");
    let episode_schema = r#"
        USE NS kensho; USE DB anime;
        
        DEFINE TABLE episode TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        
        DEFINE FIELD id ON episode TYPE string ASSERT $value != NONE;
        DEFINE FIELD anime_id ON episode TYPE string ASSERT $value != NONE;
        DEFINE FIELD episode_number ON episode TYPE number ASSERT $value != NONE;
        DEFINE FIELD title ON episode TYPE string;
        DEFINE FIELD synopsis ON episode TYPE option<string>;
        DEFINE FIELD air_date ON episode TYPE option<datetime>;
        DEFINE FIELD duration ON episode TYPE option<number>;
        DEFINE FIELD created_at ON episode TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON episode TYPE datetime DEFAULT time::now();
        
        DEFINE INDEX episode_anime ON episode COLUMNS anime_id;
        DEFINE INDEX episode_number_idx ON episode COLUMNS anime_id, episode_number UNIQUE;
    "#;
    
    execute_query(episode_schema).await?;
    println!("   Episode table created");
    
    // Define user table for personalization
    println!("6. Creating user table for personalization...");
    let user_schema = r#"
        USE NS kensho; USE DB anime;
        
        DEFINE TABLE user TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        
        DEFINE FIELD id ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD email ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD username ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON user TYPE datetime DEFAULT time::now();
        
        -- Flexible field for experiments
        DEFINE FIELD preferences ON user FLEXIBLE TYPE option<object>;
        DEFINE FIELD experiments ON user FLEXIBLE TYPE option<object>;
        
        DEFINE INDEX user_email ON user COLUMNS email UNIQUE;
        DEFINE INDEX user_username ON user COLUMNS username UNIQUE;
    "#;
    
    execute_query(user_schema).await?;
    println!("   User table created");
    
    // Define relationship tables (SCHEMALESS for flexibility)
    println!("7. Creating relationship tables...");
    let relationship_schema = r#"
        USE NS kensho; USE DB anime;
        
        -- Anime to tag relationships
        DEFINE TABLE has_tag TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
        
        -- User watch history
        DEFINE TABLE user_watched TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
        
        -- User likes/ratings
        DEFINE TABLE user_likes TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
        
        -- Anime similarity relationships
        DEFINE TABLE is_similar TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
        
        -- Sequential relationships
        DEFINE TABLE is_sequel TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
    "#;
    
    execute_query(relationship_schema).await?;
    println!("   Relationship tables created");
    
    // Verify schema
    println!("\n8. Verifying schema...");
    let info = execute_query("USE NS kensho; USE DB anime; INFO FOR DB;").await?;
    
    if info.contains("anime") && info.contains("tag") && info.contains("episode") && info.contains("user") {
        println!("✅ Schema setup complete!");
        
        // Test insert
        println!("\n9. Testing anime insert...");
        let test_insert = r#"
            USE NS kensho; USE DB anime;
            CREATE anime:test SET 
                id = 'test',
                title = 'Test Anime',
                status = 'completed',
                anime_type = 'TV',
                episodes = 12,
                poster_url = 'https://example.com/poster.jpg',
                anime_season = {
                    season: 'spring',
                    year: 2024
                },
                synopsis = 'A test anime entry'
        "#;
        
        let test_result = execute_query(test_insert).await?;
        if test_result.contains("test") && test_result.contains("Test Anime") {
            println!("✅ Test insert successful!");
            
            // Clean up test
            execute_query("USE NS kensho; USE DB anime; DELETE anime:test;").await?;
        } else {
            println!("⚠️ Test insert failed: {}", test_result);
        }
    } else {
        println!("❌ Schema verification failed");
    }
    
    println!("\n✨ Database ready for import!");
    Ok(())
}