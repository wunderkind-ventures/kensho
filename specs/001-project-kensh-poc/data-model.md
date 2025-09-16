# Data Model: Project Kenshō POC

**Feature**: Project Kenshō POC - All-Rust Anime Streaming Platform
**Date**: September 13, 2025
**Status**: Complete

## Overview
This document defines the complete data model for the Kenshō POC, including entities, relationships, and validation rules derived from the functional requirements.

## Core Entities

### 1. Anime
Primary entity representing an anime series with enriched metadata.

```rust
struct Anime {
    id: String,              // Unique identifier (UUID)
    title: String,           // Primary title
    synonyms: Vec<String>,   // Alternative titles for search
    sources: Vec<String>,    // Data source URLs
    episodes: u32,           // Total episode count
    status: AnimeStatus,     // Airing status
    anime_type: AnimeType,   // TV, Movie, OVA, etc.
    anime_season: AnimeSeason, // Season and year
    synopsis: String,        // Description
    poster_url: String,      // Cover image URL
    imdb: Option<ImdbData>,  // External rating data
    created_at: DateTime,
    updated_at: DateTime,
}

enum AnimeStatus {
    Finished,
    Ongoing,
    Upcoming,
    Unknown,
}

enum AnimeType {
    TV,
    Movie,
    OVA,
    ONA,
    Special,
    Unknown,
}

struct AnimeSeason {
    season: Season,
    year: u16,
}

enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

struct ImdbData {
    id: String,        // IMDb ID (e.g., "tt1234567")
    rating: f32,       // Rating value (0.0-10.0)
    votes: u32,        // Number of votes
}
```

**Validation Rules**:
- `title`: Required, non-empty, max 500 chars
- `episodes`: Must be >= 0
- `anime_season.year`: Must be between 1900 and current year + 5
- `imdb.rating`: Must be between 0.0 and 10.0
- `poster_url`: Must be valid URL

### 2. Episode
Individual episode within an anime series.

```rust
struct Episode {
    id: String,              // Unique identifier
    anime_id: String,        // Foreign key to Anime
    episode_number: u32,     // Episode sequence number
    title: Option<String>,   // Episode title
    duration: Option<u32>,   // Duration in seconds
    air_date: Option<Date>,  // Original air date
    synopsis: Option<String>, // Episode description
    thumbnail_url: Option<String>, // Episode thumbnail
    created_at: DateTime,
    updated_at: DateTime,
}
```

**Validation Rules**:
- `anime_id`: Must reference existing Anime
- `episode_number`: Must be > 0 and unique within anime
- `duration`: If present, must be > 0

### 3. Tag
Categorization tags for anime (genres, themes, etc.)

```rust
struct Tag {
    id: String,           // Unique identifier
    name: String,         // Tag name (e.g., "Action")
    category: TagCategory, // Tag classification
    description: Option<String>,
    created_at: DateTime,
}

enum TagCategory {
    Genre,      // Action, Comedy, Drama
    Theme,      // School, Military, Supernatural
    Demographic, // Shounen, Seinen, Josei
    Content,    // Violence, Romance
}
```

**Validation Rules**:
- `name`: Required, unique, max 50 chars
- `category`: Required, must be valid enum value

### 4. Session
User authentication session for accessing streaming content.

```rust
struct Session {
    id: String,              // Session ID (JWT token ID)
    user_id: String,         // Crunchyroll user identifier
    cr_token_key: String,    // Redis key for encrypted CR token
    expires_at: DateTime,    // Session expiration
    refresh_token: String,   // Token for session refresh
    created_at: DateTime,
    last_accessed: DateTime,
}
```

**Validation Rules**:
- `expires_at`: Must be future timestamp
- `cr_token_key`: Must be valid Redis key format
- Session auto-expires after 15 minutes of inactivity

## Graph Relationships

### 1. has_tag
Links anime to their associated tags.

```rust
struct HasTag {
    in: String,   // anime.id
    out: String,  // tag.id
    relevance: f32, // Tag relevance score (0.0-1.0)
}
```

### 2. is_sequel_of
Represents sequel relationships between anime.

```rust
struct IsSequelOf {
    in: String,   // sequel anime.id
    out: String,  // original anime.id
    order: u32,   // Sequence order in series
}
```

### 3. is_prequel_of
Represents prequel relationships between anime.

```rust
struct IsPrequelOf {
    in: String,   // prequel anime.id
    out: String,  // sequel anime.id
    order: u32,   // Sequence order in timeline
}
```

### 4. is_related_to
General relationship between anime (side stories, alternate versions).

```rust
struct IsRelatedTo {
    in: String,   // anime.id
    out: String,  // related anime.id
    relation_type: RelationType,
}

enum RelationType {
    SideStory,
    AlternativeVersion,
    SpinOff,
    Adaptation,
    Other,
}
```

## Database Schema (SurrealDB)

```surql
-- Core Tables
DEFINE TABLE anime SCHEMAFULL;
DEFINE FIELD title ON anime TYPE string ASSERT $value != NONE;
DEFINE FIELD synonyms ON anime TYPE array;
DEFINE FIELD sources ON anime TYPE array;
DEFINE FIELD episodes ON anime TYPE number ASSERT $value >= 0;
DEFINE FIELD status ON anime TYPE string;
DEFINE FIELD anime_type ON anime TYPE string;
DEFINE FIELD anime_season ON anime TYPE object;
DEFINE FIELD anime_season.season ON anime TYPE string;
DEFINE FIELD anime_season.year ON anime TYPE number;
DEFINE FIELD synopsis ON anime TYPE string;
DEFINE FIELD poster_url ON anime TYPE string;
DEFINE FIELD imdb ON anime TYPE option<object>;
DEFINE FIELD created_at ON anime TYPE datetime VALUE $before OR time::now();
DEFINE FIELD updated_at ON anime TYPE datetime VALUE time::now();

DEFINE TABLE episode SCHEMAFULL;
DEFINE FIELD anime_id ON episode TYPE record(anime) ASSERT $value != NONE;
DEFINE FIELD episode_number ON episode TYPE number ASSERT $value > 0;
DEFINE FIELD title ON episode TYPE option<string>;
DEFINE FIELD duration ON episode TYPE option<number>;
DEFINE FIELD air_date ON episode TYPE option<datetime>;
DEFINE FIELD synopsis ON episode TYPE option<string>;
DEFINE FIELD thumbnail_url ON episode TYPE option<string>;
DEFINE FIELD created_at ON episode TYPE datetime VALUE $before OR time::now();
DEFINE FIELD updated_at ON episode TYPE datetime VALUE time::now();

DEFINE TABLE tag SCHEMAFULL;
DEFINE FIELD name ON tag TYPE string ASSERT $value != NONE;
DEFINE FIELD category ON tag TYPE string ASSERT $value IN ["Genre", "Theme", "Demographic", "Content"];
DEFINE FIELD description ON tag TYPE option<string>;
DEFINE FIELD created_at ON tag TYPE datetime VALUE $before OR time::now();

DEFINE TABLE session SCHEMAFULL;
DEFINE FIELD user_id ON session TYPE string ASSERT $value != NONE;
DEFINE FIELD cr_token_key ON session TYPE string ASSERT $value != NONE;
DEFINE FIELD expires_at ON session TYPE datetime ASSERT $value > time::now();
DEFINE FIELD refresh_token ON session TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON session TYPE datetime VALUE $before OR time::now();
DEFINE FIELD last_accessed ON session TYPE datetime VALUE time::now();

-- Relationship Tables
DEFINE TABLE has_tag SCHEMAFULL;
DEFINE FIELD in ON has_tag TYPE record(anime);
DEFINE FIELD out ON has_tag TYPE record(tag);
DEFINE FIELD relevance ON has_tag TYPE number ASSERT $value >= 0 AND $value <= 1;

DEFINE TABLE is_sequel_of SCHEMAFULL;
DEFINE FIELD in ON is_sequel_of TYPE record(anime);
DEFINE FIELD out ON is_sequel_of TYPE record(anime);
DEFINE FIELD order ON is_sequel_of TYPE number;

DEFINE TABLE is_prequel_of SCHEMAFULL;
DEFINE FIELD in ON is_prequel_of TYPE record(anime);
DEFINE FIELD out ON is_prequel_of TYPE record(anime);
DEFINE FIELD order ON is_prequel_of TYPE number;

DEFINE TABLE is_related_to SCHEMAFULL;
DEFINE FIELD in ON is_related_to TYPE record(anime);
DEFINE FIELD out ON is_related_to TYPE record(anime);
DEFINE FIELD relation_type ON is_related_to TYPE string;

-- Indexes for Performance
DEFINE INDEX anime_title_search ON anime COLUMNS title, synonyms SEARCH ANALYZER ascii FILTERS lowercase, synonyms;
DEFINE INDEX anime_season_idx ON anime COLUMNS anime_season.year, anime_season.season;
DEFINE INDEX episode_anime_idx ON episode COLUMNS anime_id, episode_number UNIQUE;
DEFINE INDEX tag_name_idx ON tag COLUMNS name UNIQUE;
DEFINE INDEX session_user_idx ON session COLUMNS user_id;
```

## Query Patterns

### 1. Search by Title or Synonym
```surql
SELECT * FROM anime 
WHERE title @@ $search_term 
OR synonyms CONTAINS $search_term
LIMIT 20;
```

### 2. Browse by Season
```surql
SELECT * FROM anime 
WHERE anime_season.year = $year 
AND anime_season.season = $season
ORDER BY imdb.rating DESC NULLS LAST;
```

### 3. Get Anime with All Relationships
```surql
SELECT *,
  ->has_tag->tag AS tags,
  ->is_sequel_of->anime AS sequels,
  ->is_prequel_of->anime AS prequels,
  ->is_related_to->anime AS related
FROM anime:$id;
```

### 4. Get Episodes for Anime
```surql
SELECT * FROM episode 
WHERE anime_id = anime:$id
ORDER BY episode_number ASC;
```

## State Transitions

### Session Lifecycle
```
Created -> Active -> Refreshed -> Expired
                 \-> Invalidated (logout)
```

### Anime Status Flow
```
Upcoming -> Ongoing -> Finished
```

## Data Constraints & Business Rules

1. **Unique Constraints**:
   - Anime title + year combination should be unique
   - Episode number within an anime must be unique
   - Tag names must be globally unique
   - Session tokens must be cryptographically unique

2. **Referential Integrity**:
   - Episodes cannot exist without parent anime
   - Relationships must reference existing anime
   - Tags must exist before association

3. **Performance Requirements**:
   - Search queries must return in <200ms
   - Season browse must handle 500+ results
   - Relationship traversal max depth: 3 levels

4. **Data Retention**:
   - Sessions expire after 15 minutes inactivity
   - Expired sessions cleaned up after 24 hours
   - Anime metadata retained indefinitely
   - No user data persisted beyond session

## Migration Strategy

Initial data load sequence:
1. Create all tag entities
2. Import anime metadata from anime-offline-database
3. Establish has_tag relationships
4. Import IMDb enrichment data
5. Create anime relationship edges
6. Generate episode records from CR API

## Next Steps
With the data model defined, proceed to generate API contracts that expose these entities through RESTful endpoints.