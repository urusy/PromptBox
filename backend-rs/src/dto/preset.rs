//! Search preset DTOs (mirror schemas/search_preset.py).
//!
//! `SearchFilters` is the saved filter set shared by search presets and smart
//! folders. Every field is optional; `None` fields are omitted on serialization
//! (mirroring Pydantic's `model_dump(exclude_none=True)` on store), and unknown
//! keys in the request body are ignored (serde default), matching the Python
//! schema's drop-unknown behaviour.

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampler_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs_improvement: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lora_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_xyz_grid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_upscaled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchPresetCreate {
    pub name: String,
    pub filters: SearchFilters,
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchPresetUpdate {
    pub name: Option<String>,
    pub filters: Option<SearchFilters>,
}

#[derive(Debug, serde::Serialize)]
pub struct SearchPresetResponse {
    pub id: Uuid,
    pub name: String,
    pub filters: SearchFilters,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
