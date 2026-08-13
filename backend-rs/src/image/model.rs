//! Database row mapping for the `images` table and conversion to DTOs.

use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::Value;
use uuid::Uuid;

use crate::dto::image::{ImageDetail, ImageListItem};

/// One row of the `images` table. Column names match struct fields (sqlx
/// FromRow maps by name). Postgres types: rating is SMALLINT (i16), cfg_scale
/// is NUMERIC (Decimal), JSONB columns map to serde_json::Value.
#[derive(Debug, sqlx::FromRow)]
pub struct ImageRow {
    pub id: Uuid,
    pub source_tool: String,
    pub model_type: Option<String>,
    pub has_metadata: bool,
    pub original_filename: String,
    pub storage_path: String,
    pub thumbnail_path: String,
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
    pub cfg_scale: Option<Decimal>,
    pub seed: Option<i64>,
    pub loras: Value,
    pub controlnets: Value,
    pub embeddings: Value,
    pub model_params: Value,
    pub workflow_extras: Value,
    pub raw_metadata: Option<Value>,
    pub rating: i16,
    pub is_favorite: bool,
    pub needs_improvement: bool,
    pub user_tags: Value,
    pub user_memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Build a from-root storage URL (Falcon reads original_url/thumbnail_url).
pub fn storage_url(path: &str) -> String {
    format!("/storage/{path}")
}

/// Decode a JSONB string array (user_tags) into Vec<String>, defaulting to empty.
fn tags_to_vec(v: Value) -> Vec<String> {
    serde_json::from_value(v).unwrap_or_default()
}

impl ImageRow {
    /// Convert to the full detail DTO. prev_id/next_id come from the search
    /// navigation context (None when not applicable).
    pub fn into_detail(self, prev_id: Option<String>, next_id: Option<String>) -> ImageDetail {
        let original_url = storage_url(&self.storage_path);
        let thumbnail_url = storage_url(&self.thumbnail_path);
        let user_tags = tags_to_vec(self.user_tags);
        let cfg_scale = self.cfg_scale.and_then(|d| d.to_f64());

        ImageDetail {
            id: self.id.to_string(),
            source_tool: self.source_tool,
            model_type: self.model_type,
            has_metadata: self.has_metadata,
            original_filename: self.original_filename,
            storage_path: self.storage_path,
            thumbnail_path: self.thumbnail_path,
            original_url,
            thumbnail_url,
            file_hash: self.file_hash,
            width: self.width,
            height: self.height,
            file_size_bytes: self.file_size_bytes,
            positive_prompt: self.positive_prompt,
            negative_prompt: self.negative_prompt,
            model_name: self.model_name,
            sampler_name: self.sampler_name,
            scheduler: self.scheduler,
            steps: self.steps,
            cfg_scale,
            seed: self.seed,
            loras: self.loras,
            controlnets: self.controlnets,
            embeddings: self.embeddings,
            model_params: self.model_params,
            workflow_extras: self.workflow_extras,
            raw_metadata: self.raw_metadata,
            rating: self.rating as i32,
            is_favorite: self.is_favorite,
            needs_improvement: self.needs_improvement,
            user_tags,
            user_memo: self.user_memo,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            prev_id,
            next_id,
        }
    }

    /// Convert to a list item DTO (subset used in image listings).
    pub fn into_list_item(self) -> ImageListItem {
        let thumbnail_url = storage_url(&self.thumbnail_path);
        let user_tags = tags_to_vec(self.user_tags);

        ImageListItem {
            id: self.id.to_string(),
            source_tool: self.source_tool,
            model_type: self.model_type,
            storage_path: self.storage_path,
            thumbnail_path: self.thumbnail_path,
            thumbnail_url,
            width: self.width,
            height: self.height,
            model_name: self.model_name,
            rating: self.rating as i32,
            is_favorite: self.is_favorite,
            needs_improvement: self.needs_improvement,
            user_tags,
            created_at: self.created_at,
        }
    }
}
