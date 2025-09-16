// Cache service for Redis operations

use anyhow::{Result, Context};
use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

pub struct CacheService {
    client: redis::aio::ConnectionManager,
}

impl CacheService {
    pub async fn new(redis_url: &str) -> Result<Self> {
        tracing::debug!("Creating Redis client for cache service with URL: {}", redis_url);
        let client = redis::Client::open(redis_url)
            .context("Failed to create Redis client for cache")?;
        
        tracing::debug!("Establishing Redis connection for cache...");
        let conn = redis::aio::ConnectionManager::new(client).await
            .context("Failed to establish Redis connection for cache")?;
        
        tracing::debug!("Cache service Redis connection established");
        
        Ok(CacheService { client: conn })
    }
    
    pub async fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>> {
        let data: Option<String> = self.client
            .get(key)
            .await
            .ok();
        
        match data {
            Some(json) => {
                let value = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            None => Ok(None)
        }
    }
    
    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let json = serde_json::to_string(value)?;
        
        self.client
            .set_ex(key, json, ttl.as_secs())
            .await?;
        
        Ok(())
    }
    
    pub async fn delete(&mut self, key: &str) -> Result<()> {
        let _: () = self.client.del(key).await?;
        Ok(())
    }
    
    pub async fn exists(&mut self, key: &str) -> Result<bool> {
        let exists: bool = self.client.exists(key).await?;
        Ok(exists)
    }
    
    pub async fn expire(&mut self, key: &str, ttl: Duration) -> Result<()> {
        let _: bool = self.client
            .expire(key, ttl.as_secs() as i64)
            .await?;
        
        Ok(())
    }
    
    // Cache keys for different entities
    pub fn anime_key(id: &str) -> String {
        format!("anime:{}", id)
    }
    
    pub fn episode_key(anime_id: &str, episode_num: u32) -> String {
        format!("episode:{}:{}", anime_id, episode_num)
    }
    
    pub fn search_key(query: &str) -> String {
        format!("search:{}", query.to_lowercase().replace(" ", "_"))
    }
    
    pub fn stream_key(episode_id: &str) -> String {
        format!("stream:{}", episode_id)
    }
    
    // Batch operations
    pub async fn get_many<T: DeserializeOwned>(&mut self, keys: &[String]) -> Result<Vec<Option<T>>> {
        let mut results = Vec::new();
        
        for key in keys {
            results.push(self.get(key).await?);
        }
        
        Ok(results)
    }
    
    pub async fn invalidate_pattern(&mut self, pattern: &str) -> Result<usize> {
        let keys: Vec<String> = self.client.keys(pattern).await?;
        let count = keys.len();
        
        for key in keys {
            self.delete(&key).await?;
        }
        
        Ok(count)
    }
}

// Cache-aside pattern helper
pub struct CacheAside<T> {
    cache: CacheService,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> CacheAside<T> {
    pub fn new(cache: CacheService) -> Self {
        CacheAside {
            cache,
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub async fn get_or_fetch<F, Fut>(
        &mut self,
        key: &str,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try cache first
        if let Some(cached) = self.cache.get(key).await? {
            return Ok(cached);
        }
        
        // Fetch from source
        let value = fetch_fn().await?;
        
        // Store in cache
        self.cache.set(key, &value, ttl).await?;
        
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires Redis running
    async fn test_cache_operations() {
        let mut cache = CacheService::new("redis://localhost:6379").await.unwrap();
        
        // Test set and get
        #[derive(Serialize, serde::Deserialize, Debug, PartialEq)]
        struct TestData {
            id: String,
            value: i32,
        }
        
        let data = TestData {
            id: "test123".to_string(),
            value: 42,
        };
        
        cache.set("test_key", &data, Duration::from_secs(60)).await.unwrap();
        
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(data));
        
        // Test delete
        cache.delete("test_key").await.unwrap();
        let deleted: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(deleted, None);
    }
    
    #[test]
    fn test_cache_keys() {
        assert_eq!(CacheService::anime_key("123"), "anime:123");
        assert_eq!(CacheService::episode_key("456", 5), "episode:456:5");
        assert_eq!(CacheService::search_key("spy family"), "search:spy_family");
        assert_eq!(CacheService::stream_key("789"), "stream:789");
    }
}