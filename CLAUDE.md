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

## Recent Changes
1. Added POC specification and implementation plan
2. Defined data model with graph relationships
3. Created OpenAPI contract specification

## Active Branch
`001-project-kensh-poc`

---
*Last updated: September 13, 2025*