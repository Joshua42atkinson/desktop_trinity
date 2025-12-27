use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// A unified error type for the entire Daydream Backend.
/// This prevents "Stringly Typed" errors and ensures privacy.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication required")]
    AuthError,

    #[error("User not found")]
    NotFound,

    #[error("Database integrity violation")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Invalid input data: {0}")]
    ValidationError(&'static str),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
}

/// A custom Result type for our application.
pub type Result<T> = std::result::Result<T, AppError>;

// Allow conversion from anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

/// How to convert an AppError into an HTTP Response.
/// Notice: Database errors are logged internally but appear as "500 Internal Error" to the user.
/// This satisfies the "Privacy-First" Directive.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::AuthError => (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),

            // SECURITY CRITICAL: Log the real error, send a generic one.
            AppError::DatabaseError(e) => {
                tracing::error!("Database Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal Server Error: {}", msg),
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
