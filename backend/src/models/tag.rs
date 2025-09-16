// T025: Tag model and enums
// Reference: data-model.md lines 97-113 for Tag struct and TagCategory enum

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Tag {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    
    #[validate(length(min = 1, max = 50, message = "Tag name must be between 1 and 50 characters"))]
    pub name: String,
    
    pub category: TagCategory,
    
    pub description: Option<String>,
    
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TagCategory {
    Genre,      // Action, Comedy, Drama
    Theme,      // School, Military, Supernatural
    Demographic, // Shounen, Seinen, Josei
    Content,    // Violence, Romance
}

impl Tag {
    pub fn new(name: String, category: TagCategory) -> Self {
        Tag {
            id: Uuid::new_v4(),
            name,
            category,
            description: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

// Common tags for seeding
impl Tag {
    pub fn action() -> Self {
        Tag::new("Action".to_string(), TagCategory::Genre)
            .with_description("Fast-paced, exciting sequences".to_string())
    }
    
    pub fn comedy() -> Self {
        Tag::new("Comedy".to_string(), TagCategory::Genre)
            .with_description("Humorous and entertaining content".to_string())
    }
    
    pub fn drama() -> Self {
        Tag::new("Drama".to_string(), TagCategory::Genre)
            .with_description("Emotional and character-driven stories".to_string())
    }
    
    pub fn school() -> Self {
        Tag::new("School".to_string(), TagCategory::Theme)
            .with_description("Set in or around school environments".to_string())
    }
    
    pub fn supernatural() -> Self {
        Tag::new("Supernatural".to_string(), TagCategory::Theme)
            .with_description("Features supernatural elements".to_string())
    }
    
    pub fn shounen() -> Self {
        Tag::new("Shounen".to_string(), TagCategory::Demographic)
            .with_description("Targeted at young male audience".to_string())
    }
    
    pub fn seinen() -> Self {
        Tag::new("Seinen".to_string(), TagCategory::Demographic)
            .with_description("Targeted at adult male audience".to_string())
    }
}

// Response DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
    pub category: TagCategory,
}

impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        TagResponse {
            id: tag.id,
            name: tag.name,
            category: tag.category,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_validation() {
        // Valid tag
        let tag = Tag::new("Action".to_string(), TagCategory::Genre);
        assert!(tag.validate().is_ok());
        
        // Empty name
        let mut invalid_tag = tag.clone();
        invalid_tag.name = "".to_string();
        assert!(invalid_tag.validate().is_err());
        
        // Name too long
        let mut long_tag = tag.clone();
        long_tag.name = "a".repeat(51);
        assert!(long_tag.validate().is_err());
    }
    
    #[test]
    fn test_preset_tags() {
        let action = Tag::action();
        assert_eq!(action.name, "Action");
        assert_eq!(action.category, TagCategory::Genre);
        assert!(action.description.is_some());
        
        let shounen = Tag::shounen();
        assert_eq!(shounen.category, TagCategory::Demographic);
    }
}