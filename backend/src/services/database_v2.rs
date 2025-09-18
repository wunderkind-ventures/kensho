// Database service for SurrealDB v2 operations
// Fixed for SurrealDB 2.1 API changes

use anyhow::{Result, Context};
use surrealdb::{Surreal, Response};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::models::{
    Anime, AnimeSummary, Episode, Tag,
    HasTag, IsSequelOf, RelatedTo
};

pub struct DatabaseService {
    db: Surreal<Client>,
}

impl DatabaseService {
    pub async fn new(url: &str) -> Result<Self> {
        // Connect to SurrealDB
        let db = Surreal::new::<Ws>(url).await?;
        
        // Sign in as root user (use env vars in production)
        let username = std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string());
        let password = std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string());
        
        db.signin(Root {
            username: &username,
            password: &password,
        }).await?;
        
        // Create namespace if it doesn't exist
        let _: surrealdb::Response = db.query("DEFINE NAMESPACE IF NOT EXISTS kensho").await?;
        
        // Use namespace and create database if it doesn't exist
        db.use_ns("kensho").await?;
        let _: surrealdb::Response = db.query("DEFINE DATABASE IF NOT EXISTS anime").await?;
        
        // Now use the database
        db.use_ns("kensho").use_db("anime").await?;
        
        Ok(DatabaseService { db })
    }
    
    pub async fn initialize_schema(&self) -> Result<()> {
        // Create tables with proper result handling for v2
        self.db.query("DEFINE TABLE IF NOT EXISTS anime SCHEMAFULL")
            .await?
            .check()?;
            
        self.db.query("DEFINE TABLE IF NOT EXISTS episode SCHEMAFULL")
            .await?
            .check()?;
            
        self.db.query("DEFINE TABLE IF NOT EXISTS tag SCHEMAFULL")
            .await?
            .check()?;
            
        self.db.query("DEFINE TABLE IF NOT EXISTS user SCHEMAFULL")
            .await?
            .check()?;
        
        // Define indexes
        self.db.query("DEFINE INDEX IF NOT EXISTS anime_title ON anime FIELDS title SEARCH ANALYZER ascii TOKENIZERS lowercase, class")
            .await?
            .check()?;
            
        self.db.query("DEFINE INDEX IF NOT EXISTS anime_season ON anime FIELDS anime_season.year, anime_season.season")
            .await?
            .check()?;
            
        self.db.query("DEFINE INDEX IF NOT EXISTS episode_anime ON episode FIELDS anime_id")
            .await?
            .check()?;
        
        // Define graph edge tables for relationships
        self.db.query("DEFINE TABLE IF NOT EXISTS has_tag SCHEMAFULL")
            .await?
            .check()?;
            
        self.db.query("DEFINE TABLE IF NOT EXISTS is_sequel SCHEMAFULL")
            .await?
            .check()?;
            
        self.db.query("DEFINE TABLE IF NOT EXISTS is_similar SCHEMAFULL")
            .await?
            .check()?;
            
        self.db.query("DEFINE TABLE IF NOT EXISTS user_watched SCHEMAFULL")
            .await?
            .check()?;
            
        self.db.query("DEFINE TABLE IF NOT EXISTS user_likes SCHEMAFULL")
            .await?
            .check()?;
        
        Ok(())
    }
    
    // Anime CRUD operations
    pub async fn create_anime(&self, anime: &Anime) -> Result<Anime> {
        let anime_clone = anime.clone();
        let created: Option<Anime> = self.db
            .create(("anime", anime.id.to_string()))
            .content(anime_clone)
            .await?;
        
        created.context("Failed to create anime")
    }
    
    pub async fn get_anime(&self, id: Uuid) -> Result<Option<Anime>> {
        let anime: Option<Anime> = self.db
            .select(("anime", id.to_string()))
            .await?;
        
        Ok(anime)
    }
    
    pub async fn update_anime(&self, anime: &Anime) -> Result<Anime> {
        let anime_clone = anime.clone();
        let updated: Option<Anime> = self.db
            .update(("anime", anime.id.to_string()))
            .content(anime_clone)
            .await?;
        
        updated.context("Failed to update anime")
    }
    
    pub async fn delete_anime(&self, id: Uuid) -> Result<()> {
        let _: Option<Anime> = self.db
            .delete(("anime", id.to_string()))
            .await?;
        
        Ok(())
    }
    
    // Search operations
    pub async fn search_anime(&self, query: &str) -> Result<Vec<AnimeSummary>> {
        let query_string = query.to_string();
        let mut response = self.db
            .query("SELECT * FROM anime WHERE title @@ $query OR $query IN synonyms LIMIT 20")
            .bind(("query", query_string))
            .await?;
        
        let anime: Vec<Anime> = response.take(0)?;
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    pub async fn get_seasonal_anime(&self, year: u16, season: &str) -> Result<Vec<AnimeSummary>> {
        let mut response = self.db
            .query("SELECT * FROM anime WHERE anime_season.year = $year AND anime_season.season = $season ORDER BY title")
            .bind(("year", year as i64))
            .bind(("season", season.to_lowercase()))
            .await?;
        
        let anime: Vec<Anime> = response.take(0)?;
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    pub async fn list_anime(&self, limit: usize, offset: usize) -> Result<Vec<AnimeSummary>> {
        let mut response = self.db
            .query("SELECT * FROM anime ORDER BY created_at DESC LIMIT $limit START $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        
        let anime: Vec<Anime> = response.take(0)?;
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    pub async fn get_anime_count(&self) -> Result<usize> {
        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }
        
        let mut response = self.db
            .query("SELECT count() as count FROM anime GROUP ALL")
            .await?;
        
        let result: Option<CountResult> = response.take(0)?;
        Ok(result.map(|r| r.count as usize).unwrap_or(0))
    }
    
    // Graph relationship operations for recommendations
    pub async fn create_anime_tag_relationship(&self, anime_id: Uuid, tag_id: Uuid, relevance: f32) -> Result<()> {
        self.db
            .query("RELATE $anime->has_tag->$tag SET relevance = $relevance, created_at = time::now()")
            .bind(("anime", format!("anime:{}", anime_id)))
            .bind(("tag", format!("tag:{}", tag_id)))
            .bind(("relevance", relevance))
            .await?
            .check()?;
        
        Ok(())
    }
    
    pub async fn create_sequel_relationship(&self, sequel_id: Uuid, prequel_id: Uuid) -> Result<()> {
        self.db
            .query("RELATE $prequel->is_sequel->$sequel SET created_at = time::now()")
            .bind(("prequel", format!("anime:{}", prequel_id)))
            .bind(("sequel", format!("anime:{}", sequel_id)))
            .await?
            .check()?;
        
        Ok(())
    }
    
    pub async fn create_similarity_relationship(&self, anime1_id: Uuid, anime2_id: Uuid, similarity_score: f32) -> Result<()> {
        self.db
            .query("RELATE $anime1->is_similar->$anime2 SET score = $score, created_at = time::now()")
            .bind(("anime1", format!("anime:{}", anime1_id)))
            .bind(("anime2", format!("anime:{}", anime2_id)))
            .bind(("score", similarity_score))
            .await?
            .check()?;
        
        Ok(())
    }
    
    // Recommendation queries using graph traversal
    pub async fn get_similar_anime(&self, anime_id: Uuid, limit: usize) -> Result<Vec<AnimeSummary>> {
        // Get anime with similar tags (2-hop graph traversal)
        let mut response = self.db
            .query(r#"
                SELECT * FROM anime 
                WHERE id IN (
                    SELECT out FROM has_tag 
                    WHERE in IN (
                        SELECT in FROM has_tag 
                        WHERE out = $anime_id
                    ) AND out != $anime_id
                )
                LIMIT $limit
            "#)
            .bind(("anime_id", format!("anime:{}", anime_id)))
            .bind(("limit", limit))
            .await?;
        
        let anime: Vec<Anime> = response.take(0)?;
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    pub async fn get_recommendations_for_user(&self, user_id: Uuid, limit: usize) -> Result<Vec<AnimeSummary>> {
        // Get recommendations based on user's watch history and preferences
        let mut response = self.db
            .query(r#"
                LET $watched = (SELECT out FROM user_watched WHERE in = $user_id);
                LET $liked_tags = (
                    SELECT DISTINCT out FROM has_tag 
                    WHERE in IN (
                        SELECT out FROM user_likes WHERE in = $user_id
                    )
                );
                
                SELECT * FROM anime 
                WHERE id NOT IN $watched
                AND id IN (
                    SELECT DISTINCT in FROM has_tag 
                    WHERE out IN $liked_tags
                )
                ORDER BY (
                    SELECT count() FROM has_tag 
                    WHERE in = anime.id AND out IN $liked_tags
                ) DESC
                LIMIT $limit
            "#)
            .bind(("user_id", format!("user:{}", user_id)))
            .bind(("limit", limit))
            .await?;
        
        let anime: Vec<Anime> = response.take(0)?;
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    // User interaction tracking for personalization
    pub async fn track_user_watched(&self, user_id: Uuid, anime_id: Uuid, episode: u32) -> Result<()> {
        self.db
            .query(r#"
                RELATE $user->user_watched->$anime 
                SET episode = $episode, 
                    watched_at = time::now(),
                    completed = $episode >= (SELECT episodes FROM $anime)
            "#)
            .bind(("user", format!("user:{}", user_id)))
            .bind(("anime", format!("anime:{}", anime_id)))
            .bind(("episode", episode))
            .await?
            .check()?;
        
        Ok(())
    }
    
    pub async fn track_user_likes(&self, user_id: Uuid, anime_id: Uuid, rating: f32) -> Result<()> {
        self.db
            .query(r#"
                RELATE $user->user_likes->$anime 
                SET rating = $rating,
                    liked_at = time::now()
            "#)
            .bind(("user", format!("user:{}", user_id)))
            .bind(("anime", format!("anime:{}", anime_id)))
            .bind(("rating", rating))
            .await?
            .check()?;
        
        // Update similarity relationships based on user preferences
        self.update_similarities_from_user_preference(user_id, anime_id).await?;
        
        Ok(())
    }
    
    async fn update_similarities_from_user_preference(&self, user_id: Uuid, anime_id: Uuid) -> Result<()> {
        // Find other anime this user liked and increase their similarity scores
        self.db
            .query(r#"
                LET $other_liked = (
                    SELECT out FROM user_likes 
                    WHERE in = $user AND out != $anime AND rating >= 4.0
                );
                
                FOR $other IN $other_liked {
                    IF NOT EXISTS (SELECT * FROM is_similar WHERE in = $anime AND out = $other) {
                        RELATE $anime->is_similar->$other SET score = 0.5, created_at = time::now();
                    } ELSE {
                        UPDATE is_similar SET score += 0.1 
                        WHERE in = $anime AND out = $other AND score < 1.0;
                    }
                }
            "#)
            .bind(("user", format!("user:{}", user_id)))
            .bind(("anime", format!("anime:{}", anime_id)))
            .await?
            .check()?;
        
        Ok(())
    }
    
    // Batch import optimizations
    pub async fn batch_create_anime(&self, anime_list: Vec<Anime>) -> Result<usize> {
        let mut count = 0;
        
        // Use transaction for consistency
        self.db.query("BEGIN TRANSACTION").await?.check()?;
        
        for anime in anime_list {
            match self.create_anime(&anime).await {
                Ok(_) => count += 1,
                Err(e) => {
                    tracing::warn!("Failed to import anime '{}': {}", anime.title, e);
                }
            }
        }
        
        self.db.query("COMMIT TRANSACTION").await?.check()?;
        
        Ok(count)
    }
    
    // Episode operations
    pub async fn create_episode(&self, episode: &Episode) -> Result<Episode> {
        let episode_clone = episode.clone();
        let created: Option<Episode> = self.db
            .create(("episode", episode.id.to_string()))
            .content(episode_clone)
            .await?;
        
        created.context("Failed to create episode")
    }
    
    pub async fn get_anime_episodes(&self, anime_id: Uuid) -> Result<Vec<Episode>> {
        let mut response = self.db
            .query("SELECT * FROM episode WHERE anime_id = $anime_id ORDER BY episode_number")
            .bind(("anime_id", anime_id))
            .await?;
        
        let episodes: Vec<Episode> = response.take(0)?;
        Ok(episodes)
    }
    
    // Tag operations
    pub async fn create_tag(&self, tag: &Tag) -> Result<Tag> {
        let tag_clone = tag.clone();
        let created: Option<Tag> = self.db
            .create(("tag", tag.id.to_string()))
            .content(tag_clone)
            .await?;
        
        created.context("Failed to create tag")
    }
    
    pub async fn get_tags(&self) -> Result<Vec<Tag>> {
        let tags: Vec<Tag> = self.db
            .select("tag")
            .await?;
        
        Ok(tags)
    }
    
    pub async fn get_anime_tags(&self, anime_id: Uuid) -> Result<Vec<Tag>> {
        let mut response = self.db
            .query("SELECT out.* FROM has_tag WHERE in = $anime_id")
            .bind(("anime_id", format!("anime:{}", anime_id)))
            .await?;
        
        let tags: Vec<Tag> = response.take(0)?;
        Ok(tags)
    }
}