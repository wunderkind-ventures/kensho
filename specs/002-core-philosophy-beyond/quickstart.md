# Quick Start Guide: Personalized Recommendation System

**Branch**: `002-core-philosophy-beyond` | **Date**: 2025-01-18

## Overview
This guide demonstrates the core functionality of the personalized recommendation system through a complete user journey from onboarding to receiving personalized recommendations.

## Prerequisites

### Environment Setup
```bash
# Clone repository and checkout branch
git clone https://github.com/kensho/anime-streaming.git
cd anime-streaming
git checkout 002-core-philosophy-beyond

# Install dependencies
cd backend && cargo build --release
cd ../ml-services && pip install -r requirements.txt
cd ../frontend && trunk build --release

# Start infrastructure
docker-compose up -d surrealdb redis

# Initialize database
./scripts/init-db.sh
./scripts/load-content-data.sh data/anime-offline-database.json
```

### Configuration
Create `.env` file in project root:
```env
DATABASE_URL=ws://localhost:8000/rpc
REDIS_URL=redis://localhost:6379
MODEL_PATH=./models/foundation-v1
API_PORT=3000
FRONTEND_PORT=8080
LOG_LEVEL=info
```

## User Journey Walkthrough

### Step 1: New User Onboarding

Create a new user and complete the onboarding survey:

```bash
# Create user account
curl -X POST http://localhost:3000/api/v1/user/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "anime_fan_123",
    "email": "fan@example.com",
    "password": "secure_password"
  }'

# Response
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}

# Complete onboarding survey
curl -X POST http://localhost:3000/api/v1/user/onboarding \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "favorite_titles": ["1", "20", "1535"],  # Death Note, Naruto, Death Parade
    "preferred_genres": ["Psychological", "Action", "Supernatural"],
    "preferred_audio": "sub"
  }'

# Response
{
  "profile_initialized": true,
  "initial_recommendations": [...]
}
```

### Step 2: Get Personalized Homepage

Fetch personalized carousel rows for the homepage:

```bash
curl -X GET "http://localhost:3000/api/v1/recommendations/home?user_id=550e8400-e29b-41d4-a716-446655440000&limit=5" \
  -H "Authorization: Bearer $TOKEN"

# Response
{
  "carousels": [
    {
      "id": "carousel_001",
      "title": "Because You Loved Death Note",
      "subtitle": "Dark psychological thrillers",
      "carousel_type": "STANDARD_TILES",
      "position": 1,
      "recommendations": [
        {
          "content": {
            "title": "Code Geass",
            "mal_id": 1575,
            "genres": ["Mecha", "Drama", "Military"]
          },
          "relevance_score": 0.92,
          "reason": "Strategic mind games and moral ambiguity"
        }
      ]
    },
    {
      "id": "carousel_002", 
      "title": "Trending This Week",
      "carousel_type": "LARGE_TILES",
      "position": 2,
      "recommendations": [...]
    }
  ],
  "generated_at": "2025-01-18T10:30:00Z",
  "model_version": "foundation-v1.0"
}
```

### Step 3: Track User Interactions

Track viewing behavior to improve recommendations:

```bash
# Start watching an episode
curl -X POST http://localhost:3000/api/v1/interactions/track \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "content_id": "content:1575",
    "interaction_type": "WATCH_START",
    "timestamp": "2025-01-18T10:35:00Z",
    "metadata": {
      "episode_number": 1,
      "device_type": "web"
    }
  }'

# Complete watching (after 23 minutes)
curl -X POST http://localhost:3000/api/v1/interactions/track \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "content_id": "content:1575",
    "interaction_type": "WATCH_COMPLETE",
    "timestamp": "2025-01-18T10:58:00Z",
    "metadata": {
      "episode_number": 1,
      "watch_duration": 1380,
      "completion_rate": 0.95
    }
  }'
```

### Step 4: Get Next Episode Recommendations

After watching an episode, get personalized "up next" suggestions:

```bash
curl -X POST http://localhost:3000/api/v1/recommendations/next \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "content_id": "content:1575",
    "episode_number": 1,
    "completion_rate": 0.95
  }'

# Response
{
  "recommendations": [
    {
      "content": {
        "title": "Code Geass",
        "episode_title": "The Day a New Demon was Born"
      },
      "recommendation_type": "CONTINUE_WATCHING",
      "reason": "Next episode"
    },
    {
      "content": {
        "title": "Steins;Gate",
        "mal_id": 9253
      },
      "recommendation_type": "CONTENT_BASED",
      "reason": "Complex plot with strategic elements"
    }
  ],
  "continue_next_episode": true,
  "next_episode_number": 2
}
```

### Step 5: Personalized Search

Search for content with personalized ranking:

```bash
curl -X POST http://localhost:3000/api/v1/recommendations/search \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "query": "fantasy",
    "filters": {
      "media_type": "TV",
      "year_range": {"min": 2020, "max": 2025}
    },
    "limit": 10
  }'

# Response shows fantasy anime ranked by user's preference for dark themes
{
  "results": [
    {
      "content": {
        "title": "Made in Abyss: The Golden City of the Scorching Sun",
        "mal_id": 48926
      },
      "relevance_score": 0.88,
      "personalization_score": 0.91,
      "match_reason": "Dark fantasy matching your psychological preferences"
    }
  ],
  "total_count": 47,
  "personalization_applied": true
}
```

### Step 6: Update User Preferences

Adjust preferences based on viewing patterns:

```bash
curl -X PUT http://localhost:3000/api/v1/user/profile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "disliked_genres": ["Romance", "Sports"],
    "content_maturity": "mature"
  }'
```

## Testing the Recommendation Engine

### 1. Test Content-Based Filtering
```bash
# Run content similarity test
cargo test --package recommendation-engine test_content_similarity

# Expected: Similar anime based on genre/theme overlap
```

### 2. Test Collaborative Filtering
```bash
# Run collaborative filtering test
cargo test --package recommendation-engine test_collaborative_filtering

# Expected: Recommendations based on similar users
```

### 3. Test Session Intent Prediction
```bash
# Run intent prediction test
python ml-services/tests/test_intent_prediction.py

# Expected: Correct intent classification (browsing vs binge-watching)
```

### 4. Test Diversity Injection
```bash
# Verify 16.7% diverse content
curl -X GET "http://localhost:3000/api/v1/recommendations/home?user_id=$USER_ID" \
  | jq '.carousels[].recommendations[] | select(.recommendation_type == "DIVERSE")' \
  | jq -s 'length'

# Expected: ~17% of recommendations are diverse
```

## Performance Validation

### 1. Latency Test
```bash
# Measure recommendation latency
ab -n 100 -c 10 -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/v1/recommendations/home?user_id=$USER_ID"

# Expected: p95 < 500ms
```

### 2. Load Test
```bash
# Run k6 load test
k6 run tests/load/recommendation-load-test.js

# Expected: 100+ RPS with <500ms latency
```

## Monitoring

### Check System Health
```bash
# Overall health
curl http://localhost:3000/api/health/ready

# Component health
curl http://localhost:3000/api/health/components

# Response
{
  "database": "healthy",
  "redis": "healthy",
  "model_server": "healthy",
  "status": "operational"
}
```

### View Metrics
```bash
# Prometheus metrics
curl http://localhost:3000/metrics

# Key metrics to monitor:
# - recommendation_latency_seconds
# - recommendation_cache_hit_ratio
# - model_inference_duration_seconds
```

## Troubleshooting

### Common Issues

1. **High Latency on First Request**
   - Cause: Model cold start
   - Solution: Implement model preloading on startup

2. **Empty Recommendations**
   - Cause: Insufficient user data
   - Solution: Ensure onboarding completed and fallback to popular content

3. **Database Connection Errors**
   - Cause: SurrealDB not running
   - Solution: `docker-compose up -d surrealdb`

4. **Model Loading Failure**
   - Cause: Missing model files
   - Solution: Run `./scripts/download-models.sh`

### Debug Mode
Enable detailed logging:
```bash
LOG_LEVEL=debug cargo run --bin backend-server
```

## Next Steps

1. **Add More Content**: Import additional anime from MAL API
2. **Train Custom Model**: Use collected interaction data for fine-tuning
3. **A/B Testing**: Set up experiments for recommendation algorithms
4. **Scale Testing**: Increase concurrent users gradually
5. **UI Integration**: Connect frontend to recommendation API

## Support

- Documentation: `/docs/recommendation-system.md`
- API Reference: `/contracts/recommendation-api.yaml`
- Data Model: `/specs/002-core-philosophy-beyond/data-model.md`