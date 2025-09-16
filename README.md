# Project Kenshō POC

An all-Rust anime streaming platform POC that aggregates metadata and integrates with Crunchyroll for authentication and streaming.

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ (`rustup update stable`)
- Docker & Docker Compose
- Node.js 18+ (for wasm-opt)

### Setup

1. **Start infrastructure**:
```bash
docker-compose up -d
```

2. **Install dependencies**:
```bash
# Backend
cd backend
cargo build

# Frontend (in another terminal)
cd frontend
cargo install trunk
rustup target add wasm32-unknown-unknown
trunk build
```

3. **Run the application**:
```bash
# Backend (terminal 1)
cd backend
cargo run --bin backend-server

# Frontend (terminal 2)
cd frontend
trunk serve --open
```

Application will be available at `http://localhost:8080`

## 📁 Project Structure

```
kensho/
├── backend/           # Rust backend with Axum
│   ├── src/
│   │   ├── models/    # Data models
│   │   ├── services/  # Business logic
│   │   ├── api/       # HTTP handlers
│   │   └── cli/       # CLI tools
│   └── tests/         # Backend tests
├── frontend/          # Rust frontend with Dioxus
│   ├── src/
│   │   ├── components/  # UI components
│   │   ├── pages/       # Route pages
│   │   └── services/    # API client
│   └── tests/           # Frontend tests
├── specs/             # Specifications and plans
└── docker-compose.yml # Infrastructure setup
```

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Backend tests only
cd backend && cargo test

# Frontend tests
cd frontend && wasm-pack test --headless --chrome

# Linting
cargo clippy --all-targets --all-features
```

## 🔧 Development

### Environment Variables
Copy `.env.example` to `.env` in both `backend/` and `frontend/` directories and update values as needed.

### Database Access
```bash
# Connect to SurrealDB
docker exec -it kensho-surrealdb surreal sql --conn http://localhost:8000 --ns kensho --db poc

# Connect to Redis
docker exec -it kensho-redis redis-cli -a kensho_redis_pass
```

## 📚 Documentation

- [Implementation Plan](specs/001-project-kensh-poc/plan.md)
- [Task List](specs/001-project-kensh-poc/tasks.md)
- [API Specification](specs/001-project-kensh-poc/contracts/openapi.yaml)
- [Quick Start Guide](specs/001-project-kensh-poc/quickstart.md)

## 🎯 Current Status

**Phase 3.1: Setup & Infrastructure** ✅ Complete
- [x] T001: Project structure created
- [x] T002: Backend initialized with dependencies
- [x] T003: Frontend initialized with dependencies
- [x] T004: Docker Compose configured
- [x] T005: GitHub Actions CI/CD configured
- [x] T006: Environment files created
- [x] T007: Rust formatting configured

**Next**: Phase 3.2 - Test-First Development (T008-T022)

## 📄 License

This is a POC project for demonstration purposes.