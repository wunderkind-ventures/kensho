use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Root structure for the MyAnimeList dead entries database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadEntriesDatabase {
    /// JSON schema URL for validation
    #[serde(rename = "$schema")]
    pub schema: String,
    
    /// License information
    pub license: License,
    
    /// Repository URL
    pub repository: String,
    
    /// Last update date (YYYY-MM-DD format)
    pub last_update: String,
    
    /// Array of dead entry IDs (as strings)
    pub dead_entries: Vec<String>,
}

/// License information for the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// License name
    pub name: String,
    /// License URL
    pub url: String,
}

impl DeadEntriesDatabase {
    /// Load the dead entries database from JSON file
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let db: DeadEntriesDatabase = serde_json::from_str(&content)?;
        Ok(db)
    }
    
    /// Check if a given MAL ID is in the dead entries list
    pub fn is_dead_entry(&self, mal_id: &str) -> bool {
        self.dead_entries.contains(&mal_id.to_string())
    }
    
    /// Check if a given MAL ID (as number) is in the dead entries list
    pub fn is_dead_entry_id(&self, mal_id: u32) -> bool {
        self.is_dead_entry(&mal_id.to_string())
    }
    
    /// Get the total count of dead entries
    pub fn dead_count(&self) -> usize {
        self.dead_entries.len()
    }
    
    /// Convert to HashSet for faster lookups if needed frequently
    pub fn to_hashset(&self) -> HashSet<String> {
        self.dead_entries.iter().cloned().collect()
    }
    
    /// Get statistics about the dead entries
    pub fn get_stats(&self) -> DeadEntriesStats {
        let total_count = self.dead_entries.len();
        
        // Analyze ID ranges
        let mut numeric_ids: Vec<u32> = self.dead_entries.iter()
            .filter_map(|id| id.parse().ok())
            .collect();
        
        numeric_ids.sort();
        
        let min_id = numeric_ids.first().copied().unwrap_or(0);
        let max_id = numeric_ids.last().copied().unwrap_or(0);
        let numeric_count = numeric_ids.len();
        let non_numeric_count = total_count - numeric_count;
        
        DeadEntriesStats {
            total_count,
            numeric_count,
            non_numeric_count,
            min_id,
            max_id,
            last_update: self.last_update.clone(),
        }
    }
    
    /// Filter anime database entries by removing dead entries
    pub fn filter_anime_entries<T>(&self, entries: Vec<T>) -> Vec<T>
    where
        T: HasMalId,
    {
        let dead_set = self.to_hashset();
        entries.into_iter()
            .filter(|entry| {
                if let Some(mal_id) = entry.get_mal_id() {
                    !dead_set.contains(&mal_id)
                } else {
                    true // Keep entries without MAL IDs
                }
            })
            .collect()
    }
}

/// Statistics about the dead entries database
#[derive(Debug, Serialize, Deserialize)]
pub struct DeadEntriesStats {
    pub total_count: usize,
    pub numeric_count: usize,
    pub non_numeric_count: usize,
    pub min_id: u32,
    pub max_id: u32,
    pub last_update: String,
}

/// Trait for types that have a MAL ID
pub trait HasMalId {
    fn get_mal_id(&self) -> Option<String>;
}

// Implement for the offline database entry
impl HasMalId for crate::models::anime_offline_db::AnimeOfflineEntry {
    fn get_mal_id(&self) -> Option<String> {
        self.get_mal_id()
    }
}

// Implement for the main Anime model if needed
impl HasMalId for crate::models::Anime {
    fn get_mal_id(&self) -> Option<String> {
        // Try to extract MAL ID from sources if available
        self.sources.iter()
            .find(|s| s.contains("myanimelist.net"))
            .and_then(|s| s.split('/').last())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_entries_check() {
        let dead_db = DeadEntriesDatabase {
            schema: "test".to_string(),
            license: License {
                name: "Test License".to_string(),
                url: "http://test.com".to_string(),
            },
            repository: "test-repo".to_string(),
            last_update: "2024-01-01".to_string(),
            dead_entries: vec!["123".to_string(), "456".to_string(), "789".to_string()],
        };

        assert!(dead_db.is_dead_entry("123"));
        assert!(dead_db.is_dead_entry_id(456));
        assert!(!dead_db.is_dead_entry("999"));
        assert_eq!(dead_db.dead_count(), 3);
    }

    #[test]
    fn test_dead_entries_stats() {
        let dead_db = DeadEntriesDatabase {
            schema: "test".to_string(),
            license: License {
                name: "Test License".to_string(),
                url: "http://test.com".to_string(),
            },
            repository: "test-repo".to_string(),
            last_update: "2024-01-01".to_string(),
            dead_entries: vec![
                "10".to_string(),
                "20".to_string(),
                "30".to_string(),
                "abc".to_string(),
            ],
        };

        let stats = dead_db.get_stats();
        assert_eq!(stats.total_count, 4);
        assert_eq!(stats.numeric_count, 3);
        assert_eq!(stats.non_numeric_count, 1);
        assert_eq!(stats.min_id, 10);
        assert_eq!(stats.max_id, 30);
    }

    #[test]
    fn test_to_hashset() {
        let dead_db = DeadEntriesDatabase {
            schema: "test".to_string(),
            license: License {
                name: "Test License".to_string(),
                url: "http://test.com".to_string(),
            },
            repository: "test-repo".to_string(),
            last_update: "2024-01-01".to_string(),
            dead_entries: vec!["123".to_string(), "456".to_string()],
        };

        let hashset = dead_db.to_hashset();
        assert!(hashset.contains("123"));
        assert!(hashset.contains("456"));
        assert!(!hashset.contains("789"));
        assert_eq!(hashset.len(), 2);
    }
}
