// T028: Metadata ingestion library
// Reference: plan.md "Metadata Ingestion" section
// Aggregates from anime-offline-database and IMDb

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::models::{Anime, AnimeStatus, AnimeType, AnimeSeason, Season, ImdbData, Tag, TagCategory, Episode};

// anime-offline-database format
#[derive(Debug, Clone, Deserialize)]
pub struct OfflineAnimeEntry {
    pub sources: Vec<String>,
    pub title: String,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(rename = "type")]
    pub anime_type: String,
    pub episodes: u32,
    pub status: String,
    #[serde(rename = "animeSeason")]
    pub anime_season: OfflineAnimeSeason,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub picture: String,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfflineAnimeSeason {
    pub season: String,
    pub year: Option<u16>,
}

// IMDb data structure
#[derive(Debug, Clone, Deserialize)]
pub struct ImdbEntry {
    pub id: String,
    pub rating: f32,
    pub votes: u32,
}

pub struct MetadataService {
    offline_db_path: String,
    imdb_data: HashMap<String, ImdbEntry>,
}

impl MetadataService {
    pub fn new(offline_db_path: String) -> Self {
        MetadataService {
            offline_db_path,
            imdb_data: HashMap::new(),
        }
    }
    
    pub async fn load_offline_database(&mut self) -> Result<Vec<OfflineAnimeEntry>> {
        let content = tokio::fs::read_to_string(&self.offline_db_path)
            .await
            .context("Failed to read offline database")?;
        
        let data: serde_json::Value = serde_json::from_str(&content)?;
        let entries = data["data"]
            .as_array()
            .context("Invalid offline database format")?
            .iter()
            .filter_map(|v| serde_json::from_value::<OfflineAnimeEntry>(v.clone()).ok())
            .collect();
        
        Ok(entries)
    }
    
    pub async fn load_imdb_data(&mut self, imdb_path: &str) -> Result<()> {
        let content = tokio::fs::read_to_string(imdb_path)
            .await
            .context("Failed to read IMDb data")?;
        
        // Parse TSV or JSON format
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let entry = ImdbEntry {
                    id: parts[0].to_string(),
                    rating: parts[1].parse().unwrap_or(0.0),
                    votes: parts[2].parse().unwrap_or(0),
                };
                self.imdb_data.insert(entry.id.clone(), entry);
            }
        }
        
        Ok(())
    }
    
    pub fn convert_to_anime(&self, entry: OfflineAnimeEntry) -> Result<Anime> {
        let anime_type = match entry.anime_type.as_str() {
            "TV" => AnimeType::TV,
            "MOVIE" => AnimeType::Movie,
            "OVA" => AnimeType::OVA,
            "ONA" => AnimeType::ONA,
            "SPECIAL" => AnimeType::Special,
            _ => AnimeType::Unknown,
        };
        
        let status = match entry.status.as_str() {
            "FINISHED" => AnimeStatus::Finished,
            "ONGOING" => AnimeStatus::Ongoing,
            "UPCOMING" => AnimeStatus::Upcoming,
            _ => AnimeStatus::Unknown,
        };
        
        let season = match entry.anime_season.season.as_str() {
            "SPRING" => Season::Spring,
            "SUMMER" => Season::Summer,
            "FALL" => Season::Fall,
            "WINTER" => Season::Winter,
            _ => Season::Spring,
        };
        
        let anime_season = AnimeSeason {
            season,
            year: entry.anime_season.year.unwrap_or(2024),
        };
        
        // Try to find IMDb data
        let imdb = self.find_imdb_match(&entry.title)
            .map(|data| ImdbData {
                id: data.id.clone(),
                rating: data.rating,
                votes: data.votes,
            });
        
        Ok(Anime {
            id: Uuid::new_v4(),
            title: entry.title,
            synonyms: entry.synonyms,
            sources: entry.sources,
            episodes: entry.episodes,
            status,
            anime_type,
            anime_season,
            synopsis: String::new(), // To be enriched from other sources
            poster_url: entry.picture,
            imdb,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    
    fn find_imdb_match(&self, title: &str) -> Option<&ImdbEntry> {
        // Simple title matching - can be enhanced with fuzzy matching
        let _normalized = title.to_lowercase().replace(" ", "");
        
        self.imdb_data.values().find(|_entry| {
            // Match by ID pattern or title similarity
            // This is simplified - real implementation would use better matching
            false // Placeholder
        })
    }
    
    pub fn extract_tags(&self, entry: &OfflineAnimeEntry) -> Vec<Tag> {
        entry.tags.iter().map(|tag_name| {
            let category = self.categorize_tag(tag_name);
            Tag::new(tag_name.clone(), category)
        }).collect()
    }
    
    fn categorize_tag(&self, tag_name: &str) -> TagCategory {
        match tag_name.to_lowercase().as_str() {
            "action" | "comedy" | "drama" | "romance" | "horror" | "thriller" | "mystery" => TagCategory::Genre,
            "school" | "military" | "supernatural" | "historical" | "space" => TagCategory::Theme,
            "shounen" | "seinen" | "josei" | "shoujo" => TagCategory::Demographic,
            _ => TagCategory::Content,
        }
    }
    
    pub async fn generate_episodes(&self, anime_id: Uuid, episode_count: u32) -> Vec<Episode> {
        (1..=episode_count).map(|num| {
            Episode::new(anime_id, num)
        }).collect()
    }
    
    // Batch ingestion
    pub async fn ingest_all(&mut self) -> Result<IngestResult> {
        let entries = self.load_offline_database().await?;
        
        let mut anime_list = Vec::new();
        let mut tags_map = HashMap::new();
        let mut episodes_list = Vec::new();
        
        for entry in entries {
            let anime = self.convert_to_anime(entry.clone())?;
            let tags = self.extract_tags(&entry);
            let episodes = self.generate_episodes(anime.id, anime.episodes).await;
            
            // Store tags by name to avoid duplicates
            for tag in tags {
                tags_map.entry(tag.name.clone()).or_insert(tag);
            }
            
            episodes_list.extend(episodes);
            anime_list.push(anime);
        }
        
        Ok(IngestResult {
            anime_count: anime_list.len(),
            tag_count: tags_map.len(),
            episode_count: episodes_list.len(),
            anime: anime_list,
            tags: tags_map.into_values().collect(),
            episodes: episodes_list,
        })
    }
}

#[derive(Debug)]
pub struct IngestResult {
    pub anime_count: usize,
    pub tag_count: usize,
    pub episode_count: usize,
    pub anime: Vec<Anime>,
    pub tags: Vec<Tag>,
    pub episodes: Vec<Episode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tag_categorization() {
        let service = MetadataService::new("test.json".to_string());
        
        assert_eq!(service.categorize_tag("Action"), TagCategory::Genre);
        assert_eq!(service.categorize_tag("School"), TagCategory::Theme);
        assert_eq!(service.categorize_tag("Shounen"), TagCategory::Demographic);
        assert_eq!(service.categorize_tag("Violence"), TagCategory::Content);
    }
    
    #[tokio::test]
    async fn test_episode_generation() {
        let service = MetadataService::new("test.json".to_string());
        let anime_id = Uuid::new_v4();
        let episodes = service.generate_episodes(anime_id, 12).await;
        
        assert_eq!(episodes.len(), 12);
        assert_eq!(episodes[0].episode_number, 1);
        assert_eq!(episodes[11].episode_number, 12);
        
        for episode in episodes {
            assert_eq!(episode.anime_id, anime_id);
        }
    }
}