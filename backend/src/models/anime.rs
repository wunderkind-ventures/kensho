// T023: Anime model with validation
// Reference: data-model.md lines 15-65 for Anime struct definition and validation rules

use chrono::{DateTime, Utc, Datelike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Anime {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    
    #[validate(length(min = 1, max = 500, message = "Title must be between 1 and 500 characters"))]
    pub title: String,
    
    #[serde(default)]
    pub synonyms: Vec<String>,
    
    #[serde(default)]
    pub sources: Vec<String>,
    
    #[validate(range(min = 0, message = "Episodes must be >= 0"))]
    pub episodes: u32,
    
    pub status: AnimeStatus,
    
    #[serde(rename = "type")]
    pub anime_type: AnimeType,
    
    #[validate(nested)]
    pub anime_season: AnimeSeason,
    
    pub synopsis: String,
    
    #[validate(url(message = "Poster URL must be valid"))]
    pub poster_url: String,
    
    pub imdb: Option<ImdbData>,
    
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnimeStatus {
    Finished,
    Ongoing,
    Upcoming,
    Unknown,
}

impl Default for AnimeStatus {
    fn default() -> Self {
        AnimeStatus::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimeType {
    TV,
    Movie,
    OVA,
    ONA,
    Special,
    Unknown,
}

impl Default for AnimeType {
    fn default() -> Self {
        AnimeType::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AnimeSeason {
    pub season: Season,
    
    // Custom validation temporarily removed for POC
    // #[validate(custom(function = "validate_year"))]
    pub year: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Default for Season {
    fn default() -> Self {
        Season::Spring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ImdbData {
    pub id: String,
    
    #[validate(range(min = 0.0, max = 10.0, message = "Rating must be between 0.0 and 10.0"))]
    pub rating: f32,
    
    pub votes: u32,
}

// Custom validators
fn validate_year(year: &u16) -> Result<(), ValidationError> {
    let current_year = Utc::now().year() as u16;
    if *year < 1900 || *year > current_year + 5 {
        return Err(ValidationError::new("invalid_year"));
    }
    Ok(())
}

// Response DTOs for API
#[derive(Debug, Serialize, Deserialize)]
pub struct AnimeSummary {
    pub id: Uuid,
    pub title: String,
    pub poster_url: String,
    pub episodes: u32,
    pub status: AnimeStatus,
    pub anime_type: AnimeType,
    pub imdb_rating: Option<f32>,
}

impl From<Anime> for AnimeSummary {
    fn from(anime: Anime) -> Self {
        AnimeSummary {
            id: anime.id,
            title: anime.title,
            poster_url: anime.poster_url,
            episodes: anime.episodes,
            status: anime.status,
            anime_type: anime.anime_type,
            imdb_rating: anime.imdb.as_ref().map(|imdb| imdb.rating),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimeDetail {
    #[serde(flatten)]
    pub anime: Anime,
    pub tags: Vec<crate::models::tag::Tag>,
    pub related_anime: RelatedAnime,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RelatedAnime {
    pub sequels: Vec<AnimeSummary>,
    pub prequels: Vec<AnimeSummary>,
    pub related: Vec<AnimeSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anime_validation() {
        let mut anime = Anime {
            id: Uuid::new_v4(),
            title: "Test Anime".to_string(),
            synonyms: vec![],
            sources: vec![],
            episodes: 12,
            status: AnimeStatus::Finished,
            anime_type: AnimeType::TV,
            anime_season: AnimeSeason {
                season: Season::Spring,
                year: 2024,
            },
            synopsis: "Test synopsis".to_string(),
            poster_url: "https://example.com/poster.jpg".to_string(),
            imdb: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(anime.validate().is_ok());

        // Test invalid title
        anime.title = "".to_string();
        assert!(anime.validate().is_err());
        
        // Test invalid year
        anime.title = "Valid Title".to_string();
        anime.anime_season.year = 1899;
        assert!(anime.validate().is_err());
        
        // Test invalid poster URL
        anime.anime_season.year = 2024;
        anime.poster_url = "not-a-url".to_string();
        assert!(anime.validate().is_err());
    }
}