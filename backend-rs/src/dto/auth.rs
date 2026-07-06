//! Authentication request/response DTOs (mirror schemas/auth.py + common.py).
//!
//! Uses fully-qualified `serde::` derives so there is no `use serde::...`
//! import to be flagged as unused when a derive is only exercised through an
//! external trait bound (axum's Json) rather than by name in this module.

#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub username: String,
}
