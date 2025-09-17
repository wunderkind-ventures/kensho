# Claude Code Context: Project Kenshō

## Project Overview
Building an all-Rust anime streaming POC that aggregates metadata and integrates with Crunchyroll for authentication and streaming.

## Tech Stack
- **Backend**: Rust, Tokio, Axum
- **Frontend**: Rust, Dioxus (WASM)
- **Database**: SurrealDB (graph database)
- **Cache**: Redis
- **Integration**: crunchyroll-rs

## Project Structure
```
backend/
├── src/
│   ├── models/     # Data structures
│   ├── services/   # Business logic
│   └── api/        # HTTP handlers
frontend/
├── src/
│   ├── components/ # UI components
│   ├── pages/      # Route pages
│   └── services/   # API client
specs/001-project-kensh-poc/
├── plan.md         # Implementation plan
├── data-model.md   # Entity definitions
├── contracts/      # API specifications
└── quickstart.md   # Setup guide
```

## Common Commands
```bash
# Development
cargo run --bin backend-server
cd frontend && trunk serve

# Testing
cargo test --workspace
cargo clippy --all-targets
npm run test:load    # k6 load tests
npm run test:all     # All k6 tests

# Database
docker-compose up -d surrealdb redis
```

## Key Patterns

### Axum Handler
```rust
async fn get_anime(
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<Json<Anime>, AppError> {
    let anime = db.select(("anime", id)).await?;
    Ok(Json(anime))
}
```

### SurrealDB Query
```rust
let results: Vec<Anime> = db
    .query("SELECT * FROM anime WHERE title @@ $1")
    .bind(search_term)
    .await?;
```

### Dioxus Component
```rust
#[component]
fn AnimeCard(anime: Anime) -> Element {
    rsx! {
        div { class: "anime-card",
            img { src: "{anime.poster_url}" }
            h3 { "{anime.title}" }
        }
    }
}
```

## Implementation Status

### ✅ Completed (Phase 3.1-3.6)
- **Backend Core (T001-T041)**: All models, services, and API endpoints
- **Middleware (T055-T060)**: Auth, CORS, logging, rate limiting, error handling  
- **Resilience (T061-T067)**: Connection pooling, retry logic, circuit breakers, health checks
- **JSON Validation**: Custom error handling for malformed requests
- **Frontend Components (T044-T048)**: All UI components created
- **Frontend Pages (T049-T052)**: Home, Login, Series, Browse pages
- **Frontend Services (T053-T054)**: API client and auth state management

### ✅ Test Coverage (94.4% Complete)
- **Contract Tests (T008-T015)**: 8/8 - API endpoint verification
- **Integration Tests (T016-T021)**: 6/6 - End-to-end scenarios
- **Unit Tests (T061-T062)**: 2/2 - Model validation & search algorithms
- **Load Tests (T067)**: 1/1 - k6 performance testing suite
- **E2E Tests (T022)**: 0/1 - Frontend integration pending

### 🚧 In Progress
- **E2E Frontend Test (T022)**: Complete user journey test

### ⚠️ Known Issues
- **Frontend Compilation**: Dioxus 0.4 API compatibility issues
- **Database**: Using in-memory implementation (SurrealDB planned)
- **Redis**: Required but resilient fallbacks implemented

## Current Issues & Solutions

| Issue | Status | Solution |
|-------|--------|----------|
| Redis dependency failures | ✅ Fixed | Resilient client wrapper with fallbacks |
| Frontend not viewable | ✅ Fixed | Basic UI running at localhost:8080 |
| Dioxus 0.4 compatibility | 🚧 In Progress | Using simplified main.rs, full version needs fixes |
| No sample data loaded | ⏳ Pending | data/anime-offline-database.json available |
| Backend requires Redis | ✅ Fixed | Health checks allow degraded mode |

## Recent Changes (Sept 17, 2025)
1. **Test Implementation Complete**: 94.4% test coverage achieved
2. **Contract Tests**: All 8 API endpoints validated against OpenAPI spec
3. **Integration Tests**: 6 end-to-end scenarios implemented  
4. **Unit Tests**: Comprehensive model validation and search algorithm tests
5. **Load Tests**: k6 performance testing suite with 4 scenarios
6. **Documentation**: Updated TEST_SUMMARY.md tracking all test progress

## Active Branch
`001-project-kensh-poc`

## Quick Status Check
```bash
# Frontend (should show Dioxus app)
curl http://localhost:8080

# Backend health
curl http://localhost:3000/api/health/ready

# Component health
curl http://localhost:3000/api/health/components

# Run tests
cargo test --workspace     # All Rust tests
npm run test:load          # k6 load tests
```

## Next Steps (From Implementation Roadmap)
1. **Complete E2E Test (T022)**: Final test for frontend integration
2. **Phase 1: Foundation**: Service resilience layer, health checks
3. **Phase 2: Caching**: Multi-layer cache strategy
4. **Phase 3: Observability**: Metrics and distributed tracing
5. **Phase 4: API Enhancement**: OpenAPI docs, field selection
6. **Phase 5: Validation**: Load testing and chaos engineering

---
*Last updated: September 17, 2025*