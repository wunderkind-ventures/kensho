use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anime {
    pub id: Uuid,
    pub title: String,
    pub synonyms: Vec<String>,
    pub sources: Vec<String>,
    pub episodes: i32,
    pub status: String,
    pub anime_type: String,
    pub season: Option<AnimeSeason>,
    pub imdb_data: Option<ImdbData>,
    pub poster_url: Option<String>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeSeason {
    pub season: String,
    pub year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImdbData {
    pub rating: f32,
    pub votes: i32,
    pub plot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Uuid,
    pub number: i32,
    pub title: String,
    pub anime_id: Uuid,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeSummary {
    pub id: Uuid,
    pub title: String,
    pub poster_url: Option<String>,
    pub episodes: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<AnimeSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUrl {
    pub url: String,
    pub quality: String,
    pub expires_at: DateTime<Utc>,
}