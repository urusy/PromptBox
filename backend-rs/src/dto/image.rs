//! Image request/response DTOs.
//!
//! The responses are deliberately a SUPERSET that satisfies BOTH consumers
//! simultaneously:
//!
//!   - the current React frontend, which reads storage_path / thumbnail_path
//!     and a FLAT pagination ({total, page, per_page, total_pages}); and
//!   - Falcon's client.go / entity.go, which read original_url / thumbnail_url,
//!     a NESTED pagination ({pagination:{total_items, has_next, has_prev,...}}),
//!     and user_tags / needs_improvement on list items.
//!
//! Emitting both keeps the existing frontend working while repairing the Falcon
//! integration, which is currently broken by this drift.
//!
//! Fully-qualified `serde::` derives are used so there is no `use serde::...`
//! import to be flagged as unused.

use chrono::{DateTime, Utc};
use serde_json::Value;

/// Nested pagination object Falcon decodes.
#[derive(Debug, serde::Serialize)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

/// One entry of the image list. Superset of the current ImageListResponse and
/// Falcon's promptbox.Image.
#[derive(Debug, serde::Serialize)]
pub struct ImageListItem {
    pub id: String,
    pub source_tool: String,
    pub model_type: Option<String>,

    // Current frontend: relative storage paths.
    pub storage_path: String,
    pub thumbnail_path: String,
    // Falcon: absolute-from-root URL derived from the path above.
    pub thumbnail_url: String,

    pub width: i32,
    pub height: i32,
    pub model_name: Option<String>,
    pub rating: i32,
    pub is_favorite: bool,

    // Falcon reads these on list items; the current frontend tolerates extras.
    pub needs_improvement: bool,
    pub user_tags: Vec<String>,

    pub created_at: DateTime<Utc>,
}

/// Image list response carrying BOTH pagination shapes.
#[derive(Debug, serde::Serialize)]
pub struct ImageListResponse {
    pub items: Vec<ImageListItem>,

    // Flat fields (current frontend).
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,

    // Nested object (Falcon).
    pub pagination: Pagination,
}

/// Full single-image response. Superset of the current ImageResponse and
/// Falcon's promptbox.ImageDetail.
///
/// JSONB columns (loras/controlnets/embeddings/model_params/workflow_extras/
/// raw_metadata) pass through verbatim as serde_json::Value so no field is lost:
/// the current frontend needs weight_clip/hash on loras, while Falcon reads only
/// name/weight (and ignores extras). raw_metadata may be null.
#[derive(Debug, serde::Serialize)]
pub struct ImageDetail {
    pub id: String,
    pub source_tool: String,
    pub model_type: Option<String>,
    pub has_metadata: bool,

    pub original_filename: String,

    // Current frontend: relative storage paths.
    pub storage_path: String,
    pub thumbnail_path: String,
    // Falcon: absolute-from-root URLs derived from the paths above.
    pub original_url: String,
    pub thumbnail_url: String,

    pub file_hash: String,
    pub width: i32,
    pub height: i32,
    pub file_size_bytes: i64,

    pub positive_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub model_name: Option<String>,
    pub sampler_name: Option<String>,
    pub scheduler: Option<String>,
    pub steps: Option<i32>,
    // NOTE: cfg_scale is NUMERIC(5,2) in Postgres (rust_decimal::Decimal).
    // Converted to f64 here so it serializes as a JSON number (Falcon expects a
    // number). Whether the Python backend emits number vs string must be
    // confirmed against a production golden snapshot (Phase 0).
    pub cfg_scale: Option<f64>,
    pub seed: Option<i64>,

    pub loras: Value,
    pub controlnets: Value,
    pub embeddings: Value,
    pub model_params: Value,
    pub workflow_extras: Value,
    pub raw_metadata: Option<Value>,

    pub rating: i32,
    pub is_favorite: bool,
    pub needs_improvement: bool,
    pub user_tags: Vec<String>,
    pub user_memo: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    // Navigation context (null when no search context is supplied).
    pub prev_id: Option<String>,
    pub next_id: Option<String>,
}

/// PATCH body for a single image (mirrors Python ImageUpdate). All fields
/// optional: absent (None) means "leave unchanged".
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // consumed by the update endpoint (later task)
pub struct ImageUpdate {
    pub rating: Option<i32>,
    pub is_favorite: Option<bool>,
    pub needs_improvement: Option<bool>,
    pub user_tags: Option<Vec<String>>,
    pub user_memo: Option<String>,
}
