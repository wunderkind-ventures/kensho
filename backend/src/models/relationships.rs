// T027: Graph relationships
// Reference: data-model.md lines 145-189 for relationship definitions

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Anime -> Tag relationship (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasTag {
    pub anime_id: Uuid,
    pub tag_id: Uuid,
    pub relevance_score: Option<f32>, // 0.0 to 1.0
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

// Anime -> Anime sequel relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsSequelOf {
    pub sequel_id: Uuid,
    pub prequel_id: Uuid,
    pub order: Option<u32>, // For multiple sequels
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

// Anime -> Anime prequel relationship (inverse of sequel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsPrequelOf {
    pub prequel_id: Uuid,
    pub sequel_id: Uuid,
    pub order: Option<u32>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

// Anime -> Anime related relationship (spin-offs, alternatives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedTo {
    pub anime_id: Uuid,
    pub related_id: Uuid,
    pub relation_type: RelationType,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    SpinOff,
    Alternative,
    SideStory,
    Summary,
    Other,
}

// Episode -> Anime relationship (one-to-many)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BelongsTo {
    pub episode_id: Uuid,
    pub anime_id: Uuid,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

// Graph traversal helpers
pub struct RelationshipQueries;

impl RelationshipQueries {
    // SurrealDB query to find all tags for an anime
    pub fn anime_tags_query() -> &'static str {
        r#"
        SELECT tag.* FROM has_tag 
        WHERE anime_id = $anime_id
        FETCH tag
        ORDER BY relevance_score DESC NULLS LAST
        "#
    }
    
    // Find all anime with a specific tag
    pub fn tag_anime_query() -> &'static str {
        r#"
        SELECT anime.* FROM has_tag
        WHERE tag_id = $tag_id
        FETCH anime
        "#
    }
    
    // Find sequels of an anime
    pub fn anime_sequels_query() -> &'static str {
        r#"
        SELECT sequel.* FROM is_sequel_of
        WHERE prequel_id = $anime_id
        FETCH sequel
        ORDER BY order ASC NULLS LAST
        "#
    }
    
    // Find prequels of an anime
    pub fn anime_prequels_query() -> &'static str {
        r#"
        SELECT prequel.* FROM is_prequel_of
        WHERE sequel_id = $anime_id
        FETCH prequel
        ORDER BY order DESC NULLS LAST
        "#
    }
    
    // Find related anime
    pub fn anime_related_query() -> &'static str {
        r#"
        SELECT related.* FROM related_to
        WHERE anime_id = $anime_id
        FETCH related
        "#
    }
    
    // Find similar anime based on shared tags
    pub fn similar_anime_query() -> &'static str {
        r#"
        SELECT 
            anime2.id as anime_id,
            count(*) as shared_tags,
            array::group(tag.name) as tags
        FROM anime:$anime_id->has_tag->tag<-has_tag<-anime as anime2
        WHERE anime2.id != $anime_id
        GROUP BY anime2.id
        ORDER BY shared_tags DESC
        LIMIT $limit
        "#
    }
    
    // Episodes for an anime
    pub fn anime_episodes_query() -> &'static str {
        r#"
        SELECT * FROM episode
        WHERE anime_id = $anime_id
        ORDER BY episode_number ASC
        "#
    }
}

// Relationship builders for seeding
impl HasTag {
    pub fn new(anime_id: Uuid, tag_id: Uuid) -> Self {
        HasTag {
            anime_id,
            tag_id,
            relevance_score: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn with_relevance(mut self, score: f32) -> Self {
        self.relevance_score = Some(score.clamp(0.0, 1.0));
        self
    }
}

impl IsSequelOf {
    pub fn new(sequel_id: Uuid, prequel_id: Uuid) -> Self {
        IsSequelOf {
            sequel_id,
            prequel_id,
            order: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn with_order(mut self, order: u32) -> Self {
        self.order = Some(order);
        self
    }
}

impl RelatedTo {
    pub fn new(anime_id: Uuid, related_id: Uuid, relation_type: RelationType) -> Self {
        RelatedTo {
            anime_id,
            related_id,
            relation_type,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_has_tag_relationship() {
        let anime_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        
        let rel = HasTag::new(anime_id, tag_id)
            .with_relevance(0.95);
        
        assert_eq!(rel.anime_id, anime_id);
        assert_eq!(rel.tag_id, tag_id);
        assert_eq!(rel.relevance_score, Some(0.95));
    }
    
    #[test]
    fn test_relevance_clamping() {
        let rel = HasTag::new(Uuid::new_v4(), Uuid::new_v4())
            .with_relevance(1.5);
        
        assert_eq!(rel.relevance_score, Some(1.0));
        
        let rel2 = HasTag::new(Uuid::new_v4(), Uuid::new_v4())
            .with_relevance(-0.5);
        
        assert_eq!(rel2.relevance_score, Some(0.0));
    }
    
    #[test]
    fn test_sequel_relationship() {
        let sequel_id = Uuid::new_v4();
        let prequel_id = Uuid::new_v4();
        
        let rel = IsSequelOf::new(sequel_id, prequel_id)
            .with_order(2);
        
        assert_eq!(rel.sequel_id, sequel_id);
        assert_eq!(rel.prequel_id, prequel_id);
        assert_eq!(rel.order, Some(2));
    }
}