//! Health and root endpoints.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use super::AppState;

/// GET / — basic API identity (mirrors the Python GET /).
pub async fn root() -> Json<Value> {
    Json(json!({ "name": "PromptBox API", "version": "0.1.0-rs" }))
}

/// GET /health — liveness probe.
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// GET /health/db — readiness probe that pings the database.
pub async fn health_db(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "database": "ok" })),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "error", "database": "unreachable" })),
        ),
    }
}
