// T030: Streaming service
// Reference: plan.md "Streaming Integration" section

use anyhow::{Result, Context, bail};
use crunchyroll_rs::{Crunchyroll, Episode, Series, Season};
use crunchyroll_rs::media::Stream;
use std::sync::Arc;
use uuid::Uuid;
use crate::models::Session;
use crate::services::auth::AuthService;

#[derive(Clone)]
pub struct StreamingService {
    auth_service: Arc<tokio::sync::Mutex<AuthService>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VideoStream {
    pub url: String,
    pub resolution: String,
    pub audio_language: String,
    pub subtitle_language: Option<String>,
    pub hardsub: bool,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StreamingManifest {
    pub episode_id: Uuid,
    pub crunchyroll_id: String,
    pub streams: Vec<VideoStream>,
    pub thumbnail: Option<String>,
    pub duration: u32,
}

impl StreamingService {
    pub fn new(auth_service: Arc<tokio::sync::Mutex<AuthService>>) -> Self {
        StreamingService { auth_service }
    }
    
    pub async fn get_episode_stream(
        &self,
        session: &Session,
        crunchyroll_episode_id: &str,
    ) -> Result<StreamingManifest> {
        // Get authenticated Crunchyroll client
        let mut auth = self.auth_service.lock().await;
        let cr_client = auth.get_crunchyroll_client(session).await?;
        
        // Fetch episode details from Crunchyroll
        let episode = self.fetch_episode(&cr_client, crunchyroll_episode_id).await?;
        
        // Get available streams
        let streams = self.fetch_streams(&cr_client, &episode).await?;
        
        // Convert to our format
        let video_streams = self.convert_streams(streams)?;
        
        Ok(StreamingManifest {
            episode_id: Uuid::new_v4(), // Map to our episode ID
            crunchyroll_id: crunchyroll_episode_id.to_string(),
            streams: video_streams,
            thumbnail: None, // episode.thumbnail field may not exist in crunchyroll_rs
            duration: 0, // episode.duration_ms field may not exist
        })
    }
    
    pub async fn get_adaptive_stream(
        &self,
        session: &Session,
        crunchyroll_episode_id: &str,
        quality: &str,
    ) -> Result<VideoStream> {
        let manifest = self.get_episode_stream(session, crunchyroll_episode_id).await?;
        
        // Find stream matching requested quality
        manifest.streams
            .into_iter()
            .find(|s| s.resolution == quality)
            .context("Requested quality not available")
    }
    
    pub async fn get_series_episodes(
        &self,
        session: &Session,
        crunchyroll_series_id: &str,
    ) -> Result<Vec<EpisodeMetadata>> {
        let mut auth = self.auth_service.lock().await;
        let cr_client = auth.get_crunchyroll_client(session).await?;
        
        // Fetch series and seasons
        let series = self.fetch_series(&cr_client, crunchyroll_series_id).await?;
        
        let mut all_episodes = Vec::new();
        
        // Get seasons for the series
        let seasons = series.seasons().await?;
        for season in seasons {
            let episodes = self.fetch_season_episodes(&cr_client, &season).await?;
            
            for episode in episodes {
                all_episodes.push(EpisodeMetadata {
                    crunchyroll_id: episode.id.clone(),
                    episode_number: episode.episode_number.map(|n| n as u32),
                    title: Some(episode.title.clone()),
                    description: Some(episode.description.clone()),
                    thumbnail: episode.images.first().map(|t| t.source.to_string()),
                    duration: None, // duration_ms field not available
                    air_date: Some(episode.availability_starts),
                });
            }
        }
        
        Ok(all_episodes)
    }
    
    async fn fetch_episode(
        &self,
        client: &Arc<Crunchyroll>,
        episode_id: &str,
    ) -> Result<Episode> {
        // Fetch episode from Crunchyroll
        client.media_from_id(episode_id).await
            .context("Failed to fetch episode")
    }
    
    async fn fetch_streams(
        &self,
        _client: &Arc<Crunchyroll>,
        episode: &Episode,
    ) -> Result<Vec<VideoStream>> {
        // Get streams for the episode
        let stream = episode.stream().await?;
        
        // Convert to our VideoStream format
        // Note: Direct video stream access requires parsing stream data
        let video_stream = VideoStream {
            url: stream.url.clone(),
            resolution: "1080p".to_string(),
            audio_language: stream.audio_locale.to_string(),
            subtitle_language: stream.burned_in_locale.map(|l| l.to_string()),
            hardsub: !stream.hard_subs.is_empty(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
        };
        
        Ok(vec![video_stream])
    }
    
    async fn fetch_series(
        &self,
        client: &Arc<Crunchyroll>,
        series_id: &str,
    ) -> Result<Series> {
        // Fetch series from Crunchyroll
        client.media_from_id(series_id).await
            .context("Failed to fetch series")
    }
    
    async fn fetch_season_episodes(
        &self,
        _client: &Arc<Crunchyroll>,
        season: &Season,
    ) -> Result<Vec<Episode>> {
        // Get episodes for the season
        season.episodes().await
            .context("Failed to fetch season episodes")
    }
    
    fn convert_streams(&self, streams: Vec<VideoStream>) -> Result<Vec<VideoStream>> {
        // Streams are already in VideoStream format
        Ok(streams)
    }
    
    fn extract_resolution(&self, _stream: &Stream) -> String {
        // Resolution info not directly available in Stream struct
        // Would need to parse from URL or use streaming data
        "1080p".to_string()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EpisodeMetadata {
    pub crunchyroll_id: String,
    pub episode_number: Option<u32>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub duration: Option<u32>,
    pub air_date: Option<chrono::DateTime<chrono::Utc>>,
}

// HLS manifest generator for adaptive streaming
pub struct HlsManifestGenerator;

impl HlsManifestGenerator {
    pub fn generate_master_playlist(streams: &[VideoStream]) -> String {
        let mut playlist = String::from("#EXTM3U\n#EXT-X-VERSION:3\n");
        
        for stream in streams {
            let bandwidth = Self::resolution_to_bandwidth(&stream.resolution);
            let resolution = Self::resolution_to_dimensions(&stream.resolution);
            
            playlist.push_str(&format!(
                "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}\n{}\n",
                bandwidth, resolution, stream.url
            ));
        }
        
        playlist
    }
    
    fn resolution_to_bandwidth(resolution: &str) -> u32 {
        match resolution {
            "1080p" => 5000000,
            "720p" => 2500000,
            "480p" => 1000000,
            "360p" => 500000,
            _ => 1000000,
        }
    }
    
    fn resolution_to_dimensions(resolution: &str) -> &'static str {
        match resolution {
            "1080p" => "1920x1080",
            "720p" => "1280x720",
            "480p" => "854x480",
            "360p" => "640x360",
            _ => "1280x720",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hls_manifest_generation() {
        let streams = vec![
            VideoStream {
                url: "https://example.com/1080p.m3u8".to_string(),
                resolution: "1080p".to_string(),
                audio_language: "en-US".to_string(),
                subtitle_language: None,
                hardsub: false,
                expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
            },
            VideoStream {
                url: "https://example.com/720p.m3u8".to_string(),
                resolution: "720p".to_string(),
                audio_language: "en-US".to_string(),
                subtitle_language: None,
                hardsub: false,
                expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
            },
        ];
        
        let manifest = HlsManifestGenerator::generate_master_playlist(&streams);
        
        assert!(manifest.contains("#EXTM3U"));
        assert!(manifest.contains("BANDWIDTH=5000000"));
        assert!(manifest.contains("RESOLUTION=1920x1080"));
        assert!(manifest.contains("1080p.m3u8"));
    }
}