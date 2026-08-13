//! Model and LoRA catalog DTOs (mirror schemas/model.py). These resources are
//! derived by aggregating the images table; there are no model/lora tables.

use std::collections::BTreeMap;

/// A named aggregate used for "top samplers / models / loras" lists.
#[derive(Debug, serde::Serialize)]
pub struct NamedStat {
    pub name: String,
    pub count: i64,
    pub avg_rating: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelListItem {
    /// Base model name (version suffix removed).
    pub name: String,
    pub display_name: String,
    pub model_type: Option<String>,
    pub image_count: i64,
    pub rated_count: i64,
    pub avg_rating: Option<f64>,
    pub high_rated_count: i64,
    pub version_count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelListResponse {
    pub items: Vec<ModelListItem>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelVersionStats {
    pub name: String,
    pub display_name: String,
    pub image_count: i64,
    pub rated_count: i64,
    pub avg_rating: Option<f64>,
    pub high_rated_count: i64,
    /// Rating histogram, always keyed 0..=5 (serializes as {"0": n, ...}).
    pub rating_distribution: BTreeMap<i32, i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelDetail {
    pub name: String,
    pub display_name: String,
    pub model_type: Option<String>,
    pub image_count: i64,
    pub rated_count: i64,
    pub avg_rating: Option<f64>,
    pub high_rated_count: i64,
    pub rating_distribution: BTreeMap<i32, i64>,
    pub top_samplers: Vec<NamedStat>,
    pub top_loras: Vec<NamedStat>,
    pub versions: Vec<ModelVersionStats>,
}

#[derive(Debug, serde::Serialize)]
pub struct LoraListItem {
    pub name: String,
    pub display_name: String,
    pub hash: Option<String>,
    pub image_count: i64,
    pub rated_count: i64,
    pub avg_rating: Option<f64>,
    pub high_rated_count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct LoraListResponse {
    pub items: Vec<LoraListItem>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct LoraDetail {
    pub name: String,
    pub display_name: String,
    pub hash: Option<String>,
    pub image_count: i64,
    pub rated_count: i64,
    pub avg_rating: Option<f64>,
    pub high_rated_count: i64,
    pub rating_distribution: BTreeMap<i32, i64>,
    pub avg_weight: Option<f64>,
    pub top_models: Vec<NamedStat>,
    pub top_samplers: Vec<NamedStat>,
}
