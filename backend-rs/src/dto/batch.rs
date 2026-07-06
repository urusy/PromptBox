//! Bulk operation request DTOs (mirror endpoints/batch.py).

use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct BatchUpdateRequest {
    pub ids: Vec<Uuid>,
    pub rating: Option<i32>,
    pub is_favorite: Option<bool>,
    pub needs_improvement: Option<bool>,
    pub add_tags: Option<Vec<String>>,
    pub remove_tags: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<Uuid>,
    #[serde(default)]
    pub permanent: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchRestoreRequest {
    pub ids: Vec<Uuid>,
}
