// Common test utilities for contract tests

use axum::{Router, http::StatusCode};
use reqwest::Client;
use surrealdb::{Surreal, engine::any::Any};
use tower::ServiceExt;
use std::sync::Arc;

pub struct TestApp {
    pub client: Client,
    pub db: Arc<Surreal<Any>>,
    pub address: String,
}

pub async fn setup_test_app() -> TestApp {
    // This will fail until the actual app is implemented
    panic!("App not implemented yet - test should fail");
}

pub async fn create_test_router() -> Router {
    // This will fail until routes are implemented
    panic!("Routes not implemented yet - test should fail");
}

pub async fn setup_test_database() -> Arc<Surreal<Any>> {
    // This will fail until database connection is implemented
    panic!("Database setup not implemented yet - test should fail");
}