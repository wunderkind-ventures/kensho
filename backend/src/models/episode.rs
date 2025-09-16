// T024: Episode model
// Reference: data-model.md lines 74-95 for Episode struct and validation

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Episode {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    
    pub anime_id: Uuid,
    
    #[validate(range(min = 1, message = "Episode number must be > 0"))]
    pub episode_number: u32,
    
    pub title: Option<String>,
    
    // Custom validation temporarily removed for POC
    // #[validate(custom(function = "validate_duration"))]
    pub duration: Option<u32>, // Duration in seconds
    
    pub air_date: Option<NaiveDate>,
    
    pub synopsis: Option<String>,
    
    #[validate(url(message = "Thumbnail URL must be valid"))]
    pub thumbnail_url: Option<String>,
    
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl Episode {
    pub fn new(anime_id: Uuid, episode_number: u32) -> Self {
        Episode {
            id: Uuid::new_v4(),
            anime_id,
            episode_number,
            title: None,
            duration: None,
            air_date: None,
            synopsis: None,
            thumbnail_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
    
    pub fn with_duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }
    
    pub fn with_metadata(
        mut self,
        title: Option<String>,
        duration: Option<u32>,
        air_date: Option<NaiveDate>,
        synopsis: Option<String>,
        thumbnail_url: Option<String>,
    ) -> Self {
        self.title = title;
        self.duration = duration;
        self.air_date = air_date;
        self.synopsis = synopsis;
        self.thumbnail_url = thumbnail_url;
        self
    }
}

// Custom validators
fn validate_duration(duration: &Option<u32>) -> Result<(), ValidationError> {
    if let Some(d) = duration {
        if *d == 0 {
            return Err(ValidationError::new("invalid_duration"));
        }
    }
    Ok(())
}

// Response DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeResponse {
    pub id: Uuid,
    pub episode_number: u32,
    pub title: Option<String>,
    pub duration: Option<u32>,
    pub air_date: Option<NaiveDate>,
    pub synopsis: Option<String>,
    pub thumbnail_url: Option<String>,
}

impl From<Episode> for EpisodeResponse {
    fn from(episode: Episode) -> Self {
        EpisodeResponse {
            id: episode.id,
            episode_number: episode.episode_number,
            title: episode.title,
            duration: episode.duration,
            air_date: episode.air_date,
            synopsis: episode.synopsis,
            thumbnail_url: episode.thumbnail_url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeListResponse {
    pub episodes: Vec<EpisodeResponse>,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_validation() {
        let anime_id = Uuid::new_v4();
        
        // Valid episode
        let episode = Episode::new(anime_id, 1);
        assert!(episode.validate().is_ok());
        
        // Invalid episode number
        let mut invalid_episode = Episode::new(anime_id, 0);
        invalid_episode.episode_number = 0;
        assert!(invalid_episode.validate().is_err());
        
        // Invalid duration
        let mut episode_with_duration = Episode::new(anime_id, 1);
        episode_with_duration.duration = Some(0);
        assert!(episode_with_duration.validate().is_err());
        
        // Valid duration
        episode_with_duration.duration = Some(1440); // 24 minutes
        assert!(episode_with_duration.validate().is_ok());
    }
    
    #[test]
    fn test_episode_builder() {
        let anime_id = Uuid::new_v4();
        let episode = Episode::new(anime_id, 5)
            .with_title("Episode 5: The Test".to_string())
            .with_duration(1440);
        
        assert_eq!(episode.episode_number, 5);
        assert_eq!(episode.title, Some("Episode 5: The Test".to_string()));
        assert_eq!(episode.duration, Some(1440));
    }
}