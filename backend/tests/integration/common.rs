// Common utilities for integration tests

use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};

pub struct TestApp {
    pub client: reqwest::Client,
    pub db: Arc<Surreal<Any>>,
    pub redis: redis::Client,
    pub base_url: String,
}

pub async fn setup_integration_test() -> TestApp {
    // This will fail until implementation
    panic!("Integration test setup not implemented - test should fail");
}