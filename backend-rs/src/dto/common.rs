//! Shared response DTOs (mirror schemas/common.py).
//!
//! Fully-qualified `serde::` derives are used so there is no `use serde::...`
//! import to be flagged as unused.

/// Simple `{"message": "..."}` envelope returned by mutating endpoints.
#[derive(Debug, serde::Serialize)]
pub struct MessageResponse {
    pub message: String,
}

impl MessageResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Validate a 1..=100 character name, mirroring `Field(min_length=1,
/// max_length=100)` used by presets/smart folders/showcases. Returns the error
/// message (for `AppError::BadRequest`) when invalid. Length is counted in
/// Unicode scalar values, matching Python's `len(str)` semantics closely enough
/// for the VARCHAR(100) column.
pub fn validate_name(name: &str) -> Result<(), String> {
    let len = name.chars().count();
    if len == 0 {
        return Err("name must not be empty".to_string());
    }
    if len > 100 {
        return Err("name must be at most 100 characters".to_string());
    }
    Ok(())
}

/// Flat pagination envelope mirroring schemas/common.py PaginatedResponse[T].
/// Used by endpoints other than the image listing (which carries an additional
/// nested `pagination` object for Falcon — see dto::image::ImageListResponse).
#[derive(Debug, serde::Serialize)]
#[allow(dead_code)] // consumed by list endpoints ported in #2
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}
