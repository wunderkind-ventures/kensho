// Anime Offline Database Models
// Generated from anime-offline-database.json with enhancements for Kensho project

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::{Anime, AnimeStatus, AnimeType, AnimeSeason, Season};

/// Root structure of the anime-offline-database.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeOfflineDatabase {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub license: License,
    pub repository: String,
    pub score_range: ScoreRange,
    pub last_update: String,
    pub data: Vec<AnimeOfflineEntry>,
}

impl AnimeOfflineDatabase {
    /// Load the database from the JSON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let db: AnimeOfflineDatabase = serde_json::from_str(&content)?;
        Ok(db)
    }

    /// Convert all entries to Kensho Anime models
    pub fn to_anime_models(&self) -> Vec<Anime> {
        self.data.iter().map(|entry| entry.to_anime_model()).collect()
    }

    /// Filter entries by type
    pub fn filter_by_type(&self, anime_type: &OfflineAnimeType) -> Vec<&AnimeOfflineEntry> {
        self.data.iter().filter(|entry| &entry.anime_type == anime_type).collect()
    }

    /// Filter entries by status
    pub fn filter_by_status(&self, status: &OfflineAnimeStatus) -> Vec<&AnimeOfflineEntry> {
        self.data.iter().filter(|entry| &entry.status == status).collect()
    }

    /// Get entries with scores above threshold
    pub fn filter_by_min_score(&self, min_score: f64) -> Vec<&AnimeOfflineEntry> {
        self.data.iter()
            .filter(|entry| {
                entry.score.as_ref()
                    .map(|s| s.arithmetic_mean >= min_score)
                    .unwrap_or(false)
            })
            .collect()
    }
}

/// Individual anime entry from the offline database
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AnimeOfflineEntry {
    #[validate(length(min = 1, message = "Must have at least one source"))]
    pub sources: Vec<String>,
    
    #[validate(length(min = 1, max = 500, message = "Title must be between 1 and 500 characters"))]
    pub title: String,
    
    #[serde(rename = "type")]
    pub anime_type: OfflineAnimeType,
    
    #[validate(range(min = 0, message = "Episodes must be >= 0"))]
    pub episodes: i64,
    
    pub status: OfflineAnimeStatus,
    
    #[validate(nested)]
    pub anime_season: OfflineAnimeSeason,
    
    #[validate(url(message = "Picture URL must be valid"))]
    pub picture: String,
    
    #[validate(url(message = "Thumbnail URL must be valid"))]
    pub thumbnail: String,
    
    pub duration: Option<Duration>,
    
    pub score: Option<AnimeScore>,
    
    pub synonyms: Vec<String>,
    
    pub studios: Vec<String>,
    
    pub producers: Vec<String>,
    
    #[validate(custom(function = "validate_urls"))]
    pub related_anime: Vec<String>,
    
    pub tags: Vec<String>,
}

impl AnimeOfflineEntry {
    /// Convert to Kensho Anime model
    pub fn to_anime_model(&self) -> Anime {
        Anime {
            id: Uuid::new_v4(),
            title: self.title.clone(),
            synonyms: self.synonyms.clone(),
            sources: self.sources.clone(),
            episodes: self.episodes.max(0) as u32,
            status: self.status.to_anime_status(),
            anime_type: self.anime_type.to_anime_type(),
            anime_season: self.anime_season.to_anime_season(),
            synopsis: format!("Imported from anime-offline-database. Studios: {}. Tags: {}.", 
                            self.studios.join(", "), 
                            self.tags.join(", ")),
            poster_url: self.picture.clone(),
            imdb: self.score.as_ref().map(|s| crate::models::ImdbData {
                id: format!("offline-{}", self.title.replace(" ", "-").to_lowercase()),
                rating: (s.arithmetic_mean * 10.0 / 10.0) as f32, // Normalize to 0-10 scale
                votes: 100, // Default placeholder
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Extract MyAnimeList ID from sources if available
    pub fn get_mal_id(&self) -> Option<String> {
        self.sources.iter()
            .find(|s| s.contains("myanimelist.net"))
            .and_then(|s| s.split('/').last())
            .map(|s| s.to_string())
    }

    /// Extract AniList ID from sources if available
    pub fn get_anilist_id(&self) -> Option<String> {
        self.sources.iter()
            .find(|s| s.contains("anilist.co"))
            .and_then(|s| s.split('/').last())
            .map(|s| s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfflineAnimeType {
    #[serde(rename = "TV")]
    Tv,
    #[serde(rename = "MOVIE")]
    Movie,
    #[serde(rename = "OVA")]
    Ova,
    #[serde(rename = "ONA")]
    Ona,
    #[serde(rename = "SPECIAL")]
    Special,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl OfflineAnimeType {
    pub fn to_anime_type(&self) -> AnimeType {
        match self {
            OfflineAnimeType::Tv => AnimeType::TV,
            OfflineAnimeType::Movie => AnimeType::Movie,
            OfflineAnimeType::Ova => AnimeType::OVA,
            OfflineAnimeType::Ona => AnimeType::ONA,
            OfflineAnimeType::Special => AnimeType::Special,
            OfflineAnimeType::Unknown => AnimeType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfflineAnimeStatus {
    #[serde(rename = "FINISHED")]
    Finished,
    #[serde(rename = "ONGOING")]
    Ongoing,
    #[serde(rename = "UPCOMING")]
    Upcoming,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl OfflineAnimeStatus {
    pub fn to_anime_status(&self) -> AnimeStatus {
        match self {
            OfflineAnimeStatus::Finished => AnimeStatus::Finished,
            OfflineAnimeStatus::Ongoing => AnimeStatus::Ongoing,
            OfflineAnimeStatus::Upcoming => AnimeStatus::Upcoming,
            OfflineAnimeStatus::Unknown => AnimeStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OfflineAnimeSeason {
    pub season: OfflineSeason,
    pub year: Option<i64>,
}

impl OfflineAnimeSeason {
    pub fn to_anime_season(&self) -> AnimeSeason {
        AnimeSeason {
            season: self.season.to_season(),
            year: self.year.unwrap_or(2000) as u16, // Default year for missing data
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfflineSeason {
    #[serde(rename = "SPRING")]
    Spring,
    #[serde(rename = "SUMMER")]
    Summer,
    #[serde(rename = "FALL")]
    Fall,
    #[serde(rename = "WINTER")]
    Winter,
    #[serde(rename = "UNDEFINED")]
    Undefined,
}

impl OfflineSeason {
    pub fn to_season(&self) -> Season {
        match self {
            OfflineSeason::Spring => Season::Spring,
            OfflineSeason::Summer => Season::Summer,
            OfflineSeason::Fall => Season::Fall,
            OfflineSeason::Winter => Season::Winter,
            OfflineSeason::Undefined => Season::Spring, // Default fallback
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    pub value: i64,
    pub unit: DurationUnit,
}

impl Duration {
    /// Get duration in minutes
    pub fn in_minutes(&self) -> i64 {
        match self.unit {
            DurationUnit::Seconds => self.value / 60,
        }
    }

    /// Get duration in seconds
    pub fn in_seconds(&self) -> i64 {
        match self.unit {
            DurationUnit::Seconds => self.value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurationUnit {
    #[serde(rename = "SECONDS")]
    Seconds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeScore {
    pub arithmetic_geometric_mean: f64,
    pub arithmetic_mean: f64,
    pub median: f64,
}

impl AnimeScore {
    /// Get the best representative score (using arithmetic mean)
    pub fn best_score(&self) -> f64 {
        self.arithmetic_mean
    }

    /// Check if this is a highly rated anime (score >= 8.0)
    pub fn is_highly_rated(&self) -> bool {
        self.arithmetic_mean >= 8.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreRange {
    pub min_inclusive: f64,
    pub max_inclusive: f64,
}

// Custom validation functions
fn validate_urls(urls: &Vec<String>) -> Result<(), ValidationError> {
    for url in urls {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ValidationError::new("invalid_url"));
        }
    }
    Ok(())
}

// Utility functions for data import
impl AnimeOfflineDatabase {
    /// Import a batch of entries into Kensho format with progress callback
    pub fn import_batch<F>(&self, batch_size: usize, mut progress_callback: F) -> Vec<Anime>
    where
        F: FnMut(usize, usize),
    {
        let total = self.data.len();
        let mut imported = Vec::new();
        
        for (index, chunk) in self.data.chunks(batch_size).enumerate() {
            let batch_start = index * batch_size;
            for entry in chunk {
                imported.push(entry.to_anime_model());
            }
            progress_callback(batch_start + chunk.len(), total);
        }
        
        imported
    }

    /// Get statistics about the database
    pub fn get_stats(&self) -> DatabaseStats {
        let total_count = self.data.len();
        let tv_count = self.filter_by_type(&OfflineAnimeType::Tv).len();
        let movie_count = self.filter_by_type(&OfflineAnimeType::Movie).len();
        let finished_count = self.filter_by_status(&OfflineAnimeStatus::Finished).len();
        let ongoing_count = self.filter_by_status(&OfflineAnimeStatus::Ongoing).len();
        
        let scored_entries: Vec<_> = self.data.iter()
            .filter_map(|e| e.score.as_ref())
            .collect();
        
        let average_score = if scored_entries.is_empty() {
            0.0
        } else {
            scored_entries.iter().map(|s| s.arithmetic_mean).sum::<f64>() / scored_entries.len() as f64
        };

        DatabaseStats {
            total_entries: total_count,
            tv_series: tv_count,
            movies: movie_count,
            finished: finished_count,
            ongoing: ongoing_count,
            with_scores: scored_entries.len(),
            average_score,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_entries: usize,
    pub tv_series: usize,
    pub movies: usize,
    pub finished: usize,
    pub ongoing: usize,
    pub with_scores: usize,
    pub average_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anime_type_conversion() {
        assert_eq!(OfflineAnimeType::Tv.to_anime_type(), AnimeType::TV);
        assert_eq!(OfflineAnimeType::Movie.to_anime_type(), AnimeType::Movie);
    }

    #[test]
    fn test_season_conversion() {
        assert_eq!(OfflineSeason::Spring.to_season(), Season::Spring);
        assert_eq!(OfflineSeason::Undefined.to_season(), Season::Spring);
    }

    #[test]
    fn test_duration_conversion() {
        let duration = Duration {
            value: 1440,
            unit: DurationUnit::Seconds,
        };
        assert_eq!(duration.in_minutes(), 24);
        assert_eq!(duration.in_seconds(), 1440);
    }

    #[test]
    fn test_score_evaluation() {
        let score = AnimeScore {
            arithmetic_geometric_mean: 8.5,
            arithmetic_mean: 8.7,
            median: 8.6,
        };
        assert!(score.is_highly_rated());
        assert_eq!(score.best_score(), 8.7);
    }

    #[test] 
    fn test_url_extraction() {
        let entry = AnimeOfflineEntry {
            sources: vec![
                "https://myanimelist.net/anime/12345".to_string(),
                "https://anilist.co/anime/67890".to_string(),
            ],
            title: "Test Anime".to_string(),
            anime_type: OfflineAnimeType::Tv,
            episodes: 12,
            status: OfflineAnimeStatus::Finished,
            anime_season: OfflineAnimeSeason {
                season: OfflineSeason::Spring,
                year: Some(2024),
            },
            picture: "https://example.com/picture.jpg".to_string(),
            thumbnail: "https://example.com/thumbnail.jpg".to_string(),
            duration: None,
            score: None,
            synonyms: vec![],
            studios: vec![],
            producers: vec![],
            related_anime: vec![],
            tags: vec![],
        };

        assert_eq!(entry.get_mal_id(), Some("12345".to_string()));
        assert_eq!(entry.get_anilist_id(), Some("67890".to_string()));
    }
}
