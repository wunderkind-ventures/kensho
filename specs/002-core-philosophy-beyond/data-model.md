# Data Model: Personalized Content Recommendation System

**Branch**: `002-core-philosophy-beyond` | **Date**: 2025-01-18

## Overview
This document defines the data entities, relationships, and schemas for the personalized recommendation system. All entities are designed for SurrealDB's graph capabilities while maintaining compatibility with protobuf serialization for cross-service communication.

## Core Entities

### 1. UserProfile
Represents an individual user with their preferences and settings.

```typescript
type UserProfile = {
  id: string;                    // Format: "user:{uuid}"
  username: string;               // Unique username
  email: string;                  // User email (unique)
  created_at: timestamp;          // Account creation time
  updated_at: timestamp;          // Last profile update
  
  // Preferences
  preferred_audio: "sub" | "dub"; // Audio preference
  content_maturity: "all" | "teen" | "mature"; // Content filter
  preferred_genres: string[];     // Array of genre preferences
  disliked_genres: string[];      // Genres to avoid
  
  // Onboarding
  onboarding_completed: boolean;  // Has completed initial survey
  favorite_titles: string[];      // Initial favorite anime IDs
  
  // Personalization State
  preference_vector: float[];     // 512-dim embedding vector
  last_vector_update: timestamp;  // When preference vector was updated
  cluster_id: string?;            // User segment for targeting
}

// Validation Rules
- email must be valid email format
- username must be 3-50 characters, alphanumeric + underscore
- preference_vector must be exactly 512 dimensions
- favorite_titles max 10 items
```

### 2. ContentItem
Represents anime/manga content with enriched metadata.

```typescript
type ContentItem = {
  id: string;                    // Format: "content:{mal_id}"
  mal_id: number;                // MyAnimeList ID
  title: string;                 // Primary title
  title_english: string?;        // English title
  title_japanese: string?;        // Japanese title
  
  // Core Metadata
  media_type: "TV" | "Movie" | "OVA" | "Special" | "ONA" | "Manga";
  status: "airing" | "finished" | "upcoming";
  episodes: number?;              // Total episodes (null if ongoing)
  duration: number?;              // Episode duration in minutes
  release_date: date;            // First air/publish date
  end_date: date?;               // Completion date
  
  // Taxonomy
  genres: string[];              // e.g., ["Action", "Supernatural"]
  themes: string[];              // e.g., ["School", "Revenge"]
  demographics: string[];        // e.g., ["Shounen", "Seinen"]
  studios: string[];             // Production studios
  
  // Content Features
  synopsis: string;              // Full description
  synopsis_embedding: float[];   // 768-dim BERT embedding
  poster_url: string;            // Primary artwork URL
  trailer_url: string?;          // Trailer video URL
  
  // Multi-Modal Features
  artwork_embeddings: Map<string, float[]>; // Multiple artwork variants
  audio_features: float[]?;      // Audio embedding from opening
  
  // Statistics
  score: float?;                 // Average rating (1-10)
  scored_by: number;             // Number of ratings
  popularity: number;            // Popularity rank
  members: number;               // Total members/watchers
  
  // Computed
  content_vector: float[];       // 512-dim unified embedding
  quality_score: float;          // Internal quality metric (0-1)
}

// Validation Rules
- mal_id must be positive integer
- score must be between 1.0 and 10.0
- content_vector must be exactly 512 dimensions
- genres, themes, demographics must be from predefined lists
```

### 3. UserInteraction
Represents user engagement events with content.

```typescript
type UserInteraction = {
  id: string;                    // Format: "interaction:{uuid}"
  user_id: string;               // Reference to UserProfile
  content_id: string;            // Reference to ContentItem
  session_id: string;            // Session identifier
  
  // Interaction Type
  interaction_type: InteractionType;
  timestamp: timestamp;          // When interaction occurred
  
  // Context
  device_type: "web" | "mobile" | "tv" | "tablet";
  client_version: string;        // App version
  
  // Type-Specific Data
  watch_data?: WatchData;        // For watch events
  rating_data?: RatingData;      // For rating events
  search_data?: SearchData;      // For search events
  click_data?: ClickData;        // For click events
}

enum InteractionType {
  WATCH_START,
  WATCH_PROGRESS,
  WATCH_COMPLETE,
  RATING,
  FAVORITE_ADD,
  FAVORITE_REMOVE,
  SEARCH,
  CLICK,
  HOVER
}

type WatchData = {
  episode_number: number;        // Which episode
  watch_duration: number;        // Seconds watched
  total_duration: number;        // Total episode length
  completion_rate: float;        // Percentage completed (0-1)
  is_rewatch: boolean;          // Rewatching episode
}

type RatingData = {
  rating: float;                 // User rating (1-10)
  previous_rating: float?;       // If updating rating
}

// Validation Rules
- completion_rate must be between 0 and 1
- rating must be between 1 and 10
- episode_number must be positive
```

### 4. Recommendation
Represents a generated recommendation with context.

```typescript
type Recommendation = {
  id: string;                    // Format: "rec:{uuid}"
  user_id: string;               // Target user
  content_id: string;            // Recommended content
  
  // Scoring
  relevance_score: float;        // Model confidence (0-1)
  diversity_score: float;        // Diversity contribution (0-1)
  final_score: float;            // Combined score for ranking
  
  // Context
  recommendation_type: RecommendationType;
  reason: string;                // Human-readable reason
  reason_content_ids: string[];  // Supporting content IDs
  
  // Metadata
  generated_at: timestamp;       // When recommendation created
  model_version: string;         // Model that generated this
  experiment_id: string?;        // A/B test identifier
  
  // Presentation
  carousel_id: string;           // Which carousel this belongs to
  position: number;              // Position in carousel
  artwork_variant: string;       // Which artwork to show
}

enum RecommendationType {
  CONTENT_BASED,      // Similar content
  COLLABORATIVE,      // Users also liked
  TRENDING,          // Popular now
  NEW_RELEASE,       // Recently added
  CONTINUE_WATCHING, // Resume content
  DIVERSE,           // Exploration recommendation
  PERSONALIZED_NEW   // New content matching taste
}

// Validation Rules
- scores must be between 0 and 1
- position must be positive integer
```

### 5. SessionContext
Represents the current user session with intent predictions.

```typescript
type SessionContext = {
  id: string;                    // Format: "session:{uuid}"
  user_id: string;               // Session owner
  
  // Session Info
  started_at: timestamp;         // Session start
  last_activity: timestamp;      // Last interaction
  expires_at: timestamp;         // Session expiration
  
  // Behavioral Tokens
  interaction_sequence: string[]; // Tokenized interaction sequence
  context_window: Interaction[]; // Last N interactions
  
  // Intent Prediction
  intent_embedding: float[];     // 256-dim intent vector
  predicted_intents: IntentPrediction[];
  primary_intent: UserIntent;    // Dominant intent
  
  // Context Signals
  time_of_day: "morning" | "afternoon" | "evening" | "night";
  day_type: "weekday" | "weekend";
  available_duration: number?;   // Minutes available to watch
  device_type: string;          // Current device
}

type IntentPrediction = {
  intent: UserIntent;
  confidence: float;             // Model confidence (0-1)
}

enum UserIntent {
  BROWSING,          // Exploring catalog
  BINGE_WATCHING,    // Ready for marathon
  QUICK_WATCH,       // Short content only
  DISCOVERY,         // Finding new content
  COMFORT_WATCH,     // Familiar content
  SPECIFIC_SEARCH    // Looking for something specific
}

// Validation Rules
- intent confidence must be between 0 and 1
- context_window max 100 interactions
- interaction_sequence max 500 tokens
```

### 6. CarouselRow
Represents a personalized carousel on the homepage.

```typescript
type CarouselRow = {
  id: string;                    // Format: "carousel:{uuid}"
  user_id: string;               // Target user
  
  // Display
  title: string;                 // Carousel title (may be LLM-generated)
  subtitle: string?;             // Optional subtitle
  carousel_type: CarouselType;  // Visual style
  
  // Content
  recommendation_ids: string[];  // Ordered list of recommendations
  total_items: number;          // Total available items
  
  // Ranking
  position: number;              // Vertical position on page
  relevance_score: float;        // Carousel relevance (0-1)
  
  // Metadata
  generated_at: timestamp;       // When carousel was created
  generation_reason: string;     // Why this carousel exists
  is_dynamic: boolean;          // Dynamically generated vs static
}

enum CarouselType {
  LARGE_TILES,       // Featured content
  STANDARD_TILES,    // Default view
  COMPACT_LIST,      // Space-efficient
  CONTINUE_WATCHING, // Resume tiles
  TOP_10,           // Ranked list
  EDITORIAL         // Curated content
}

// Validation Rules
- position must be positive integer
- relevance_score between 0 and 1
- recommendation_ids max 50 items
```

## Graph Relationships

### User-Content Relationships
```surrealql
// User interactions with content
RELATE user:123->watches->content:456 
  SET completion_rate = 0.95, 
      timestamp = time::now();

RELATE user:123->rates->content:456 
  SET rating = 8.5;

RELATE user:123->favorites->content:456;
```

### Content-Content Relationships
```surrealql
// Content similarity relationships
RELATE content:123->similar_to->content:456 
  SET similarity_score = 0.89,
      similarity_type = "genre_based";

// Franchise relationships  
RELATE content:123->sequel_of->content:456;
RELATE content:123->prequel_of->content:456;
RELATE content:123->related_to->content:456;
```

### User-User Relationships
```surrealql
// User similarity for collaborative filtering
RELATE user:123->similar_taste->user:456 
  SET similarity_score = 0.76,
      common_titles = 15;
```

## Indexes and Performance

### Required Indexes
```sql
-- User lookups
DEFINE INDEX user_email ON TABLE UserProfile COLUMNS email UNIQUE;
DEFINE INDEX user_username ON TABLE UserProfile COLUMNS username UNIQUE;

-- Content queries
DEFINE INDEX content_mal_id ON TABLE ContentItem COLUMNS mal_id UNIQUE;
DEFINE INDEX content_genres ON TABLE ContentItem COLUMNS genres;
DEFINE INDEX content_demographics ON TABLE ContentItem COLUMNS demographics;
DEFINE INDEX content_score ON TABLE ContentItem COLUMNS score;

-- Interaction queries
DEFINE INDEX interaction_user_time ON TABLE UserInteraction 
  COLUMNS user_id, timestamp;
DEFINE INDEX interaction_session ON TABLE UserInteraction 
  COLUMNS session_id, timestamp;

-- Recommendation serving
DEFINE INDEX rec_user_carousel ON TABLE Recommendation 
  COLUMNS user_id, carousel_id, position;
```

## Data Consistency Rules

### Cascading Deletes
- When UserProfile deleted → delete all related Interactions, Recommendations, Sessions
- When ContentItem deleted → delete all related Interactions, Recommendations

### Update Triggers
- When UserInteraction created → update UserProfile.preference_vector async
- When Rating changed → recalculate ContentItem.score
- When SessionContext expires → archive to cold storage

### Validation Constraints
- UserProfile.email must be unique across system
- ContentItem.mal_id must be unique and positive
- All embedding vectors must maintain consistent dimensions
- Timestamps must be in UTC

## Migration and Versioning

### Schema Versioning
- Current version: 1.0.0
- Migrations tracked in `/migrations` directory
- Backward compatibility maintained for 2 major versions

### Data Import
- Initial content load from anime-offline-database.json
- User data import from CSV with validation
- Batch import APIs with transaction support

## Privacy and Compliance

### PII Fields
- UserProfile.email (encrypted at rest)
- UserProfile.username (indexed but not exposed)
- UserInteraction.session_id (anonymized after 30 days)

### Data Retention
- UserProfile: Indefinite (user-requested deletion supported)
- UserInteraction: Full data for 2 years, aggregated thereafter
- SessionContext: 30 days then archived
- Recommendations: 90 days then purged

### GDPR Support
- User data export endpoint planned
- Right to deletion implemented
- Consent tracking in UserProfile
- Audit log for all data access