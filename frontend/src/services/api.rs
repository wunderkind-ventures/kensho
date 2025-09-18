use gloo_net::http::Request;
use serde::Serialize;
use crate::models::*;

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        // Use relative URL for same-origin requests
        Self {
            base_url: "/api".to_string(),
        }
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    fn request(&self, path: &str) -> gloo_net::http::RequestBuilder {
        Request::get(&format!("{}{}", self.base_url, path))
    }

    fn request_with_auth(&self, path: &str, token: &str) -> gloo_net::http::RequestBuilder {
        Request::get(&format!("{}{}", self.base_url, path))
            .header("Authorization", &format!("Bearer {}", token))
    }

    fn post_json<T: Serialize>(&self, path: &str, body: &T) -> Result<gloo_net::http::Request, gloo_net::Error> {
        Request::post(&format!("{}{}", self.base_url, path))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(body).unwrap())
    }

    fn post_json_with_auth<T: Serialize>(&self, path: &str, body: &T, token: &str) -> Result<gloo_net::http::Request, gloo_net::Error> {
        Request::post(&format!("{}{}", self.base_url, path))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .body(serde_json::to_string(body).unwrap())
    }

    // Health check
    pub async fn health_check(&self) -> Result<bool, String> {
        match self.request("/health").send().await {
            Ok(resp) if resp.ok() => Ok(true),
            Ok(resp) => Err(format!("Health check failed: {}", resp.status())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    // Authentication endpoints
    pub async fn login(&self, email: String, password: String) -> Result<LoginResponse, String> {
        let req = LoginRequest { email, password };
        
        match self.post_json("/auth/login", &req).unwrap().send().await {
            Ok(resp) if resp.ok() => {
                resp.json::<LoginResponse>().await
                    .map_err(|e| format!("Failed to parse response: {}", e))
            },
            Ok(resp) => Err(format!("Login failed: {}", resp.status_text())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    pub async fn logout(&self, token: &str) -> Result<(), String> {
        match Request::post(&format!("{}/auth/logout", self.base_url))
            .header("Authorization", &format!("Bearer {}", token))
            .send().await {
            Ok(resp) if resp.ok() => Ok(()),
            Ok(resp) => Err(format!("Logout failed: {}", resp.status())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    // Anime endpoints
    pub async fn get_anime(&self, id: &str) -> Result<Anime, String> {
        match self.request(&format!("/anime/{}", id)).send().await {
            Ok(resp) if resp.ok() => {
                resp.json::<Anime>().await
                    .map_err(|e| format!("Failed to parse anime: {}", e))
            },
            Ok(resp) => Err(format!("Failed to get anime: {}", resp.status())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    pub async fn search_anime(&self, query: &str) -> Result<SearchResponse, String> {
        let url = format!("/search?q={}", urlencoding::encode(query));
        
        match self.request(&url).send().await {
            Ok(resp) if resp.ok() => {
                resp.json::<SearchResponse>().await
                    .map_err(|e| format!("Failed to parse search results: {}", e))
            },
            Ok(resp) => Err(format!("Search failed: {}", resp.status())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    // Alias for search_anime for compatibility
    pub async fn search(&self, query: &str) -> Result<Vec<AnimeSummary>, String> {
        self.search_anime(query).await.map(|resp| resp.results)
    }

    pub async fn browse_seasonal(&self, year: i32, season: &str) -> Result<Vec<AnimeSummary>, String> {
        let url = format!("/browse/season/{}/{}", year, season);
        
        match self.request(&url).send().await {
            Ok(resp) if resp.ok() => {
                resp.json::<SeasonalBrowseResponse>().await
                    .map(|r| r.anime)
                    .map_err(|e| format!("Failed to parse seasonal anime: {}", e))
            },
            Ok(resp) => Err(format!("Browse failed: {}", resp.status())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    // Episode endpoints
    pub async fn get_episodes(&self, anime_id: &str) -> Result<Vec<Episode>, String> {
        let url = format!("/anime/{}/episodes", anime_id);
        
        match self.request(&url).send().await {
            Ok(resp) if resp.ok() => {
                resp.json::<Vec<Episode>>().await
                    .map_err(|e| format!("Failed to parse episodes: {}", e))
            },
            Ok(resp) => Err(format!("Failed to get episodes: {}", resp.status())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    // Streaming endpoint (requires authentication)
    pub async fn get_stream_url(&self, anime_id: &str, episode: i32, token: &str) -> Result<StreamUrl, String> {
        let url = format!("/stream/{}/{}", anime_id, episode);
        
        match self.request_with_auth(&url, token).send().await {
            Ok(resp) if resp.ok() => {
                resp.json::<StreamUrl>().await
                    .map_err(|e| format!("Failed to parse stream URL: {}", e))
            },
            Ok(resp) if resp.status() == 401 => Err("Authentication required".to_string()),
            Ok(resp) => Err(format!("Failed to get stream: {}", resp.status())),
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

// Utility module for URL encoding
mod urlencoding {
    pub fn encode(s: &str) -> String {
        js_sys::encode_uri_component(s).as_string().unwrap()
    }
}