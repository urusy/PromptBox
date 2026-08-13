//! Showcase DTOs (mirror schemas/showcase.py).

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct ShowcaseCreate {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

/// Update body. For `description`/`icon`, an absent/null value leaves the field
/// unchanged while an empty string clears it (mirrors the Python handler's
/// `value if value else None`). `cover_image_id` is only set when present.
#[derive(Debug, serde::Deserialize)]
pub struct ShowcaseUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub cover_image_id: Option<Uuid>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ShowcaseImageIds {
    pub image_ids: Vec<Uuid>,
}

#[derive(Debug, serde::Serialize)]
pub struct ShowcaseImageInfo {
    pub id: Uuid,
    pub storage_path: String,
    pub thumbnail_path: String,
    pub sort_order: i32,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct ShowcaseResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub cover_image_id: Option<Uuid>,
    pub cover_thumbnail_path: Option<String>,
    pub image_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct ShowcaseDetailResponse {
    #[serde(flatten)]
    pub showcase: ShowcaseResponse,
    pub images: Vec<ShowcaseImageInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct ShowcaseImageCheckResult {
    pub showcase_id: Uuid,
    pub existing_count: i64,
}
