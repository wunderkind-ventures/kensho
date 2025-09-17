# Implementation Plan: Project Kenshō POC - All-Rust Anime Streaming Platform

**Branch**: `001-project-kensh-poc` | **Date**: September 13, 2025 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-project-kensh-poc/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → SUCCESS: Specification loaded and analyzed
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detected Project Type: web (frontend + backend)
   → Set Structure Decision: Option 2 - Web application
3. Evaluate Constitution Check section below
   → No violations detected - approach follows simplicity principles
   → Update Progress Tracking: Initial Constitution Check
4. Execute Phase 0 → research.md
   → Research completed: All technical decisions clarified
5. Execute Phase 1 → contracts, data-model.md, quickstart.md, CLAUDE.md
   → Design artifacts generated
6. Re-evaluate Constitution Check section
   → No new violations detected
   → Update Progress Tracking: Post-Design Constitution Check  
7. Plan Phase 2 → Task generation approach defined
8. STOP - Ready for /tasks command
```

## Summary
Build a high-performance anime streaming POC using an all-Rust stack to validate aggregated metadata presentation and secure Crunchyroll integration. The system will provide enriched anime discovery through graph-based relationships, authenticate users via Crunchyroll credentials, and stream video content directly from Crunchyroll servers.

## Technical Context
**Language/Version**: Rust 1.75+ (backend and frontend)
**Primary Dependencies**: Tokio (async runtime), Axum (web framework), Dioxus (WASM frontend), SurrealDB (graph database), crunchyroll-rs (API integration)
**Storage**: SurrealDB for metadata and graph relationships
**Testing**: cargo test with integration and e2e test suites
**Target Platform**: Web browsers (Chrome, Firefox, Safari latest stable)
**Project Type**: web - Full-stack application with WASM frontend
**Performance Goals**: <200ms P95 API response, <2s First Contentful Paint, <4s Time to First Frame
**Constraints**: No credential storage, secure session management, browser compatibility
**Scale/Scope**: POC for ~1000 anime series, ~10k episodes, ~100 concurrent users

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 2 (backend, frontend)
- Using framework directly? Yes - Axum and Dioxus without wrappers
- Single data model? Yes - shared between backend and frontend
- Avoiding patterns? Yes - direct database access, no unnecessary abstractions

**Architecture**:
- EVERY feature as library? Yes - core logic in libraries
- Libraries listed:
  - `metadata-ingestion`: Parse and import anime data
  - `auth-service`: Handle Crunchyroll authentication
  - `streaming-service`: Manage video stream URLs
  - `search-engine`: Query and filter anime data
- CLI per library: Each library exposes CLI for testing/debugging
- Library docs: llms.txt format planned for each library

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? Yes
- Git commits show tests before implementation? Will be enforced
- Order: Contract→Integration→E2E→Unit strictly followed? Yes
- Real dependencies used? Yes - actual SurrealDB instance, real Crunchyroll API
- Integration tests for: All API contracts, authentication flow, streaming endpoints
- FORBIDDEN: Implementation before test - strictly avoided

**Observability**:
- Structured logging included? Yes - tracing crate for structured logs
- Frontend logs → backend? Yes - unified logging stream
- Error context sufficient? Yes - detailed error context with request IDs

**Versioning**:
- Version number assigned? 0.1.0 (POC version)
- BUILD increments on every change? Yes
- Breaking changes handled? N/A for POC

## Project Structure

### Documentation (this feature)
```
specs/001-project-kensh-poc/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 2: Web application (selected based on Technical Context)
backend/
├── src/
│   ├── models/
│   │   ├── anime.rs
│   │   ├── episode.rs
│   │   └── session.rs
│   ├── services/
│   │   ├── metadata_ingestion.rs
│   │   ├── auth.rs
│   │   ├── streaming.rs
│   │   └── search.rs
│   └── api/
│       ├── routes.rs
│       └── handlers.rs
└── tests/
    ├── contract/
    ├── integration/
    └── unit/

frontend/
├── src/
│   ├── components/
│   │   ├── ip_hub.rs
│   │   ├── search_bar.rs
│   │   └── video_player.rs
│   ├── pages/
│   │   ├── home.rs
│   │   ├── login.rs
│   │   └── series.rs
│   └── services/
│       ├── api_client.rs
│       └── auth.rs
└── tests/
```

**Structure Decision**: Option 2 - Web application structure selected based on frontend+backend requirements

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context**:
   - SurrealDB graph modeling best practices
   - crunchyroll-rs library capabilities and limitations
   - Dioxus WASM performance optimization techniques
   - HLS/DASH video streaming in browser

2. **Generate and dispatch research agents**:
   ```
   Task: "Research SurrealDB graph relationships for anime metadata"
   Task: "Find best practices for Rust WASM bundle optimization"
   Task: "Research crunchyroll-rs authentication flow"
   Task: "Investigate HLS.js integration with Dioxus"
   ```

3. **Consolidate findings** in `research.md`

**Output**: research.md with all technical decisions documented

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Anime: title, synonyms, sources, episodes, status, type, season, IMDb data
   - Episode: number, title, anime_id, duration
   - Tag: name, category
   - Session: token, user_id, expires_at
   - Graph edges: has_tag, is_sequel_of, is_prequel_of, is_related_to

2. **Generate API contracts** from functional requirements:
   - GET /api/anime/:id - Retrieve anime details
   - GET /api/search?q={query} - Search anime by title/synonyms
   - GET /api/browse/season/:year/:season - Browse by season
   - POST /api/auth/login - Authenticate with Crunchyroll
   - POST /api/auth/logout - Invalidate session
   - GET /api/anime/:id/episodes - List episodes
   - GET /api/stream/:anime_id/:episode - Get stream URL (authenticated)

3. **Generate contract tests** from contracts:
   - One test file per endpoint in `/backend/tests/contract/`
   - OpenAPI schema validation
   - Request/response structure tests

4. **Extract test scenarios** from user stories:
   - Search for "SPY x FAMILY" and navigate to IP Hub
   - Login flow with valid/invalid credentials
   - Authenticated video streaming initiation
   - Performance validation tests

5. **Update CLAUDE.md incrementally**:
   - Add Rust/Tokio/Axum/Dioxus patterns
   - Include SurrealDB query examples
   - Document crunchyroll-rs usage

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, CLAUDE.md

## Phase 2: Task Planning Approach (Enhanced for Production)
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs
- Contract test tasks for each API endpoint [P]
- Model creation tasks for each entity [P]
- Integration test tasks for user stories
- Implementation tasks organized by milestone
- **NEW**: Production-ready enhancement tasks

**Enhanced Task Categories**:
- **Resilience Tasks**: Connection pooling, retry logic, circuit breakers
- **Observability Tasks**: Health checks, metrics, distributed tracing
- **Performance Tasks**: Caching strategy, query optimization, response compression
- **API Enhancement Tasks**: OpenAPI generation, field selection, pagination
- **Security Tasks**: Rate limiting, input validation, audit logging

**Ordering Strategy**:
- Milestone 1: Data ingestion and foundation (T001-T022)
- Milestone 2: Authentication and streaming (T023-T041)
- Milestone 3: Frontend UI and integration (T042-T060)
- **Milestone 4**: Production hardening (T061-T080)
- **Milestone 5**: Observability and validation (T081-T110)
- TDD order maintained throughout

**Estimated Output**: 
- Original: 60 tasks (T001-T060) - POC implementation
- Enhanced: 50 additional tasks (T061-T110) - Production features
- Total: 110 numbered, ordered tasks

## Production Enhancement Strategy
*Added September 16, 2025 - Post POC implementation*

### Enhancement Goals
Transform the POC into a production-ready system through incremental improvements that maintain backward compatibility.

### Key Production Features

**1. Service Resilience (T061-T070)**
- Connection pooling for external APIs
- Retry logic with exponential backoff
- Circuit breaker pattern for failures
- Timeout configuration for all calls
- Graceful degradation strategies

**2. Performance Optimization (T071-T080)**
- Multi-layer caching (L1: Memory, L2: Redis, L3: Database)
- Database query optimization and indexing
- Response compression and streaming
- Lazy loading and pagination
- Cache warming and invalidation

**3. Observability Stack (T081-T090)**
- Health check endpoints (/health/live, /health/ready)
- Prometheus metrics collection
- OpenTelemetry distributed tracing
- Structured logging with correlation IDs
- Performance monitoring dashboards

**4. API Enhancement (T091-T100)**
- OpenAPI/Swagger documentation generation
- Field selection support (?fields=title,episodes)
- Cursor-based pagination
- Batch operations for efficiency
- SDK generation for multiple languages

**5. Testing & Validation (T101-T110)**
- k6 load testing scenarios
- Chaos engineering experiments
- Security audit and penetration testing
- Performance benchmarking
- Integration test expansion

### Implementation Timeline
- **Week 1**: Foundation & Resilience
- **Week 2**: Caching & Performance
- **Week 3**: Observability & Monitoring
- **Week 4**: API Polish & Documentation
- **Week 5**: Testing & Validation

### Success Criteria
- P95 latency < 200ms (with cache)
- 99.9% uptime SLA
- <1% error rate under load
- 80%+ cache hit ratio
- Zero downtime deployments

## Complexity Tracking
*No violations detected - POC follows constitutional simplicity principles*
*Production enhancements maintain simplicity through modular, independent components*

## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command) 
- [x] Phase 2: Task planning complete (/plan command - approach defined)
- [x] Phase 3: Tasks generated (/tasks command)
- [x] Phase 3.1: Setup & Infrastructure (T001-T007) - COMPLETE
- [x] Phase 3.2: Tests First (T008-T022) - COMPLETE
- [x] Phase 3.3: Core Backend (T023-T041) - COMPLETE
- [x] Phase 3.4: Core Frontend (T042-T054) - COMPLETE
- [x] Phase 3.5: Integration & Middleware (T055-T060) - COMPLETE
- [x] Phase 3.6: JSON Validation Enhancement - COMPLETE
- [ ] Phase 4: Production Hardening (T061-T080) - IN PLANNING
- [ ] Phase 5: Observability (T081-T090) - PLANNED
- [ ] Phase 6: API Enhancement (T091-T100) - PLANNED
- [ ] Phase 7: Testing & Validation (T101-T110) - PLANNED

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented (none needed)

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*