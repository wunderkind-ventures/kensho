// Database service for SurrealDB operations

use anyhow::{Result, Context};
use surrealdb::Surreal;
use surrealdb::engine::any::{connect, Any};
use uuid::Uuid;
use crate::models::{
    Anime, AnimeSummary, Episode, Tag, 
    HasTag, IsSequelOf, RelatedTo, RelationshipQueries
};

pub struct DatabaseService {
    db: Surreal<Any>,
}

impl DatabaseService {
    pub async fn new(url: &str) -> Result<Self> {
        // Initialize SurrealDB connection
        let db = connect(url).await?;
        
        // Use default namespace and database
        db.use_ns("kensho").use_db("anime").await?;
        
        Ok(DatabaseService { db })
    }
    
    pub async fn initialize_schema(&self) -> Result<()> {
        // Create tables - simplified for POC
        let _: Vec<()> = self.db.query("DEFINE TABLE anime SCHEMAFULL").await?;
        let _: Vec<()> = self.db.query("DEFINE TABLE episode SCHEMAFULL").await?;
        let _: Vec<()> = self.db.query("DEFINE TABLE tag SCHEMAFULL").await?;
        let _: Vec<()> = self.db.query("DEFINE TABLE session SCHEMAFULL").await?;
        
        // Define indexes
        let _: Vec<()> = self.db.query("DEFINE INDEX anime_title ON anime COLUMNS title").await?;
        let _: Vec<()> = self.db.query("DEFINE INDEX anime_status ON anime COLUMNS status").await?;
        let _: Vec<()> = self.db.query("DEFINE INDEX episode_anime ON episode COLUMNS anime_id").await?;
        let _: Vec<()> = self.db.query("DEFINE INDEX tag_category ON tag COLUMNS category").await?;
        
        // Define graph edges
        let _: Vec<()> = self.db.query("DEFINE TABLE has_tag SCHEMAFULL").await?;
        let _: Vec<()> = self.db.query("DEFINE TABLE is_sequel_of SCHEMAFULL").await?;
        let _: Vec<()> = self.db.query("DEFINE TABLE is_prequel_of SCHEMAFULL").await?;
        let _: Vec<()> = self.db.query("DEFINE TABLE related_to SCHEMAFULL").await?;
        
        Ok(())
    }
    
    // Anime operations
    pub async fn create_anime(&self, anime: &Anime) -> Result<Anime> {
        let created: Option<Anime> = self.db
            .create(("anime", anime.id.to_string()))
            .content(anime)
            .await?;
        
        created.context("Failed to create anime")
    }
    
    pub async fn get_anime(&self, id: Uuid) -> Result<Option<Anime>> {
        let result: Option<Anime> = self.db
            .select(("anime", id.to_string()))
            .await?;
        
        Ok(result)
    }
    
    pub async fn search_anime(&self, query: &str) -> Result<Vec<AnimeSummary>> {
        let sql = "SELECT * FROM anime WHERE title @@ $query OR $query IN synonyms LIMIT 20";
        
        let mut result = self.db
            .query(sql)
            .bind(("query", query))
            .await?;
        
        let anime: Vec<Anime> = result.take(0)?;
        
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    pub async fn list_anime(&self, limit: usize, offset: usize) -> Result<Vec<AnimeSummary>> {
        let sql = "SELECT * FROM anime ORDER BY created_at DESC LIMIT $limit START $offset";
        
        let mut result = self.db
            .query(sql)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        
        let anime: Vec<Anime> = result.take(0)?;
        
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    pub async fn get_anime_count(&self) -> Result<usize> {
        let sql = "SELECT count() as count FROM anime GROUP ALL";
        
        let mut result = self.db.query(sql).await?;
        let response: Option<serde_json::Value> = result.take(0)?;
        
        if let Some(val) = response {
            if let Some(count) = val.get("count").and_then(|v| v.as_u64()) {
                return Ok(count as usize);
            }
        }
        
        Ok(0)
    }
    
    pub async fn get_seasonal_anime(&self, year: u16, season: &str) -> Result<Vec<AnimeSummary>> {
        let sql = "SELECT * FROM anime WHERE anime_season.year = $year AND anime_season.season = $season";
        
        let mut result = self.db
            .query(sql)
            .bind(("year", year as i64))
            .bind(("season", season.to_lowercase()))
            .await?;
        
        let anime: Vec<Anime> = result.take(0)?;
        
        Ok(anime.into_iter().map(AnimeSummary::from).collect())
    }
    
    // Episode operations
    pub async fn create_episode(&self, episode: &Episode) -> Result<Episode> {
        let created: Option<Episode> = self.db
            .create(("episode", episode.id.to_string()))
            .content(episode)
            .await?;
        
        created.context("Failed to create episode")
    }
    
    pub async fn get_anime_episodes(&self, anime_id: Uuid) -> Result<Vec<Episode>> {
        let mut result = self.db
            .query(RelationshipQueries::anime_episodes_query())
            .bind(("anime_id", anime_id))
            .await?;
        
        let episodes: Vec<Episode> = result.take(0)?;
        Ok(episodes)
    }
    
    // Tag operations
    pub async fn create_tag(&self, tag: &Tag) -> Result<Tag> {
        let created: Option<Tag> = self.db
            .create(("tag", tag.id.to_string()))
            .content(tag)
            .await?;
        
        created.context("Failed to create tag")
    }
    
    pub async fn get_tags(&self) -> Result<Vec<Tag>> {
        let tags: Vec<Tag> = self.db
            .select("tag")
            .await?;
        
        Ok(tags)
    }
    
    // Relationship operations
    pub async fn link_anime_tag(&self, anime_id: Uuid, tag_id: Uuid, relevance: Option<f32>) -> Result<()> {
        let relation = HasTag::new(anime_id, tag_id)
            .with_relevance(relevance.unwrap_or(1.0));
        
        let _: Option<HasTag> = self.db
            .create("has_tag")
            .content(&relation)
            .await?;
        
        Ok(())
    }
    
    pub async fn link_sequel(&self, sequel_id: Uuid, prequel_id: Uuid) -> Result<()> {
        let relation = IsSequelOf::new(sequel_id, prequel_id);
        
        let _: Option<IsSequelOf> = self.db
            .create("is_sequel_of")
            .content(&relation)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_anime_tags(&self, anime_id: Uuid) -> Result<Vec<Tag>> {
        let mut result = self.db
            .query(RelationshipQueries::anime_tags_query())
            .bind(("anime_id", anime_id))
            .await?;
        
        let tags: Vec<Tag> = result.take(0)?;
        Ok(tags)
    }
    
    pub async fn get_similar_anime(&self, anime_id: Uuid, limit: usize) -> Result<Vec<AnimeSummary>> {
        let mut result = self.db
            .query(RelationshipQueries::similar_anime_query())
            .bind(("anime_id", anime_id))
            .bind(("limit", limit))
            .await?;
        
        let similar: Vec<Anime> = result.take(0)?;
        
        Ok(similar.into_iter().map(AnimeSummary::from).collect())
    }
    
    // Batch operations for ingestion
    pub async fn batch_insert_anime(&self, anime_list: Vec<Anime>) -> Result<usize> {
        let mut count = 0;
        
        for anime in anime_list {
            if self.create_anime(&anime).await.is_ok() {
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    pub async fn batch_insert_tags(&self, tags: Vec<Tag>) -> Result<usize> {
        let mut count = 0;
        
        for tag in tags {
            if self.create_tag(&tag).await.is_ok() {
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    pub async fn batch_insert_episodes(&self, episodes: Vec<Episode>) -> Result<usize> {
        let mut count = 0;
        
        for episode in episodes {
            if self.create_episode(&episode).await.is_ok() {
                count += 1;
            }
        }
        
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires SurrealDB running
    async fn test_database_connection() {
        let db = DatabaseService::new("ws://localhost:8000").await;
        assert!(db.is_ok());
    }
}