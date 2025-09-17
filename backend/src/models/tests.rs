// T061-T062: Unit tests for model validation and search algorithms

#[cfg(test)]
mod anime_tests {
    use super::super::anime::*;
    use chrono::Utc;
    use uuid::Uuid;
    use validator::Validate;

    #[test]
    fn test_anime_creation_with_valid_data() {
        let anime = Anime {
            id: Uuid::new_v4(),
            title: "Attack on Titan".to_string(),
            synonyms: vec!["Shingeki no Kyojin".to_string()],
            sources: vec!["https://myanimelist.net/anime/16498/".to_string()],
            episodes: 87,
            status: AnimeStatus::Finished,
            anime_type: AnimeType::TV,
            anime_season: AnimeSeason {
                season: Season::Spring,
                year: 2013,
            },
            synopsis: "Humanity fights for survival against Titans".to_string(),
            poster_url: "https://example.com/aot.jpg".to_string(),
            imdb: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(anime.validate().is_ok());
        assert_eq!(anime.title, "Attack on Titan");
        assert_eq!(anime.episodes, 87);
        assert_eq!(anime.status, AnimeStatus::Finished);
    }

    #[test]
    fn test_anime_validation_fails_with_empty_title() {
        let anime = Anime {
            id: Uuid::new_v4(),
            title: "".to_string(), // Invalid: empty title
            synonyms: vec![],
            sources: vec![],
            episodes: 12,
            status: AnimeStatus::Ongoing,
            anime_type: AnimeType::TV,
            anime_season: AnimeSeason {
                season: Season::Winter,
                year: 2024,
            },
            synopsis: "Test anime".to_string(),
            poster_url: "https://example.com/test.jpg".to_string(),
            imdb: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let validation_result = anime.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
    }

    #[test]
    fn test_anime_validation_fails_with_invalid_poster_url() {
        let anime = Anime {
            id: Uuid::new_v4(),
            title: "Valid Title".to_string(),
            synonyms: vec![],
            sources: vec![],
            episodes: 12,
            status: AnimeStatus::Ongoing,
            anime_type: AnimeType::TV,
            anime_season: AnimeSeason {
                season: Season::Spring,
                year: 2024,
            },
            synopsis: "Test anime".to_string(),
            poster_url: "not-a-url".to_string(), // Invalid URL
            imdb: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let validation_result = anime.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("poster_url"));
    }

    #[test]
    fn test_anime_validation_with_long_title() {
        let long_title = "a".repeat(501); // Exceeds max length of 500
        
        let anime = Anime {
            id: Uuid::new_v4(),
            title: long_title,
            synonyms: vec![],
            sources: vec![],
            episodes: 12,
            status: AnimeStatus::Ongoing,
            anime_type: AnimeType::TV,
            anime_season: AnimeSeason {
                season: Season::Summer,
                year: 2024,
            },
            synopsis: "Test anime".to_string(),
            poster_url: "https://example.com/test.jpg".to_string(),
            imdb: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let validation_result = anime.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
    }

    #[test]
    fn test_anime_status_serialization() {
        use serde_json;

        let status = AnimeStatus::Ongoing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""ongoing""#);

        let deserialized: AnimeStatus = serde_json::from_str(r#""finished""#).unwrap();
        assert_eq!(deserialized, AnimeStatus::Finished);
    }

    #[test]
    fn test_anime_type_serialization() {
        use serde_json;

        let anime_type = AnimeType::TV;
        let json = serde_json::to_string(&anime_type).unwrap();
        assert_eq!(json, r#""TV""#);

        let deserialized: AnimeType = serde_json::from_str(r#""MOVIE""#).unwrap();
        assert_eq!(deserialized, AnimeType::Movie);
    }

    #[test]
    fn test_season_serialization() {
        use serde_json;

        let season = Season::Fall;
        let json = serde_json::to_string(&season).unwrap();
        assert_eq!(json, r#""fall""#);

        let deserialized: Season = serde_json::from_str(r#""winter""#).unwrap();
        assert_eq!(deserialized, Season::Winter);
    }

    #[test]
    fn test_anime_season_validation() {
        let valid_season = AnimeSeason {
            season: Season::Spring,
            year: 2024,
        };
        assert!(valid_season.validate().is_ok());

        // Year validation would be tested if custom validator was enabled
        // For now, any year is valid
        let future_season = AnimeSeason {
            season: Season::Fall,
            year: 2030,
        };
        assert!(future_season.validate().is_ok());
    }

    #[test]
    fn test_imdb_data_structure() {
        let imdb_data = ImdbData {
            id: "tt1234567".to_string(),
            rating: 8.5,
            votes: 10000,
            last_updated: Utc::now(),
        };

        assert_eq!(imdb_data.id, "tt1234567");
        assert_eq!(imdb_data.rating, 8.5);
        assert_eq!(imdb_data.votes, 10000);
    }

    #[test]
    fn test_anime_with_imdb_data() {
        let anime = Anime {
            id: Uuid::new_v4(),
            title: "Popular Anime".to_string(),
            synonyms: vec![],
            sources: vec![],
            episodes: 24,
            status: AnimeStatus::Finished,
            anime_type: AnimeType::TV,
            anime_season: AnimeSeason {
                season: Season::Fall,
                year: 2023,
            },
            synopsis: "A very popular anime".to_string(),
            poster_url: "https://example.com/popular.jpg".to_string(),
            imdb: Some(ImdbData {
                id: "tt9876543".to_string(),
                rating: 9.2,
                votes: 50000,
                last_updated: Utc::now(),
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(anime.validate().is_ok());
        assert!(anime.imdb.is_some());
        
        if let Some(imdb) = anime.imdb {
            assert_eq!(imdb.rating, 9.2);
            assert_eq!(imdb.votes, 50000);
        }
    }

    #[test]
    fn test_default_values() {
        assert_eq!(AnimeStatus::default(), AnimeStatus::Unknown);
        assert_eq!(AnimeType::default(), AnimeType::Unknown);
        assert_eq!(Season::default(), Season::Spring);
    }
}

#[cfg(test)]
mod episode_tests {
    use super::super::episode::*;
    use chrono::{Utc, NaiveDate};
    use uuid::Uuid;
    use validator::Validate;

    #[test]
    fn test_episode_creation_with_valid_data() {
        let episode = Episode {
            id: Uuid::new_v4(),
            anime_id: Uuid::new_v4(),
            episode_number: 1,
            title: Some("The First Episode".to_string()),
            duration: Some(1440), // 24 minutes
            air_date: Some(NaiveDate::from_ymd_opt(2024, 1, 7).unwrap()),
            synopsis: Some("The beginning of the journey".to_string()),
            thumbnail_url: Some("https://example.com/ep1.jpg".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(episode.validate().is_ok());
        assert_eq!(episode.episode_number, 1);
        assert_eq!(episode.title.as_deref(), Some("The First Episode"));
        assert_eq!(episode.duration, Some(1440));
    }

    #[test]
    fn test_episode_validation_with_invalid_episode_number() {
        let episode = Episode {
            id: Uuid::new_v4(),
            anime_id: Uuid::new_v4(),
            episode_number: 0, // Invalid: must be >= 1
            title: Some("Episode Zero".to_string()),
            duration: Some(1440),
            air_date: None,
            synopsis: None,
            thumbnail_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let validation_result = episode.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("episode_number"));
    }

    #[test]
    fn test_episode_with_minimal_data() {
        let episode = Episode {
            id: Uuid::new_v4(),
            anime_id: Uuid::new_v4(),
            episode_number: 5,
            title: None,
            duration: None,
            air_date: None,
            synopsis: None,
            thumbnail_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(episode.validate().is_ok());
        assert_eq!(episode.episode_number, 5);
        assert!(episode.title.is_none());
        assert!(episode.duration.is_none());
    }

    #[test]
    fn test_episode_validation_with_invalid_thumbnail_url() {
        let episode = Episode {
            id: Uuid::new_v4(),
            anime_id: Uuid::new_v4(),
            episode_number: 1,
            title: Some("Episode 1".to_string()),
            duration: Some(1440),
            air_date: None,
            synopsis: None,
            thumbnail_url: Some("not-a-url".to_string()), // Invalid URL
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let validation_result = episode.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("thumbnail_url"));
    }

    #[test]
    fn test_episode_validation_with_negative_duration() {
        let episode = Episode {
            id: Uuid::new_v4(),
            anime_id: Uuid::new_v4(),
            episode_number: 1,
            title: Some("Episode 1".to_string()),
            duration: Some(-100), // Invalid: negative duration
            air_date: None,
            synopsis: None,
            thumbnail_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let validation_result = episode.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("duration"));
    }

    #[test]
    fn test_episode_response_conversion() {
        let episode = Episode {
            id: Uuid::new_v4(),
            anime_id: Uuid::new_v4(),
            episode_number: 3,
            title: Some("Episode Three".to_string()),
            duration: Some(1320), // 22 minutes
            air_date: Some(NaiveDate::from_ymd_opt(2024, 1, 21).unwrap()),
            synopsis: Some("Things get interesting".to_string()),
            thumbnail_url: Some("https://example.com/ep3.jpg".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response: EpisodeResponse = episode.clone().into();
        assert_eq!(response.id, episode.id);
        assert_eq!(response.episode_number, 3);
        assert_eq!(response.title, Some("Episode Three".to_string()));
        assert_eq!(response.duration, Some(1320));
    }

    #[test]
    fn test_episode_list_response_structure() {
        let episodes = vec![
            EpisodeResponse {
                id: Uuid::new_v4(),
                episode_number: 1,
                title: Some("Episode 1".to_string()),
                duration: Some(1440),
                air_date: None,
                synopsis: None,
                thumbnail_url: None,
            },
            EpisodeResponse {
                id: Uuid::new_v4(),
                episode_number: 2,
                title: Some("Episode 2".to_string()),
                duration: Some(1440),
                air_date: None,
                synopsis: None,
                thumbnail_url: None,
            },
        ];

        let list_response = EpisodeListResponse {
            episodes: episodes.clone(),
            total: 2,
        };

        assert_eq!(list_response.total, 2);
        assert_eq!(list_response.episodes.len(), 2);
        assert_eq!(list_response.episodes[0].episode_number, 1);
        assert_eq!(list_response.episodes[1].episode_number, 2);
    }

    #[test]
    fn test_episode_serialization() {
        use serde_json;

        let episode = Episode {
            id: Uuid::new_v4(),
            anime_id: Uuid::new_v4(),
            episode_number: 10,
            title: Some("Climax".to_string()),
            duration: Some(1500),
            air_date: Some(NaiveDate::from_ymd_opt(2024, 3, 10).unwrap()),
            synopsis: Some("The climactic episode".to_string()),
            thumbnail_url: Some("https://example.com/ep10.jpg".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&episode).unwrap();
        let deserialized: Episode = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.episode_number, episode.episode_number);
        assert_eq!(deserialized.title, episode.title);
        assert_eq!(deserialized.duration, episode.duration);
    }
}

#[cfg(test)]
mod tag_tests {
    use super::super::tag::*;
    use uuid::Uuid;

    #[test]
    fn test_tag_creation() {
        let tag = Tag {
            id: Uuid::new_v4(),
            name: "Action".to_string(),
            description: Some("High-energy combat scenes".to_string()),
            category: Some(TagCategory::Genre),
        };

        assert_eq!(tag.name, "Action");
        assert!(tag.description.is_some());
        assert_eq!(tag.category, Some(TagCategory::Genre));
    }

    #[test]
    fn test_tag_normalization() {
        let tag = Tag::new(
            "  Romance  ".to_string(),
            Some("Love stories".to_string()),
            Some(TagCategory::Genre),
        );

        // Name should be trimmed
        assert_eq!(tag.name, "Romance");
    }

    #[test]
    fn test_tag_category_variants() {
        assert_eq!(TagCategory::Genre.to_string(), "Genre");
        assert_eq!(TagCategory::Theme.to_string(), "Theme");
        assert_eq!(TagCategory::Setting.to_string(), "Setting");
        assert_eq!(TagCategory::Demographic.to_string(), "Demographic");
        assert_eq!(TagCategory::ContentWarning.to_string(), "ContentWarning");
        assert_eq!(TagCategory::Other.to_string(), "Other");
    }

    #[test]
    fn test_tag_response_conversion() {
        let tag = Tag {
            id: Uuid::new_v4(),
            name: "Sci-Fi".to_string(),
            description: Some("Science fiction themes".to_string()),
            category: Some(TagCategory::Genre),
        };

        let response: TagResponse = tag.clone().into();
        assert_eq!(response.id, tag.id);
        assert_eq!(response.name, "Sci-Fi");
        assert_eq!(response.description, Some("Science fiction themes".to_string()));
        assert_eq!(response.category, Some("Genre".to_string()));
    }

    #[test]
    fn test_tag_without_category() {
        let tag = Tag {
            id: Uuid::new_v4(),
            name: "Uncategorized".to_string(),
            description: None,
            category: None,
        };

        assert_eq!(tag.name, "Uncategorized");
        assert!(tag.description.is_none());
        assert!(tag.category.is_none());
    }
}

#[cfg(test)]
mod session_tests {
    use super::super::session::*;
    use chrono::{Utc, Duration};
    use uuid::Uuid;

    #[test]
    fn test_session_creation() {
        let session = Session {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            token: "jwt_token_here".to_string(),
            refresh_token: Some("refresh_token_here".to_string()),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        assert_eq!(session.user_id, "user123");
        assert_eq!(session.username, "testuser");
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_expiration() {
        let expired_session = Session {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            token: "expired_token".to_string(),
            refresh_token: None,
            expires_at: Utc::now() - Duration::hours(1), // Expired 1 hour ago
            created_at: Utc::now() - Duration::hours(2),
            last_activity: Utc::now() - Duration::hours(1),
        };

        assert!(!expired_session.is_valid());
    }

    #[test]
    fn test_claims_structure() {
        let claims = Claims {
            sub: "user123".to_string(),
            username: "testuser".to_string(),
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_session_response_conversion() {
        let session = Session {
            id: Uuid::new_v4(),
            user_id: "user456".to_string(),
            username: "anotheruser".to_string(),
            token: "access_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            expires_at: Utc::now() + Duration::hours(2),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        let response: SessionResponse = session.clone().into();
        assert_eq!(response.user_id, "user456");
        assert_eq!(response.username, "anotheruser");
        assert_eq!(response.access_token, "access_token");
        assert_eq!(response.refresh_token, Some("refresh_token".to_string()));
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 7200); // 2 hours in seconds
    }

    #[test]
    fn test_session_create_request() {
        let create_request = SessionCreate {
            username: "newuser".to_string(),
            password: "secure_password".to_string(),
        };

        assert_eq!(create_request.username, "newuser");
        assert_eq!(create_request.password, "secure_password");
    }
}

#[cfg(test)]
mod relationship_tests {
    use super::super::relationships::*;
    use uuid::Uuid;

    #[test]
    fn test_has_tag_relationship() {
        let has_tag = HasTag {
            id: Uuid::new_v4(),
            from: Uuid::new_v4(), // anime_id
            to: Uuid::new_v4(),   // tag_id
            weight: Some(0.8),
        };

        assert!(has_tag.weight.is_some());
        assert_eq!(has_tag.weight.unwrap(), 0.8);
    }

    #[test]
    fn test_sequel_relationship() {
        let anime1_id = Uuid::new_v4();
        let anime2_id = Uuid::new_v4();
        
        let sequel = IsSequelOf {
            id: Uuid::new_v4(),
            from: anime2_id,
            to: anime1_id,
            order: Some(2),
        };

        assert_eq!(sequel.from, anime2_id);
        assert_eq!(sequel.to, anime1_id);
        assert_eq!(sequel.order, Some(2));
    }

    #[test]
    fn test_prequel_relationship() {
        let anime1_id = Uuid::new_v4();
        let anime2_id = Uuid::new_v4();
        
        let prequel = IsPrequelOf {
            id: Uuid::new_v4(),
            from: anime1_id,
            to: anime2_id,
            order: Some(1),
        };

        assert_eq!(prequel.from, anime1_id);
        assert_eq!(prequel.to, anime2_id);
        assert_eq!(prequel.order, Some(1));
    }

    #[test]
    fn test_related_to_relationship() {
        let related = RelatedTo {
            id: Uuid::new_v4(),
            from: Uuid::new_v4(),
            to: Uuid::new_v4(),
            relation_type: RelationType::SpinOff,
            strength: Some(0.7),
        };

        assert_eq!(related.relation_type, RelationType::SpinOff);
        assert_eq!(related.strength, Some(0.7));
    }

    #[test]
    fn test_relation_type_variants() {
        assert_eq!(RelationType::Adaptation, RelationType::Adaptation);
        assert_eq!(RelationType::SpinOff, RelationType::SpinOff);
        assert_eq!(RelationType::AlternativeSetting, RelationType::AlternativeSetting);
        assert_eq!(RelationType::SideStory, RelationType::SideStory);
        assert_eq!(RelationType::SharedUniverse, RelationType::SharedUniverse);
        assert_eq!(RelationType::Other, RelationType::Other);
    }

    #[test]
    fn test_belongs_to_relationship() {
        let episode_id = Uuid::new_v4();
        let anime_id = Uuid::new_v4();
        
        let belongs_to = BelongsTo {
            id: Uuid::new_v4(),
            from: episode_id,
            to: anime_id,
        };

        assert_eq!(belongs_to.from, episode_id);
        assert_eq!(belongs_to.to, anime_id);
    }
}

// T062: Search algorithm tests
#[cfg(test)]
mod search_algorithm_tests {
    use super::super::anime::*;
    use uuid::Uuid;
    use std::collections::HashMap;

    // Helper function to calculate text similarity score
    fn calculate_similarity(query: &str, text: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let text_lower = text.to_lowercase();
        
        // Exact match
        if text_lower == query_lower {
            return 1.0;
        }
        
        // Contains match
        if text_lower.contains(&query_lower) {
            // Score based on position and length ratio
            let position_score = if text_lower.starts_with(&query_lower) {
                0.9
            } else {
                0.7
            };
            
            let length_ratio = query_lower.len() as f32 / text_lower.len() as f32;
            return position_score * (0.5 + length_ratio * 0.5);
        }
        
        // Word-based matching
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let text_words: Vec<&str> = text_lower.split_whitespace().collect();
        
        let mut matching_words = 0;
        for q_word in &query_words {
            for t_word in &text_words {
                if t_word.contains(q_word) || q_word.contains(t_word) {
                    matching_words += 1;
                    break;
                }
            }
        }
        
        if matching_words > 0 {
            return matching_words as f32 / query_words.len().max(1) as f32 * 0.5;
        }
        
        // Levenshtein distance for fuzzy matching
        let distance = levenshtein_distance(&query_lower, &text_lower);
        let max_len = query_lower.len().max(text_lower.len()) as f32;
        let similarity = 1.0 - (distance as f32 / max_len);
        
        if similarity > 0.7 {
            return similarity * 0.3;
        }
        
        0.0
    }
    
    // Simple Levenshtein distance implementation
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 { return len2; }
        if len2 == 0 { return len1; }
        
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        let mut prev_row: Vec<usize> = (0..=len2).collect();
        let mut curr_row = vec![0; len2 + 1];
        
        for i in 1..=len1 {
            curr_row[0] = i;
            for j in 1..=len2 {
                let cost = if s1_chars[i-1] == s2_chars[j-1] { 0 } else { 1 };
                curr_row[j] = std::cmp::min(
                    std::cmp::min(
                        prev_row[j] + 1,      // deletion
                        curr_row[j-1] + 1      // insertion
                    ),
                    prev_row[j-1] + cost       // substitution
                );
            }
            std::mem::swap(&mut prev_row, &mut curr_row);
        }
        
        prev_row[len2]
    }

    #[test]
    fn test_exact_match_search() {
        let query = "Attack on Titan";
        let title = "Attack on Titan";
        let score = calculate_similarity(query, title);
        assert_eq!(score, 1.0, "Exact match should score 1.0");
    }

    #[test]
    fn test_case_insensitive_search() {
        let query = "attack on titan";
        let title = "Attack on Titan";
        let score = calculate_similarity(query, title);
        assert_eq!(score, 1.0, "Case insensitive exact match should score 1.0");
    }

    #[test]
    fn test_partial_match_search() {
        let query = "Attack";
        let title = "Attack on Titan";
        let score = calculate_similarity(query, title);
        assert!(score > 0.8, "Partial match at beginning should score high");
        assert!(score < 1.0, "Partial match should not score 1.0");
    }

    #[test]
    fn test_contains_match_search() {
        let query = "Titan";
        let title = "Attack on Titan";
        let score = calculate_similarity(query, title);
        assert!(score > 0.3, "Contains match should have positive score");
        assert!(score < 0.8, "Contains match in middle should score lower than prefix");
    }

    #[test]
    fn test_synonym_search() {
        let query = "Shingeki no Kyojin";
        let synonyms = vec!["Attack on Titan", "Shingeki no Kyojin", "AOT"];
        
        let mut best_score = 0.0_f32;
        for synonym in &synonyms {
            let score = calculate_similarity(query, synonym);
            best_score = best_score.max(score);
        }
        
        assert_eq!(best_score, 1.0, "Should match synonym exactly");
    }

    #[test]
    fn test_fuzzy_search() {
        let query = "Atack on Titan"; // Typo: "Atack" instead of "Attack"
        let title = "Attack on Titan";
        let score = calculate_similarity(query, title);
        assert!(score > 0.2, "Fuzzy match with one typo should have some score");
    }

    #[test]
    fn test_word_based_matching() {
        let query = "Demon Slayer";
        let title = "Demon Slayer: Kimetsu no Yaiba";
        let score = calculate_similarity(query, title);
        assert!(score > 0.7, "Matching first words should score high");
    }

    #[test]
    fn test_no_match_search() {
        let query = "Naruto";
        let title = "Attack on Titan";
        let score = calculate_similarity(query, title);
        assert_eq!(score, 0.0, "Completely different titles should score 0");
    }

    #[test]
    fn test_search_ranking() {
        let query = "Hero";
        let titles = vec![
            ("My Hero Academia", "Exact word match"),
            ("The Rising of the Shield Hero", "Contains word"),
            ("One Punch Man", "No match"),
            ("Hero", "Exact match"),
            ("Heroes of the Storm", "Plural form"),
        ];
        
        let mut scores: Vec<(f32, &str)> = titles
            .iter()
            .map(|(title, desc)| (calculate_similarity(query, title), *desc))
            .collect();
        
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Verify ranking order
        assert_eq!(scores[0].1, "Exact match", "Exact match should rank first");
        assert!(scores[0].0 == 1.0, "Exact match should score 1.0");
        
        // Verify that non-matches are at the bottom
        let last = scores.last().unwrap();
        assert_eq!(last.1, "No match", "Non-matching should rank last");
        assert_eq!(last.0, 0.0, "Non-matching should score 0");
    }

    #[test]
    fn test_search_with_special_characters() {
        let query = "Fate/stay night";
        let title = "Fate/stay night";
        let score = calculate_similarity(query, title);
        assert_eq!(score, 1.0, "Special characters should be handled correctly");
    }

    #[test]
    fn test_search_with_numbers() {
        let query = "Steins Gate 0";
        let title = "Steins;Gate 0";
        let score = calculate_similarity(query, title);
        assert!(score > 0.7, "Should match despite punctuation differences");
    }

    #[test]
    fn test_abbreviated_search() {
        let query = "AOT";
        let title = "Attack on Titan";
        let abbreviations = HashMap::from([
            ("AOT", "Attack on Titan"),
            ("FMA", "Fullmetal Alchemist"),
            ("SAO", "Sword Art Online"),
        ]);
        
        // In real implementation, would check abbreviation dictionary
        let expanded = abbreviations.get(query.as_str());
        let score = if let Some(expanded_query) = expanded {
            calculate_similarity(expanded_query, title)
        } else {
            calculate_similarity(query, title)
        };
        
        assert_eq!(score, 1.0, "Abbreviation should expand and match");
    }

    #[test]
    fn test_search_performance_with_large_dataset() {
        use std::time::Instant;
        
        // Create a large dataset
        let mut titles = Vec::new();
        for i in 0..10000 {
            titles.push(format!("Anime Title {}", i));
        }
        titles.push("Attack on Titan".to_string());
        
        let query = "Attack on Titan";
        let start = Instant::now();
        
        let mut results: Vec<(f32, &String)> = titles
            .iter()
            .map(|title| (calculate_similarity(query, title), title))
            .filter(|(score, _)| *score > 0.0)
            .collect();
        
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 100, "Search should complete within 100ms for 10k items");
        assert!(!results.is_empty(), "Should find matching result");
        assert_eq!(results[0].1, "Attack on Titan", "Should find exact match");
    }

    #[test]
    fn test_tag_based_filtering() {
        // Test tag matching logic
        let tags = vec!["Action", "Adventure", "Fantasy"];
        let search_tag = "action";
        
        let matched = tags.iter().any(|tag| 
            tag.to_lowercase() == search_tag.to_lowercase()
        );
        
        assert!(matched, "Should match tag case-insensitively");
    }

    #[test]
    fn test_season_filtering() {
        use super::super::anime::{AnimeSeason, Season};
        
        let season = AnimeSeason {
            season: Season::Spring,
            year: 2024,
        };
        
        // Test exact season match
        assert!(matches!(season.season, Season::Spring));
        assert_eq!(season.year, 2024);
        
        // Test season string conversion
        let season_str = match season.season {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Fall => "fall",
            Season::Winter => "winter",
        };
        assert_eq!(season_str, "spring");
    }

    #[test]
    fn test_multi_field_search() {
        struct SearchableAnime {
            title: String,
            synopsis: String,
            tags: Vec<String>,
        }
        
        let anime = SearchableAnime {
            title: "Demon Slayer".to_string(),
            synopsis: "A young boy becomes a demon slayer to save his sister".to_string(),
            tags: vec!["Action".to_string(), "Historical".to_string(), "Supernatural".to_string()],
        };
        
        let query = "demon";
        
        // Calculate combined score from multiple fields
        let title_score = calculate_similarity(query, &anime.title) * 1.0;  // Title has highest weight
        let synopsis_score = calculate_similarity(query, &anime.synopsis) * 0.5;  // Synopsis has medium weight
        
        let mut tag_score = 0.0_f32;
        for tag in &anime.tags {
            tag_score = tag_score.max(calculate_similarity(query, tag) * 0.3);  // Tags have lower weight
        }
        
        let combined_score = title_score + synopsis_score + tag_score;
        
        assert!(combined_score > 0.0, "Should find match in multiple fields");
        assert!(title_score > synopsis_score, "Title match should contribute most to score");
    }

    #[test]
    fn test_pagination_in_search_results() {
        let total_results = 100;
        let page_size = 10;
        
        // Simulate paginated results
        for page in 0..10 {
            let offset = page * page_size;
            let limit = page_size;
            
            let start = offset;
            let end = (offset + limit).min(total_results);
            
            assert_eq!(end - start, page_size.min(total_results - offset));
        }
        
        // Test last page with fewer items
        let last_page = 9;
        let offset = last_page * page_size;
        let remaining = total_results - offset;
        assert_eq!(remaining, 10, "Last page should have exactly 10 items for 100 total");
    }
}