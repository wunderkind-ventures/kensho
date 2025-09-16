# Implementation Guide: Project Kenshō POC

**Branch**: `001-project-kensh-poc`
**Generated**: September 13, 2025
**Based on**: Specify CLI Workflow v2.0

## Pre-Implementation Validation

### ✅ Step 1: Specification Review
The specification has been created and validated:
- [x] User stories defined
- [x] Functional requirements (FR-001 to FR-016) documented
- [x] Non-functional requirements specified
- [x] Acceptance scenarios created
- [x] Edge cases identified

### ✅ Step 2: Plan Validation
The implementation plan has been generated with:
- [x] Technical context defined (Rust, Tokio, Axum, Dioxus, SurrealDB)
- [x] Constitution compliance checked
- [x] Project structure established
- [x] Research completed (research.md)
- [x] Data model defined (data-model.md)
- [x] API contracts specified (contracts/openapi.yaml)
- [x] Quickstart guide created (quickstart.md)

### ✅ Step 3: Task Generation
Tasks have been created with:
- [x] 68 total tasks defined
- [x] Documentation references added
- [x] Parallel execution marked
- [x] Dependencies mapped

## Areas for Additional Research

Based on the rapidly evolving nature of our tech stack, these specific areas need targeted research:

### 1. SurrealDB Graph Performance at Scale
**Question**: How do graph traversals perform with 1000+ anime and complex relationships?
**Research Task**: Test SurrealDB query performance with nested relationships up to 3 levels deep

### 2. Dioxus WASM Bundle Optimization
**Question**: What's the optimal code-splitting strategy for <2s First Contentful Paint?
**Research Task**: Benchmark different bundling strategies with wasm-pack and trunk

### 3. Crunchyroll-rs Session Management
**Question**: How to handle session refresh without interrupting video playback?
**Research Task**: Research crunchyroll-rs token refresh patterns and retry strategies

### 4. HLS.js Integration with Dioxus
**Question**: How to efficiently bridge JavaScript HLS.js with Rust WASM?
**Research Task**: Investigate wasm-bindgen for HLS.js wrapper implementation

### 5. Redis Session Encryption
**Question**: What's the best practice for encrypting Crunchyroll tokens in Redis?
**Research Task**: Research AES-256-GCM implementation in Rust for token encryption

## Over-Engineering Review

### ✅ Components That Follow YAGNI Principle
- Direct database access (no Repository pattern)
- Simple JWT sessions (no complex OAuth flow)
- Basic error handling (no custom error framework)

### ⚠️ Potential Over-Engineering to Reconsider
1. **Graph relationships**: Consider starting with simple foreign keys, add graph edges only where traversal is needed
2. **Materialized views**: Implement only after performance testing shows need
3. **Rate limiting**: May not be needed for POC with ~100 concurrent users

## Constitution Compliance Check

### ✅ Simplicity
- 2 projects only (backend, frontend)
- Direct framework usage
- Single data model

### ✅ Library-First Architecture
Each feature implemented as library:
- `metadata-ingestion` - Standalone ingestion library
- `auth-service` - Authentication library
- `streaming-service` - Stream URL management
- `search-engine` - Search functionality

### ✅ CLI Interface
Each library exposes CLI:
- `cargo run --bin ingest-metadata`
- `cargo run --bin db-init`

### ✅ Test-First Development
TDD enforced in tasks:
- T008-T022: Tests written first (must fail)
- T023-T054: Implementation only after tests fail
- RED-GREEN-Refactor cycle mandated

### ✅ Observability
- Structured logging with tracing
- Frontend→Backend log streaming
- Request correlation IDs

## Implementation Command Sequence

### Phase 1: Environment Setup
```bash
# 1. Create project structure
implement T001-T007  # Setup tasks

# 2. Start infrastructure
docker-compose up -d

# 3. Verify services
docker ps | grep -E "surrealdb|redis"
```

### Phase 2: Test-First Development
```bash
# Write all tests first (MUST FAIL)
implement T008-T015  # Contract tests
implement T016-T022  # Integration tests

# Verify tests fail
cargo test --workspace
# Expected: All tests should fail
```

### Phase 3: Backend Implementation
```bash
# Models and services
implement T023-T027  # Data models
implement T028-T032  # Services
implement T033-T041  # API endpoints
implement T042-T043  # CLI tools

# Run tests progressively
cargo test --package backend
```

### Phase 4: Frontend Implementation
```bash
# Components and pages
implement T044-T048  # Components
implement T049-T054  # Pages and services

# Build and test
cd frontend && trunk build
cargo test --package frontend
```

### Phase 5: Integration & Polish
```bash
# Middleware and final touches
implement T055-T060  # Middleware
implement T061-T068  # Polish and docs

# Full validation
cargo test --workspace
cargo clippy --all-targets
```

## Validation Before PR

### Pre-PR Checklist
- [ ] All 68 tasks completed
- [ ] All tests passing
- [ ] Performance targets met (<200ms, <2s, <4s)
- [ ] Quickstart validation complete
- [ ] No clippy warnings
- [ ] Documentation updated

### Generate PR Description
```bash
# Using GitHub CLI
gh pr create --title "feat: Implement Project Kenshō POC" \
  --body "$(cat <<'EOF'
## Summary
Implements the Project Kenshō POC with all-Rust stack validating:
- Aggregated anime metadata from multiple sources
- Crunchyroll authentication and streaming integration
- High-performance graph-based discovery
- WASM frontend with <2s load time

## Implementation Details
- **Backend**: Rust/Tokio/Axum with SurrealDB
- **Frontend**: Rust/Dioxus compiled to WASM
- **Auth**: JWT sessions with Redis cache
- **Streaming**: HLS.js integration

## Testing
- ✅ 8 contract tests
- ✅ 7 integration tests
- ✅ E2E user journey test
- ✅ Performance validation (<200ms API)

## Checklist
- [x] All functional requirements implemented (FR-001 to FR-016)
- [x] Performance targets met (NFR-1.1 to NFR-3.1)
- [x] TDD process followed
- [x] Documentation complete

Closes #001-project-kensh-poc
EOF
)"
```

## Common Implementation Issues & Solutions

### Issue: SurrealDB Connection Failed
```bash
# Check if running
docker logs kensho-surrealdb

# Restart if needed
docker-compose restart surrealdb
```

### Issue: WASM Build Failed
```bash
# Install prerequisites
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-pack

# Clean and rebuild
trunk clean && trunk build
```

### Issue: Test Failing After Implementation
```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Check test matches implementation
diff backend/tests/contract/test_*.rs backend/src/api/handlers/*.rs
```

## Next Steps After Implementation

1. **Performance Testing**
   ```bash
   k6 run tests/load/api-stress.js
   ```

2. **Security Audit**
   ```bash
   cargo audit
   ```

3. **Bundle Analysis**
   ```bash
   cd frontend && trunk build --release -- --features analyze
   ```

4. **Deployment Readiness**
   - Review DEPLOYMENT.md
   - Set up CI/CD pipeline
   - Configure production environment

---
*This guide follows the Specify CLI workflow and ensures systematic, validated implementation*