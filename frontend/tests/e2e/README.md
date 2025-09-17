# E2E Tests - Project Kenshō Frontend

## Overview
End-to-end tests that validate the complete user journey through the Kenshō application, from landing page through authentication, search, discovery, and streaming initiation.

## Test Coverage (T022)

### User Journey Tests
- **Complete User Flow**: Landing → Login → Search → Select → Stream
- **Authentication Persistence**: Login state across navigation
- **Seasonal Browse**: Browse current season with filters
- **Watchlist Management**: Add/remove anime from watchlist
- **Error Handling**: Invalid searches, empty queries
- **Responsive Navigation**: Mobile menu functionality

### Performance Tests
- **Page Load**: < 3 seconds initial load
- **Navigation**: < 1 second between pages
- **Search Response**: < 2 seconds for results
- **Stream Initiation**: < 5 seconds to player ready

## Prerequisites

### Required Tools
```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install Chrome or Firefox driver
# Chrome: https://chromedriver.chromium.org/
# Firefox: https://github.com/mozilla/geckodriver
```

### Required Services
1. Backend running on http://localhost:3000
2. Frontend running on http://localhost:8080

## Running Tests

### Quick Start
```bash
# Run all E2E tests
./tests/e2e/run_tests.sh

# Run with visible browser (non-headless)
./tests/e2e/run_tests.sh --headed

# Run specific test
wasm-pack test --chrome -- --test user_journey
```

### Manual Test Execution
```bash
cd frontend

# Run tests in Chrome (headless)
wasm-pack test --chrome --headless

# Run tests in Firefox
wasm-pack test --firefox --headless

# Run tests with browser window visible
wasm-pack test --chrome
```

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Setup wasm-pack
  run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

- name: Start services
  run: |
    docker-compose up -d
    cargo run --bin backend-server &
    cd frontend && trunk serve &
    sleep 10  # Wait for services

- name: Run E2E tests
  run: |
    cd frontend
    wasm-pack test --chrome --headless
```

## Test Structure

### Page Object Pattern
```rust
// Using PageObject for cleaner tests
let page = PageObject::new();
page.click(".login-button")?;
page.set_input_value("#email", "user@example.com")?;
assert!(page.wait_for_element(".dashboard", 5000).await);
```

### Test Utilities
- `PageObject`: Encapsulates page interactions
- `PerformanceMetrics`: Tracks timing metrics
- `mock_data`: Provides test data
- `assertions`: Custom assertion helpers

## Test Files

### Core Test Files
- `user_journey.rs`: Main E2E test scenarios
- `test_utils.rs`: Shared utilities and helpers
- `mod.rs`: Test module configuration

### Test Scenarios

#### 1. Complete User Journey
```rust
#[wasm_bindgen_test]
async fn test_complete_user_journey() {
    // Land on home page
    // Navigate to login
    // Enter credentials
    // Search for anime
    // Select from results
    // Choose episode
    // Verify streaming
}
```

#### 2. Authentication Flow
```rust
#[wasm_bindgen_test]
async fn test_authentication_persistence() {
    // Login
    // Navigate between pages
    // Verify auth persists
    // Logout
}
```

#### 3. Search & Discovery
```rust
#[wasm_bindgen_test]
async fn test_search_functionality() {
    // Enter search query
    // View results
    // Filter results
    // Select anime
}
```

## Debugging Tests

### Enable Debug Logging
```rust
// In test setup
console_log::init_with_level(log::Level::Debug).ok();
log::info!("Test step: {}", description);
```

### Browser Developer Tools
1. Run tests with `--headed` flag
2. Open browser DevTools (F12)
3. Set breakpoints in WASM code
4. Inspect network requests

### Common Issues

#### Tests Timeout
- Increase timeout values in `wait_for_element`
- Check if services are running
- Verify network connectivity

#### Elements Not Found
- Check selectors match current HTML
- Ensure page has loaded completely
- Verify element visibility

#### Authentication Failures
- Check backend auth service
- Verify test credentials
- Clear localStorage between tests

## Performance Benchmarks

### Target Metrics
| Operation | Target | Actual |
|-----------|--------|--------|
| Initial Load | < 3s | TBD |
| Navigation | < 1s | TBD |
| Search | < 2s | TBD |
| Stream Init | < 5s | TBD |

### Measuring Performance
```rust
let mut metrics = PerformanceMetrics::new();
metrics.mark("search_start");
// ... perform search ...
metrics.mark("search_end");
metrics.assert_performance("search_start", "search_end", 2000.0, "Search");
```

## Continuous Integration

### Test Pipeline
1. **Build**: Compile frontend WASM
2. **Start Services**: Launch backend & frontend
3. **Wait**: Ensure services are ready
4. **Test**: Run E2E test suite
5. **Report**: Generate test report
6. **Cleanup**: Stop services

### Environment Variables
```bash
BASE_URL=http://localhost:8080
API_URL=http://localhost:3000
TEST_MODE=true
HEADLESS=true
```

## Extending Tests

### Adding New Test
1. Create test function in `user_journey.rs`
2. Use `#[wasm_bindgen_test]` attribute
3. Implement test logic using PageObject
4. Add assertions for expected behavior

### Example Template
```rust
#[wasm_bindgen_test]
async fn test_new_feature() {
    let page = PageObject::new();
    
    // Arrange
    page.click(".feature-button").unwrap();
    
    // Act
    page.wait_for_element(".feature-modal", 3000).await;
    
    // Assert
    assert!(page.element_exists(".expected-element"));
}
```

## Test Maintenance

### Regular Updates
- Update selectors when HTML changes
- Adjust timing for performance changes
- Add tests for new features
- Remove tests for deprecated features

### Best Practices
1. Keep tests independent
2. Clean up state after each test
3. Use meaningful assertions
4. Add descriptive error messages
5. Document complex test scenarios

---

*Part of Project Kenshō Test Suite - T022: E2E Frontend Tests*