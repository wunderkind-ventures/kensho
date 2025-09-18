// T031: Search engine with full-text search
// Reference: spec.md FR-002 for search requirements

use anyhow::{Result, Context};
use crate::models::{Anime, AnimeSummary, Tag};
use crate::services::DatabaseService;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SearchService {
    db: Arc<DatabaseService>,
}

impl SearchService {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        SearchService { db }
    }
    
    pub async fn search_anime(&self, query: &str) -> Result<Vec<AnimeSummary>> {
        // Use database search functionality
        self.db.search_anime(query).await
    }
    
    pub async fn search_by_tag(&self, tag_name: &str) -> Result<Vec<AnimeSummary>> {
        // Find all anime with a specific tag
        let tags = self.db.get_tags().await?;
        
        let matching_tag = tags
            .iter()
            .find(|t| t.name.to_lowercase() == tag_name.to_lowercase())
            .context("Tag not found")?;
        
        // Get all anime with this tag
        // For POC, simplified - would use graph query in production
        let all_anime = self.db.list_anime(100, 0).await?;
        
        // Filter anime that have this tag
        let mut results = Vec::new();
        for anime_summary in all_anime {
            if let Ok(Some(anime)) = self.db.get_anime(anime_summary.id).await {
                let anime_tags = self.db.get_anime_tags(anime.id).await?;
                if anime_tags.iter().any(|t| t.id == matching_tag.id) {
                    results.push(AnimeSummary::from(anime));
                }
            }
        }
        
        Ok(results)
    }
    
    pub async fn search_by_season(&self, year: u16, season: &str) -> Result<Vec<AnimeSummary>> {
        // Use optimized database method
        self.db.get_seasonal_anime(year, season).await
    }
    
    pub async fn get_trending(&self, limit: usize) -> Result<Vec<AnimeSummary>> {
        // For POC, just return recent anime
        // In production, would track views/popularity
        self.db.list_anime(limit, 0).await
    }
    
    pub async fn get_recommendations(&self, anime_id: uuid::Uuid, limit: usize) -> Result<Vec<AnimeSummary>> {
        // Get similar anime based on tags
        self.db.get_similar_anime(anime_id, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_search_service_creation() {
        let db = Arc::new(DatabaseService::new("memory://").await.unwrap());
        let search = SearchService::new(db);
        
        // Should be able to search even with empty database
        let results = search.search_anime("test").await.unwrap();
        assert_eq!(results.len(), 0);
    }
}