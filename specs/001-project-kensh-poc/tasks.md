# Tasks: Project Kenshō POC - All-Rust Anime Streaming Platform

**Input**: Design documents from `/specs/001-project-kensh-poc/`
**Prerequisites**: plan.md (✓), research.md (✓), data-model.md (✓), contracts/openapi.yaml (✓)

## Task Execution Guide

### How to Use This Document
Each task includes a 📖 reference pointing to the exact location in the implementation details where you can find:
- **Data structures** and field definitions (data-model.md)
- **API contracts** and request/response schemas (contracts/openapi.yaml)
- **Technical decisions** and implementation strategies (research.md)
- **User requirements** and acceptance criteria (spec.md)
- **Setup instructions** and validation steps (quickstart.md)
- **Code patterns** and examples (CLAUDE.md)

### Reference Format
- **Line numbers**: `data-model.md lines 15-65` - Look at specific line range
- **Sections**: `research.md section 5` - Find the numbered section
- **Requirements**: `spec.md FR-004` - Reference specific functional requirement
- **Test scenarios**: `quickstart.md "Test 3: Authentication"` - Follow test steps

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → SUCCESS: Tech stack extracted (Rust, Tokio, Axum, Dioxus, SurrealDB)
2. Load optional design documents:
   → data-model.md: 4 entities, 4 relationships extracted
   → contracts/openapi.yaml: 8 endpoints extracted  
   → research.md: Technical decisions loaded
3. Generate tasks by category:
   → Setup: 5 tasks (project init, dependencies, infrastructure)
   → Tests: 15 tasks (8 contract, 7 integration)
   → Core: 25 tasks (models, services, endpoints, UI)
   → Integration: 6 tasks (DB, middleware, logging)
   → Polish: 8 tasks (unit tests, performance, docs)
4. Apply task rules:
   → Marked [P] for parallel execution where applicable
   → Tests before implementation enforced
5. Number tasks T001-T059
6. Generate dependency graph
7. Create parallel execution examples
8. Validate: All contracts covered, all entities modeled
9. Return: SUCCESS (59 tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
- **Backend**: `backend/src/`, `backend/tests/`
- **Frontend**: `frontend/src/`, `frontend/tests/`
- **Contracts**: `backend/tests/contract/`
- **Integration**: `backend/tests/integration/`

## Phase 3.1: Setup & Infrastructure

- [ ] T001 Create project structure with backend/ and frontend/ directories per plan.md
- [ ] T002 Initialize backend Rust project with Cargo.toml including Tokio, Axum, SurrealDB dependencies
- [ ] T003 Initialize frontend Rust project with Cargo.toml including Dioxus, wasm-bindgen dependencies
- [ ] T004 [P] Create docker-compose.yml with SurrealDB and Redis services
- [ ] T005 [P] Configure GitHub Actions CI/CD pipeline in .github/workflows/ci.yml
- [ ] T006 [P] Set up environment configuration in backend/.env and frontend/.env
- [ ] T007 [P] Configure Rust formatting and linting with rustfmt.toml and clippy.toml

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Contract Tests [P] - All can run in parallel
- [ ] T008 [P] Contract test GET /api/anime/{id} in backend/tests/contract/test_anime_get.rs
- [ ] T009 [P] Contract test GET /api/search in backend/tests/contract/test_search.rs
- [ ] T010 [P] Contract test GET /api/browse/season/{year}/{season} in backend/tests/contract/test_browse_season.rs
- [ ] T011 [P] Contract test GET /api/anime/{id}/episodes in backend/tests/contract/test_episodes_get.rs
- [ ] T012 [P] Contract test POST /api/auth/login in backend/tests/contract/test_auth_login.rs
- [ ] T013 [P] Contract test POST /api/auth/logout in backend/tests/contract/test_auth_logout.rs
- [ ] T014 [P] Contract test POST /api/auth/refresh in backend/tests/contract/test_auth_refresh.rs
- [ ] T015 [P] Contract test GET /api/stream/{anime_id}/{episode} in backend/tests/contract/test_stream.rs

### Integration Tests [P]
- [ ] T016 [P] Integration test "Search for SPY x FAMILY" scenario in backend/tests/integration/test_search_scenario.rs
      📖 See: quickstart.md "Test 1: Search and Discovery" section; spec.md Acceptance Scenario 1
- [ ] T017 [P] Integration test "Login with valid/invalid credentials" in backend/tests/integration/test_auth_flow.rs
      📖 See: quickstart.md "Test 3: Authentication Flow" section; spec.md Acceptance Scenario 3
- [ ] T018 [P] Integration test "Authenticated video streaming" in backend/tests/integration/test_streaming.rs
      📖 See: quickstart.md "Test 4: Video Streaming" section; spec.md Acceptance Scenarios 4-5
- [ ] T019 [P] Integration test "Session persistence" in backend/tests/integration/test_session.rs
      📖 See: quickstart.md "Test 5: Session Management" section; spec.md FR-005
- [ ] T020 [P] Integration test "Browse by season" in backend/tests/integration/test_seasonal_browse.rs
      📖 See: quickstart.md "Test 8: Seasonal Browse" section; spec.md FR-006
- [ ] T021 [P] Performance test "API response <200ms" in backend/tests/integration/test_performance.rs
      📖 See: quickstart.md "Test 6: Performance Validation" section; spec.md NFR-1.1
- [ ] T022 [P] E2E test "Complete user journey" in frontend/tests/e2e/test_user_journey.rs
      📖 See: quickstart.md "Success Criteria Checklist" section; spec.md "Primary User Story"

## Phase 3.3: Core Implementation - Backend

### Data Models [P]
- [ ] T023 [P] Anime model with validation in backend/src/models/anime.rs
      📖 See: data-model.md lines 15-65 for Anime struct definition and validation rules
- [ ] T024 [P] Episode model in backend/src/models/episode.rs
      📖 See: data-model.md lines 74-95 for Episode struct and validation
- [ ] T025 [P] Tag model and enums in backend/src/models/tag.rs
      📖 See: data-model.md lines 97-113 for Tag struct and TagCategory enum
- [ ] T026 [P] Session model with JWT handling in backend/src/models/session.rs
      📖 See: data-model.md lines 115-130 for Session struct; research.md section 5 for JWT strategy
- [ ] T027 [P] Graph relationships (has_tag, is_sequel_of, etc.) in backend/src/models/relationships.rs
      📖 See: data-model.md lines 134-183 for all relationship definitions

### Services/Libraries [P]
- [ ] T028 [P] Metadata ingestion library in backend/src/services/metadata_ingestion.rs
      📖 See: research.md section 1 for SurrealDB schema; data-model.md "Migration Strategy" section
- [ ] T029 [P] Auth service with crunchyroll-rs in backend/src/services/auth.rs  
      📖 See: research.md section 2 for crunchyroll-rs integration; section 5 for session flow
- [ ] T030 [P] Streaming service for URL retrieval in backend/src/services/streaming.rs
      📖 See: research.md section 4 for HLS.js approach; contracts/openapi.yaml lines 257-296
- [ ] T031 [P] Search engine with full-text search in backend/src/services/search.rs
      📖 See: data-model.md "Query Patterns" section; research.md section 6 for optimization
- [ ] T032 [P] SurrealDB connection pool in backend/src/db/connection.rs
      📖 See: research.md section 1 for connection strategy; data-model.md "Database Schema" section

### API Endpoints (Sequential - shared files)
- [ ] T033 Setup Axum router and middleware in backend/src/api/routes.rs
      📖 See: plan.md lines 30-32 for tech stack; CLAUDE.md "Axum Handler" pattern
- [ ] T034 GET /api/anime/{id} handler in backend/src/api/handlers/anime.rs
      📖 See: contracts/openapi.yaml lines 24-44; data-model.md "Get Anime with All Relationships" query
- [ ] T035 GET /api/search handler in backend/src/api/handlers/search.rs
      📖 See: contracts/openapi.yaml lines 46-77; data-model.md "Search by Title or Synonym" query
- [ ] T036 GET /api/browse/season/{year}/{season} handler in backend/src/api/handlers/browse.rs
      📖 See: contracts/openapi.yaml lines 79-117; data-model.md "Browse by Season" query
- [ ] T037 GET /api/anime/{id}/episodes handler in backend/src/api/handlers/episodes.rs
      📖 See: contracts/openapi.yaml lines 119-143; data-model.md "Get Episodes for Anime" query
- [ ] T038 POST /api/auth/login handler in backend/src/api/handlers/auth.rs
      📖 See: contracts/openapi.yaml lines 145-182; research.md section 5 "Session Flow"
- [ ] T039 POST /api/auth/logout handler in backend/src/api/handlers/auth.rs (same file as T038)
      📖 See: contracts/openapi.yaml lines 184-195
- [ ] T040 POST /api/auth/refresh handler in backend/src/api/handlers/auth.rs (same file as T038)
      📖 See: contracts/openapi.yaml lines 197-231
- [ ] T041 GET /api/stream/{anime_id}/{episode} handler in backend/src/api/handlers/stream.rs
      📖 See: contracts/openapi.yaml lines 233-296; research.md section 4 "Integration Approach"

### CLI Tools [P]
- [ ] T042 [P] Ingestion CLI for anime-offline-database in backend/src/cli/ingest.rs
      📖 See: data-model.md "Migration Strategy" section; quickstart.md "Import Anime Data" section
- [ ] T043 [P] Database initialization CLI in backend/src/cli/db_init.rs
      📖 See: data-model.md "Database Schema" section; quickstart.md "Start Infrastructure" section

## Phase 3.4: Core Implementation - Frontend

### Components [P]
- [ ] T044 [P] IP Hub component with tabs in frontend/src/components/ip_hub.rs
      📖 See: spec.md FR-011 for disabled tabs requirement; CLAUDE.md "Dioxus Component" pattern
- [ ] T045 [P] Search bar component with autocomplete in frontend/src/components/search_bar.rs
      📖 See: spec.md FR-002 for search requirements; research.md section 7 for state management
- [ ] T046 [P] Video player wrapper for HLS.js in frontend/src/components/video_player.rs
      📖 See: research.md section 4 "Integration Approach" for HLS.js; spec.md FR-009 for controls
- [ ] T047 [P] Anime card for grid display in frontend/src/components/anime_card.rs
      📖 See: contracts/openapi.yaml AnimeSummary schema; spec.md FR-003 for display requirements
- [ ] T048 [P] Episode list component in frontend/src/components/episode_list.rs
      📖 See: spec.md FR-010 for episode list requirements; data-model.md Episode struct

### Pages [P]
- [ ] T049 [P] Home page with search in frontend/src/pages/home.rs
- [ ] T050 [P] Login page with form validation in frontend/src/pages/login.rs
- [ ] T051 [P] Series/IP Hub page in frontend/src/pages/series.rs
- [ ] T052 [P] Seasonal browse page in frontend/src/pages/browse.rs

### Frontend Services
- [ ] T053 API client with auth handling in frontend/src/services/api_client.rs
- [ ] T054 Authentication state management in frontend/src/services/auth.rs

## Phase 3.5: Integration & Middleware

- [ ] T055 JWT authentication middleware in backend/src/middleware/auth.rs
      📖 See: research.md section 5 "JWT-Based Sessions"; spec.md FR-004, FR-005, FR-007
- [ ] T056 CORS configuration in backend/src/middleware/cors.rs
      📖 See: research.md section 5 "Security Measures"; plan.md line 38 for constraints
- [ ] T057 Request/response logging with tracing in backend/src/middleware/logging.rs
      📖 See: research.md section 10 "Structured Logging"; plan.md lines 69-71
- [ ] T058 Rate limiting middleware in backend/src/middleware/rate_limit.rs
      📖 See: research.md section 2 "Limitations Discovered" for rate limiting need
- [ ] T059 Error handling middleware in backend/src/middleware/error.rs
      📖 See: research.md section 10 "Error Handling"; contracts/openapi.yaml Error schema
- [ ] T060 Frontend → Backend log streaming in frontend/src/services/logging.rs
      📖 See: plan.md line 70 "Frontend logs → backend"; research.md section 10

## Phase 3.6: Polish & Documentation

- [ ] T061 [P] Unit tests for model validation in backend/tests/unit/test_models.rs
- [ ] T062 [P] Unit tests for search algorithms in backend/tests/unit/test_search.rs
- [ ] T063 [P] Performance optimization for WASM bundle size
- [ ] T064 [P] API documentation generation from OpenAPI spec
- [ ] T065 [P] Update README.md with setup instructions
- [ ] T066 [P] Create DEPLOYMENT.md for production deployment
- [ ] T067 Load testing with k6 scripts in tests/load/
- [ ] T068 Execute quickstart.md validation checklist

## Dependencies

### Critical Path
1. **Setup** (T001-T007) → Everything else
2. **Tests** (T008-T022) → **MUST** complete before implementation
3. **Models** (T023-T027) → Services (T028-T032) → Endpoints (T033-T041)
4. **Backend Core** (T023-T043) → Frontend can start in parallel
5. **Middleware** (T055-T060) requires endpoints complete
6. **Polish** (T061-T068) after all implementation

### Blocking Dependencies
- T032 (DB connection) blocks T028-T031 (services)
- T033 (router setup) blocks T034-T041 (endpoints)
- T053 (API client) blocks T049-T052 (pages)
- T055 (auth middleware) blocks T041 (streaming endpoint)

## Parallel Execution Examples

### Launch all contract tests together (T008-T015):
```bash
# Using Task agent
Task: "Contract test GET /api/anime/{id} in backend/tests/contract/test_anime_get.rs"
Task: "Contract test GET /api/search in backend/tests/contract/test_search.rs"
Task: "Contract test GET /api/browse/season in backend/tests/contract/test_browse_season.rs"
Task: "Contract test GET /api/anime/{id}/episodes in backend/tests/contract/test_episodes_get.rs"
Task: "Contract test POST /api/auth/login in backend/tests/contract/test_auth_login.rs"
Task: "Contract test POST /api/auth/logout in backend/tests/contract/test_auth_logout.rs"
Task: "Contract test POST /api/auth/refresh in backend/tests/contract/test_auth_refresh.rs"
Task: "Contract test GET /api/stream in backend/tests/contract/test_stream.rs"
```

### Launch all models together (T023-T027):
```bash
Task: "Create Anime model with validation in backend/src/models/anime.rs"
Task: "Create Episode model in backend/src/models/episode.rs"
Task: "Create Tag model and enums in backend/src/models/tag.rs"
Task: "Create Session model with JWT in backend/src/models/session.rs"
Task: "Define graph relationships in backend/src/models/relationships.rs"
```

### Launch all frontend components together (T044-T048):
```bash
Task: "Build IP Hub component in frontend/src/components/ip_hub.rs"
Task: "Build search bar component in frontend/src/components/search_bar.rs"
Task: "Build video player wrapper in frontend/src/components/video_player.rs"
Task: "Build anime card component in frontend/src/components/anime_card.rs"
Task: "Build episode list component in frontend/src/components/episode_list.rs"
```

## Validation Checklist
*GATE: All items verified before task generation*

- [x] All 8 API endpoints have corresponding contract tests
- [x] All 4 entities (Anime, Episode, Tag, Session) have model tasks
- [x] All tests (T008-T022) come before implementation (T023+)
- [x] Parallel tasks are truly independent (different files)
- [x] Each task specifies exact file path
- [x] No [P] task modifies same file as another [P] task
- [x] User stories from quickstart.md covered in integration tests
- [x] Performance requirements have specific test tasks

## Notes
- Commit after each task with descriptive message
- Run `cargo test` after implementing each endpoint
- Use `cargo clippy` before committing
- Frontend development can proceed in parallel with backend after T023
- Database must be running (`docker-compose up -d`) before integration tests

## Success Metrics
- All 68 tasks completed
- All tests passing (contract, integration, unit, e2e)
- Performance targets met (<200ms API, <2s FCP, <4s TTFF)
- Quickstart validation checklist complete
- Zero clippy warnings

---

## Quick Reference Matrix

### Where to Find Information by Task Type

| Task Type | Primary Reference | Secondary Reference | Code Examples |
|-----------|------------------|-------------------|---------------|
| **Data Models** (T023-T027) | data-model.md "Core Entities" | data-model.md "Database Schema" | CLAUDE.md patterns |
| **API Endpoints** (T034-T041) | contracts/openapi.yaml paths | data-model.md "Query Patterns" | CLAUDE.md "Axum Handler" |
| **Services** (T028-T032) | research.md technical decisions | plan.md libraries list | - |
| **Frontend Components** (T044-T048) | spec.md functional requirements | research.md section 7 | CLAUDE.md "Dioxus Component" |
| **Authentication** (T029, T038-T040, T055) | research.md section 5 | contracts/openapi.yaml auth paths | - |
| **Testing** (T008-T022) | quickstart.md validation scenarios | spec.md acceptance scenarios | - |
| **Database** (T032, T042-T043) | data-model.md "Database Schema" | research.md section 1 | CLAUDE.md "SurrealDB Query" |
| **Performance** (T021, T063, T067) | spec.md NFRs | quickstart.md "Test 6" | research.md optimization |

### Key Document Sections

**data-model.md Structure:**
- Lines 10-130: Entity definitions (Anime, Episode, Tag, Session)
- Lines 134-183: Graph relationships
- Lines 185-280: Database schema
- Lines 282-320: Query patterns
- Lines 350+: Migration strategy

**contracts/openapi.yaml Structure:**
- Lines 24-44: GET /api/anime/{id}
- Lines 46-77: GET /api/search
- Lines 79-117: GET /api/browse/season
- Lines 145-182: POST /api/auth/login
- Lines 233-296: GET /api/stream
- Lines 300+: Component schemas

**research.md Sections:**
1. SurrealDB Graph Modeling
2. Crunchyroll-rs Integration
3. Dioxus WASM Optimization
4. Video Streaming Architecture
5. Authentication & Session Management
6. Database Query Optimization
7. Frontend State Management
8. Development Environment
9. Deployment Architecture
10. Error Handling & Observability

---
*Generated from Project Kenshō POC design documents*
*Total Tasks: 68 | Parallel Capable: 42 | Sequential: 26*
*Enhanced with documentation references for guided implementation*