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

### 7. Experiment
Represents an A/B test or multi-variant experiment.

```typescript
type Experiment = {
  id: string;                    // Format: "experiment:{uuid}"
  name: string;                  // Human-readable experiment name
  description: string;           // Detailed experiment hypothesis
  status: ExperimentStatus;      // Current experiment state
  
  // Targeting
  target_percentage: float;       // % of users to include (0-100)
  target_segments: string[];      // User segments to target
  exclusion_experiments: string[]; // Mutually exclusive experiments
  
  // Variants
  variants: ExperimentVariant[];  // Test variants including control
  
  // Metrics
  primary_metric: string;         // Main success metric (e.g., "ctr")
  secondary_metrics: string[];    // Additional metrics to track
  minimum_sample_size: number;    // Required samples per variant
  
  // Timeline
  created_at: timestamp;         // Experiment creation time
  started_at: timestamp?;        // When experiment went live
  ended_at: timestamp?;          // When experiment concluded
  
  // Results
  winner_variant_id: string?;    // Winning variant if concluded
  statistical_significance: float?; // p-value of results
}

enum ExperimentStatus {
  DRAFT,           // Being configured
  SCHEDULED,       // Waiting to start
  RUNNING,         // Active experiment
  PAUSED,          // Temporarily stopped
  COMPLETED,       // Reached conclusion
  CANCELLED        // Terminated early
}

type ExperimentVariant = {
  id: string;                    // Format: "variant:{uuid}"
  name: string;                  // e.g., "control", "treatment_a"
  allocation_percentage: float;   // Traffic % for this variant
  
  // Configuration overrides
  config: VariantConfig;
}

type VariantConfig = {
  // Carousel concept selection
  carousel_concepts?: CarouselConcept[];  // Which carousel types to show
  concept_selection_strategy?: string;     // How to pick concepts
  
  // Carousel ranking
  carousel_ranking_algorithm?: string;     // Algorithm for ordering
  carousel_ranking_weights?: Map<string, float>; // Feature weights
  
  // Title generation
  title_generation_mode?: "static" | "template" | "llm";
  title_template?: string;                 // Template with variables
  title_llm_prompt?: string;              // LLM prompt for titles
  
  // Content selection
  content_selection_algorithm?: string;    // Which recommender to use
  diversity_injection_rate?: float;        // Override diversity %
  
  // Visual presentation
  default_carousel_type?: CarouselType;    // Display style
  artwork_selection_strategy?: string;     // How to pick images
  
  // Behavioral modifications
  prefetch_enabled?: boolean;              // Preload next content
  autoplay_enabled?: boolean;              // Auto-advance carousels
  animation_style?: string;                // Transition effects
}

enum CarouselConcept {
  CONTINUE_WATCHING,    // Resume content
  FOR_YOU,             // Personalized picks
  TRENDING_NOW,        // Popular content
  NEW_RELEASES,        // Fresh content
  BECAUSE_YOU_WATCHED, // Based on specific show
  GENRE_COLLECTION,    // Genre-focused
  MOOD_BASED,         // Emotional themes
  TIME_BASED,         // Morning/Evening picks
  SOCIAL_PROOF,       // Friends watching
  HIDDEN_GEMS,        // Underrated content
  AWARD_WINNERS,      // Critically acclaimed
  BINGE_WORTHY        // Complete series
}

// Validation Rules
- allocation_percentage sum must equal 100
- minimum_sample_size must be >= 100
- target_percentage between 0 and 100
```

### 8. ExperimentAssignment
Tracks which users are in which experiments.

```typescript
type ExperimentAssignment = {
  id: string;                    // Format: "assignment:{uuid}"
  user_id: string;               // User in experiment
  experiment_id: string;         // Active experiment
  variant_id: string;            // Assigned variant
  
  // Assignment details
  assigned_at: timestamp;        // When user was assigned
  assignment_reason: string;     // Why user qualified
  hash_bucket: number;          // Stable hash for assignment
  
  // Tracking
  first_exposure: timestamp?;    // First time variant shown
  last_exposure: timestamp?;     // Most recent exposure
  exposure_count: number;        // Times variant shown
}

// Validation Rules
- hash_bucket between 0 and 9999 (for stable assignment)
- One assignment per user per experiment
```

### 9. ExperimentMetrics
Aggregated metrics for experiment analysis.

```typescript
type ExperimentMetrics = {
  id: string;                    // Format: "metrics:{experiment_id}:{variant_id}"
  experiment_id: string;         // Parent experiment
  variant_id: string;            // Variant being measured
  
  // Core metrics
  impressions: number;           // Times shown
  unique_users: number;          // Distinct users
  
  // Engagement metrics
  clicks: number;                // Total clicks
  ctr: float;                   // Click-through rate
  avg_watch_time: float;        // Average viewing duration
  completion_rate: float;       // % who finish episodes
  
  // Carousel-specific metrics
  carousel_engagement_rate: float;     // % who interact with carousel
  avg_scroll_depth: float;             // How far users scroll
  avg_hover_time: float;               // Time spent browsing
  carousel_abandonment_rate: float;    // % who leave from carousel
  
  // Advanced metrics
  diversity_score: float;              // Content variety consumed
  discovery_rate: float;               // % new content discovered
  session_duration: float;             // Total time in session
  return_rate_24h: float;             // % who return within 24h
  
  // Statistical metrics
  sample_size: number;                 // Users in this variant
  confidence_interval: [float, float]; // 95% CI for primary metric
  p_value: float?;                    // Statistical significance
  
  // Time series
  hourly_metrics: Map<timestamp, MetricSnapshot>;
  
  // Updated
  last_calculated: timestamp;
}

type MetricSnapshot = {
  impressions: number;
  clicks: number;
  ctr: float;
  unique_users: number;
}

// Validation Rules
- All rates must be between 0 and 1
- p_value between 0 and 1
- sample_size must be positive
```

### 10. MultiArmedBandit
Configuration for dynamic optimization.

```typescript
type MultiArmedBandit = {
  id: string;                    // Format: "mab:{feature}"
  feature: string;               // What's being optimized
  algorithm: MABAlgorithm;       // Selection algorithm
  
  // Arms (options)
  arms: BanditArm[];            // Available choices
  
  // Parameters
  exploration_rate: float;       // Epsilon for epsilon-greedy
  temperature: float;           // For softmax selection
  window_size: number;          // Recent events to consider
  
  // State
  total_pulls: number;          // Total selections made
  last_update: timestamp;       // Last model update
}

type BanditArm = {
  id: string;                   // Arm identifier
  name: string;                 // Human-readable name
  
  // Statistics
  pulls: number;                // Times selected
  rewards: float;               // Total reward earned
  avg_reward: float;           // Average reward
  
  // Confidence
  ucb_score: float?;           // Upper confidence bound
  thompson_sample: float?;     // Thompson sampling score
}

enum MABAlgorithm {
  EPSILON_GREEDY,    // Random exploration
  UCB1,             // Upper confidence bound
  THOMPSON_SAMPLING, // Bayesian approach
  EXP3              // Adversarial bandit
}

// Validation Rules
- exploration_rate between 0 and 1
- temperature > 0
- window_size > 0
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