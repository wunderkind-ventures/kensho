# Quickstart Guide: Project Kenshō POC

**Version**: 0.1.0
**Date**: September 13, 2025

## Prerequisites

Before starting, ensure you have:
- Rust 1.75+ installed (`rustup update stable`)
- Docker and Docker Compose installed
- Node.js 18+ (for frontend tooling)
- Git configured

## Quick Setup

### 1. Clone and Initialize

```bash
# Clone the repository
git clone https://github.com/your-org/kensho.git
cd kensho

# Checkout the feature branch
git checkout 001-project-kensh-poc

# Install dependencies
cargo build --all
cd frontend && npm install && cd ..
```

### 2. Start Infrastructure

```bash
# Start SurrealDB and Redis
docker-compose up -d surrealdb redis

# Wait for services to be ready
sleep 5

# Initialize database schema
cargo run --bin db-init
```

### 3. Import Anime Data

```bash
# Download anime-offline-database
curl -L https://github.com/manami-project/anime-offline-database/raw/master/anime-offline-database.json \
  -o data/anime-offline-database.json

# Run the ingestion script
cargo run --bin ingest-metadata -- \
  --source data/anime-offline-database.json \
  --enrich-imdb
```

### 4. Start the Application

```bash
# Terminal 1: Start backend server
cargo run --bin backend-server

# Terminal 2: Start frontend dev server
cd frontend
trunk serve --open
```

The application will be available at `http://localhost:8080`

## Validation Walkthrough

This section validates all acceptance criteria from the specification.

### Test 1: Search and Discovery

1. **Open the application** at `http://localhost:8080`
2. **Search for "SPY x FAMILY"** in the search bar
3. **Verify**: Search results show SPY x FAMILY with:
   - Poster image
   - Title
   - IMDb rating (if available)
4. **Click on the series** to navigate to IP Hub

✅ Validates: FR-002, FR-003

### Test 2: IP Hub Display

On the SPY x FAMILY IP Hub page, verify:

1. **Series metadata** is displayed:
   - Title and poster
   - Synopsis
   - Episode count
   - IMDb rating and votes
2. **Episode list** shows all available episodes
3. **Disabled tabs** for Manga, Community, Store are visible but greyed out

✅ Validates: FR-003, FR-010, FR-011

### Test 3: Authentication Flow

1. **Click "Play"** on Episode 1 (while not logged in)
2. **Verify**: Redirected to login page
3. **Enter invalid credentials**:
   - Username: `test@example.com`
   - Password: `wrongpassword`
4. **Verify**: Error message "Invalid username or password"
5. **Enter valid Crunchyroll credentials**
6. **Verify**: Successfully logged in and redirected back

✅ Validates: FR-004, FR-007, Auth error handling

### Test 4: Video Streaming

After successful login:

1. **Click "Play"** on Episode 1 again
2. **Verify**: Video player loads
3. **Measure**: Time from click to first frame (should be <4 seconds)
4. **Test player controls**:
   - Click pause/play
   - Adjust volume slider
   - Click fullscreen button
5. **Verify**: All controls work as expected

✅ Validates: FR-008, FR-009, NFR-1.3

### Test 5: Session Management

1. **Navigate** to different pages while logged in
2. **Verify**: Session persists (no re-login required)
3. **Click "Logout"** button
4. **Verify**: Session terminated, redirected to home
5. **Try to play** an episode
6. **Verify**: Prompted to login again

✅ Validates: FR-005, FR-006

### Test 6: Performance Validation

Run performance tests:

```bash
# API Response Time Test
cargo test --test performance -- --nocapture

# Frontend Load Test
npm run test:performance

# Load Testing with k6
k6 run tests/load/api-stress.js
```

Expected results:
- API P95 response time: <200ms ✅
- First Contentful Paint: <2s ✅
- Time to First Frame: <4s ✅

✅ Validates: NFR-1.1, NFR-1.2, NFR-1.3

### Test 7: Browser Compatibility

Test the application in:
1. **Google Chrome** (latest stable)
2. **Mozilla Firefox** (latest stable)
3. **Apple Safari** (latest stable)

Verify full functionality in each browser.

✅ Validates: NFR-3.1

### Test 8: Seasonal Browse

1. **Navigate** to Browse page
2. **Select** Year: 2024, Season: Fall
3. **Verify**: Grid of anime from Fall 2024 displayed
4. **Check**: Results sorted by IMDb rating (highest first)

✅ Validates: FR-006

## Development Commands

### Backend Development

```bash
# Run backend tests
cargo test --workspace

# Run specific test suite
cargo test --test integration

# Check code quality
cargo clippy --all-targets
cargo fmt --check

# Build release version
cargo build --release
```

### Frontend Development

```bash
# Start dev server with hot reload
trunk serve

# Run frontend tests
cargo test -p frontend

# Build production WASM
trunk build --release

# Analyze bundle size
trunk build --release -- --features analyze
```

### Database Management

```bash
# Connect to SurrealDB CLI
docker exec -it kensho-surrealdb surreal sql --conn http://localhost:8000 --ns kensho --db poc

# Backup database
docker exec kensho-surrealdb surreal export --ns kensho --db poc > backup.sql

# Restore database
docker exec -i kensho-surrealdb surreal import --ns kensho --db poc < backup.sql
```

## Troubleshooting

### Issue: Cannot connect to SurrealDB

```bash
# Check if container is running
docker ps | grep surrealdb

# View logs
docker logs kensho-surrealdb

# Restart container
docker-compose restart surrealdb
```

### Issue: WASM build fails

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Install trunk
cargo install trunk

# Clear cache and rebuild
trunk clean
trunk build
```

### Issue: Crunchyroll authentication fails

1. Verify credentials are correct
2. Check for regional restrictions
3. Ensure crunchyroll-rs is up to date:
   ```bash
   cargo update -p crunchyroll-rs
   ```

### Issue: Video playback issues

1. Check browser console for errors
2. Verify HLS.js loaded correctly
3. Test stream URL directly:
   ```bash
   curl -I $(curl http://localhost:8080/api/stream/anime-id/1 -H "Authorization: Bearer $TOKEN" | jq -r .stream_url)
   ```

## Environment Variables

Create a `.env` file in the project root:

```env
# Backend Configuration
RUST_LOG=info
DATABASE_URL=ws://localhost:8000
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key-here
CORS_ORIGIN=http://localhost:8080

# Frontend Configuration
API_BASE_URL=http://localhost:8080/api

# Optional: Crunchyroll API (for testing)
CR_USERNAME=your-username
CR_PASSWORD=your-password
```

## Success Criteria Checklist

Run through this checklist to confirm POC success:

- [ ] User can search for "SPY x FAMILY"
- [ ] Search results lead to correct IP Hub
- [ ] IP Hub displays poster, synopsis, IMDb rating
- [ ] Anonymous user prompted to login when clicking play
- [ ] User can login with Crunchyroll credentials
- [ ] Authenticated user can play Episode 1
- [ ] Video starts within 4 seconds
- [ ] Player controls (pause, volume, fullscreen) work
- [ ] All tests pass in Chrome, Firefox, Safari
- [ ] API responses under 200ms (P95)
- [ ] Page loads under 2 seconds

## Next Steps

After successful validation:

1. **Document findings** in `validation-report.md`
2. **Gather performance metrics** for production planning
3. **Review security audit** results
4. **Plan Phase 1 features** based on POC learnings

## Support

For issues or questions:
- Check the [troubleshooting](#troubleshooting) section
- Review logs in `logs/` directory
- Open an issue in the project repository