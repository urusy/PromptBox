//! Application error type that renders as a FastAPI-compatible JSON envelope
//! ({"detail": "..."}).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
// NotFound / BadRequest are constructed from Task #5 (read API) onward.
#[allow(dead_code)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, detail) = match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };
        (status, Json(json!({ "detail": detail }))).into_response()
    }
}

// Any error convertible into anyhow::Error (e.g. sqlx::Error) maps to a 500.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(e: E) -> Self {
        AppError::Internal(e.into())
    }
}

#[allow(dead_code)] // used from Task #5 (read API) onward
pub type AppResult<T> = Result<T, AppError>;
