# Research: Project Kenshō POC Technical Decisions

**Feature**: Project Kenshō POC - All-Rust Anime Streaming Platform
**Date**: September 13, 2025
**Status**: Complete

## Executive Summary
This document consolidates all technical research and decisions for the Kenshō POC implementation, resolving all unknowns identified in the planning phase.

## 1. SurrealDB Graph Modeling for Anime Metadata

### Decision: Graph-First Schema Design
**Rationale**: SurrealDB's native graph capabilities enable efficient traversal of anime relationships (sequels, prequels, related series) and tag associations without complex joins.

**Implementation Approach**:
```surql
-- Nodes
DEFINE TABLE anime SCHEMAFULL;
DEFINE TABLE tag SCHEMAFULL;
DEFINE TABLE episode SCHEMAFULL;

-- Edges
DEFINE TABLE has_tag SCHEMAFULL;
DEFINE TABLE is_sequel_of SCHEMAFULL;
DEFINE TABLE is_prequel_of SCHEMAFULL;
DEFINE TABLE is_related_to SCHEMAFULL;
```

**Alternatives Considered**:
- Traditional relational model: Rejected due to complex joins for relationship queries
- Document-only approach: Rejected due to inefficient relationship traversal
- Neo4j: Rejected to maintain all-Rust stack consistency

## 2. Crunchyroll-rs Library Integration

### Decision: Direct Library Usage with Session Abstraction
**Rationale**: The crunchyroll-rs library provides stable authentication and streaming URL retrieval. We'll wrap it in a thin service layer for session management.

**Key Capabilities Identified**:
- OAuth2 authentication flow support
- Session token management
- Stream manifest URL retrieval
- Episode metadata access

**Limitations Discovered**:
- No built-in rate limiting (must implement)
- Session expiry requires manual handling
- Regional restrictions apply to content

**Implementation Strategy**:
- Implement session cache with TTL
- Add exponential backoff for API calls
- Handle region-locked content gracefully

## 3. Dioxus WASM Performance Optimization

### Decision: Component-Level Code Splitting with Lazy Loading
**Rationale**: Reduces initial bundle size and improves First Contentful Paint metrics.

**Optimization Techniques**:
1. **Bundle Size Reduction**:
   - wee_alloc for smaller memory allocator
   - wasm-opt with -Oz flag for size optimization
   - Tree-shaking unused dependencies

2. **Rendering Performance**:
   - Virtual DOM diffing minimizes re-renders
   - Component memoization for expensive computations
   - Lazy loading for below-fold content

3. **Network Optimization**:
   - Brotli compression for WASM bundles
   - CDN distribution for static assets
   - Prefetching for predictable navigation

**Alternatives Considered**:
- Yew framework: Similar performance but less mature ecosystem
- Native JavaScript: Rejected to maintain all-Rust requirement
- Server-side rendering: Deferred to future phases

## 4. Video Streaming Architecture

### Decision: HLS.js with Adaptive Bitrate Streaming
**Rationale**: HLS.js provides broad browser support and handles adaptive streaming automatically.

**Integration Approach**:
1. Backend fetches HLS manifest URL from Crunchyroll
2. Frontend loads HLS.js library dynamically
3. Video player component wraps HLS.js instance
4. Custom controls overlay for consistent UX

**Player Features**:
- Adaptive bitrate selection
- Subtitle track support
- Fullscreen API integration
- Keyboard shortcuts
- Picture-in-picture support

**Alternatives Considered**:
- DASH.js: Less browser support
- Native HTML5 video: No HLS support in all browsers
- Video.js: Heavier dependency

## 5. Authentication & Session Management

### Decision: JWT-Based Sessions with Redis Cache
**Rationale**: Stateless authentication scales better and enables distributed deployment.

**Session Flow**:
1. User provides Crunchyroll credentials
2. Backend authenticates via crunchyroll-rs
3. Generate JWT with session data
4. Cache Crunchyroll token in Redis (encrypted)
5. Frontend stores JWT in httpOnly cookie
6. Backend validates JWT and retrieves CR token for streaming

**Security Measures**:
- 15-minute JWT expiry with refresh tokens
- Encrypted Redis storage for CR tokens
- HTTPS-only cookie transmission
- CORS configuration for API endpoints

## 6. Database Query Optimization

### Decision: Materialized Views for Common Queries
**Rationale**: Precomputed views for search and browse operations ensure <200ms response times.

**Optimization Strategy**:
```surql
-- Materialized view for seasonal browse
DEFINE TABLE anime_by_season AS
  SELECT * FROM anime
  WHERE animeSeason.year = $year
  AND animeSeason.season = $season
  ORDER BY rating DESC;

-- Full-text search index
DEFINE INDEX anime_search ON anime 
  COLUMNS title, synonyms 
  SEARCH ANALYZER ascii FILTERS lowercase, synonyms;
```

## 7. Frontend State Management

### Decision: Dioxus Hooks with Local Storage Persistence
**Rationale**: Built-in hooks reduce complexity while local storage enables offline resilience.

**State Architecture**:
- Global state: Authentication, user preferences
- Local state: Component-specific UI state
- Cached state: API responses with TTL
- Persistent state: Watch progress, preferences

## 8. Development & Testing Environment

### Decision: Docker Compose for Local Development
**Rationale**: Ensures consistent environment across developers and CI/CD.

**Stack Components**:
```yaml
services:
  surrealdb:
    image: surrealdb/surrealdb:latest
  redis:
    image: redis:alpine
  backend:
    build: ./backend
  frontend:
    build: ./frontend
```

**Testing Strategy**:
- Unit tests: Pure functions and business logic
- Integration tests: API endpoints with real DB
- E2E tests: Playwright for user journeys
- Performance tests: k6 for load testing

## 9. Deployment Architecture

### Decision: Containerized Deployment with Fly.io
**Rationale**: Global edge deployment reduces latency for streaming.

**Infrastructure**:
- Backend: Multi-region Fly.io deployment
- Database: SurrealDB managed cluster
- Cache: Redis with read replicas
- CDN: Cloudflare for static assets
- Monitoring: Prometheus + Grafana

## 10. Error Handling & Observability

### Decision: Structured Logging with OpenTelemetry
**Rationale**: Unified observability across frontend and backend.

**Implementation**:
- Tracing: Request flow across services
- Metrics: Performance counters and histograms
- Logging: Structured JSON with correlation IDs
- Alerting: PagerDuty integration for critical errors

## Summary of Resolved Clarifications

All technical unknowns have been researched and resolved:

| Unknown | Resolution |
|---------|------------|
| SurrealDB graph modeling | Graph-first schema with edges as tables |
| crunchyroll-rs capabilities | Direct usage with session abstraction layer |
| Dioxus WASM optimization | Component splitting and lazy loading |
| Video streaming approach | HLS.js with adaptive bitrate |
| Session management | JWT with Redis cache for CR tokens |
| Query performance | Materialized views and search indexes |
| State management | Dioxus hooks with local storage |
| Development environment | Docker Compose stack |
| Deployment strategy | Fly.io with global edge |
| Observability | OpenTelemetry with structured logging |

## Next Steps
With all technical decisions made, proceed to Phase 1 for detailed design artifacts including data models, API contracts, and quickstart guide.