use axum::{
    Router,
    routing::post,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::net::SocketAddr;

// Import our custom JSON extractor
use kensho_backend::middleware::json_extractor::ValidatedJson;

#[derive(Debug, Deserialize)]
struct TestRequest {
    email: String,
    password: String,
}

async fn test_standard_json(
    Json(req): Json<TestRequest>,
) -> impl IntoResponse {
    (StatusCode::OK, format!("Standard JSON received: {:?}", req))
}

async fn test_validated_json(
    ValidatedJson(req): ValidatedJson<TestRequest>,
) -> impl IntoResponse {
    (StatusCode::OK, format!("Validated JSON received: {:?}", req))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/standard", post(test_standard_json))
        .route("/validated", post(test_validated_json));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Test server running on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}