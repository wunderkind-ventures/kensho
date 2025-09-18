use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anime {
    pub id: String,
    pub title: String,
    pub description: String,
    pub poster_url: String,
    #[serde(rename = "episodes", default, alias = "episode_count")]
    pub episode_count: i32,
    pub status: String,
    pub anime_type: String,
    #[serde(rename = "imdb_rating", alias = "rating")]
    pub rating: Option<f32>,
    #[serde(default)]
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
    #[serde(rename = "episodes", default, alias = "episode_count")]
    pub episode_count: i32,
    pub status: String,
    #[serde(rename = "anime_type")]
    pub anime_type: String,
    #[serde(rename = "imdb_rating", alias = "rating")]
    pub rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<AnimeSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeasonalBrowseResponse {
    pub year: i32,
    pub season: String,
    pub anime: Vec<AnimeSummary>,
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