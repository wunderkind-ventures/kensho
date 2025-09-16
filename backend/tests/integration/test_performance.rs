// T021: Performance test "API response <200ms"
// Reference: quickstart.md "Test 6: Performance Validation"
// Reference: spec.md NFR-1.1

use std::time::Instant;
mod common;
use common::*;

#[tokio::test]
async fn test_api_response_time_under_200ms() {
    let app = setup_integration_test().await;
    
    // Test multiple endpoints
    let endpoints = [
        "/api/search?q=test",
        "/api/browse/season/2024/fall",
    ];
    
    for endpoint in &endpoints {
        let start = Instant::now();
        
        let _response = app.client
            .get(&format!("{}{}", app.base_url, endpoint))
            .send()
            .await
            .expect("Request failed");
        
        let duration = start.elapsed();
        
        assert!(
            duration.as_millis() < 200,
            "API response for {} took {}ms, should be <200ms",
            endpoint,
            duration.as_millis()
        );
    }
    
    panic!("Not fully implemented - test should fail");
}