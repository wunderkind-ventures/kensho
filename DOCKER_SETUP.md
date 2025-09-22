# Docker Setup for Project Kenshō

## Overview
This project uses Docker to containerize both the backend (Rust/Axum) and frontend (Dioxus) applications, along with supporting services (SurrealDB and Redis).

## Services

### 1. SurrealDB (Port 8000)
- Graph database for anime metadata
- Persistent storage via Docker volume
- Auth: root/root

### 2. Redis (Port 6379)
- Caching layer for API responses
- Session storage
- Password: kensho_redis_pass

### 3. Backend (Port 3000)
- Rust/Axum API server
- Connects to SurrealDB and Redis
- Two Dockerfile options available

### 4. Frontend (Port 8080)
- Dioxus WASM application
- Served via nginx
- Reverse proxy to backend API

## Building and Running

### Quick Start
```bash
# Build and run all services
./docker-start.sh

# Or manually:
docker-compose up --build
```

### Individual Services
```bash
# Build specific service
docker-compose build backend
docker-compose build frontend

# Run without building
docker-compose up
```

## Dockerfile Options

### Backend
1. **Dockerfile** - Production build with Google Distroless
   - Multi-stage build
   - Static linking with musl
   - Minimal attack surface
   - Runs as non-root user

2. **Dockerfile.simple** - Development build with Alpine
   - Single-stage build
   - Faster builds, larger image
   - Better for debugging

To use the simple version:
```bash
docker build -f backend/Dockerfile.simple -t kensho-backend backend/
```

### Frontend
- Alpine-based build stage
- nginx:alpine for serving
- Automatic API proxy configuration

## Troubleshooting

### Build Issues
If you encounter network timeouts during cargo builds:
1. The Dockerfiles include retry settings (`CARGO_NET_RETRY=10`)
2. Use the simple Dockerfile for more reliable builds
3. Build with host network: `docker build --network=host ...`

### Package Compatibility
The project has been updated to use compatible package versions:
- Rust 1.81
- SurrealDB 2.0
- Redis 0.25
- Tokio 1.40

## Environment Variables

The backend expects these environment variables (set in docker-compose.yml):
- `DATABASE_URL`: http://surrealdb:8000
- `REDIS_URL`: redis://:kensho_redis_pass@redis:6379
- `JWT_SECRET`: your-secret-key-change-in-production
- `CRUNCHYROLL_EMAIL`: (optional)
- `CRUNCHYROLL_PASSWORD`: (optional)

## Data Loading

To load anime data into SurrealDB:
1. Use the MCP integration (if available)
2. Run import scripts: `cargo run --bin import-surreal-http`
3. Direct SQL via SurrealDB UI at http://localhost:8000

## Access Points

- **Frontend**: http://localhost:8080
- **Backend API**: http://localhost:3000
- **SurrealDB Admin**: http://localhost:8000
- **Redis**: localhost:6379

## Docker Commands

```bash
# View logs
docker-compose logs -f [service-name]

# Stop all services
docker-compose down

# Stop and remove volumes (clean slate)
docker-compose down -v

# Exec into container
docker exec -it kensho-backend sh
docker exec -it kensho-surrealdb surreal sql --conn http://localhost:8000
```

## Notes

- The frontend nginx configuration automatically proxies `/api` requests to the backend
- Both Dockerfiles use Alpine Linux for better Apple Silicon compatibility
- The backend uses static linking for compatibility with distroless images
- Data volumes persist between container restarts unless explicitly removed