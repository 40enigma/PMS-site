use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use uuid::Uuid;

/// Application-level error type.
///
/// Sensitive internal details (e.g. SQL messages) are logged server-side and
/// never exposed to the HTTP client.  Only safe, generic messages are returned.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Too Many Requests")]
    TooManyRequests,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Generate a correlation ID so operators can trace the specific request in logs.
        let correlation_id = Uuid::new_v4().to_string();

        let (status, client_message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests. Please slow down and try again later.".to_string(),
            ),
            AppError::Database(e) => {
                // Log the real error server-side but return a safe generic message.
                tracing::error!(
                    correlation_id = %correlation_id,
                    error = ?e,
                    "Database error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred. Please try again later.".to_string(),
                )
            }
            AppError::Internal(msg) => {
                // Log the real internal message but do not expose it to the client.
                tracing::error!(
                    correlation_id = %correlation_id,
                    message = %msg,
                    "Internal error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred. Please try again later.".to_string(),
                )
            }
        };

        let body = json!({
            "error": client_message,
            "status": status.as_u16(),
            "correlation_id": correlation_id
        });

        let mut response = (status, axum::Json(body)).into_response();

        // Attach the correlation id as a response header so clients/logs can match it.
        if let Ok(val) = axum::http::HeaderValue::from_str(&correlation_id) {
            response.headers_mut().insert("X-Correlation-Id", val);
        }

        response
    }
}
