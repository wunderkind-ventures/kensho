// Custom JSON extractor with better error handling
// This replaces Axum's default Json extractor to provide better error messages

use axum::{
    async_trait,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::json;

/// Custom JSON extractor that provides better error messages
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ValidatedJson(value)),
            Err(rejection) => Err(JsonError::from_rejection(rejection)),
        }
    }
}

/// JSON parsing error with detailed error messages
pub struct JsonError {
    message: String,
    details: Option<String>,
}

impl JsonError {
    fn from_rejection(rejection: JsonRejection) -> Self {
        let (message, details) = match rejection {
            JsonRejection::JsonDataError(err) => (
                "Invalid JSON format".to_string(),
                Some(format!("JSON parsing failed: {}", err)),
            ),
            JsonRejection::JsonSyntaxError(err) => (
                "Malformed JSON".to_string(),
                Some(format!("Syntax error: {}", err)),
            ),
            JsonRejection::MissingJsonContentType(_) => (
                "Missing Content-Type header".to_string(),
                Some("Content-Type must be 'application/json'".to_string()),
            ),
            JsonRejection::BytesRejection(_) => (
                "Failed to read request body".to_string(),
                None,
            ),
            _ => (
                "Invalid request".to_string(),
                Some("The request could not be processed".to_string()),
            ),
        };

        JsonError { message, details }
    }
}

impl IntoResponse for JsonError {
    fn into_response(self) -> Response {
        let status = StatusCode::BAD_REQUEST;
        
        let body = json!({
            "error": self.message,
            "details": self.details,
            "code": status.as_u16(),
        });

        (status, axum::Json(body)).into_response()
    }
}