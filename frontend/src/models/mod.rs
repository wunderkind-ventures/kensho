use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anime {
    pub id: String,
    pub title: String,
    pub description: String,
    pub poster_url: String,
    pub episode_count: i32,
    pub status: String,
    pub anime_type: String,
    pub rating: Option<f32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Episode {
    pub id: String,
    pub episode_number: i32,
    pub title: Option<String>,
    pub anime_id: String,
    pub duration_ms: i32,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimeSummary {
    pub id: String,
    pub title: String,
    pub poster_url: String,
    pub episode_count: i32,
    pub status: String,
    pub rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<AnimeSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamUrl {
    pub url: String,
    pub quality: String,
    pub expires_at: String,
}