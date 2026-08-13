//! Export row DTO (mirror export_service.get_export_data). Field order is
//! significant: serde serializes a struct in declaration order, so the JSON
//! export keys (and the CSV column order) match the Python output.

#[derive(Debug, serde::Serialize)]
pub struct ExportRow {
    pub id: String,
    pub original_filename: String,
    pub source_tool: String,
    pub model_type: Option<String>,
    pub model_name: Option<String>,
    pub positive_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub sampler_name: Option<String>,
    pub scheduler: Option<String>,
    pub steps: Option<i32>,
    pub cfg_scale: Option<f64>,
    pub seed: Option<i64>,
    pub width: i32,
    pub height: i32,
    pub rating: i32,
    pub is_favorite: bool,
    /// Comma-joined user tags ("" when empty).
    pub user_tags: String,
    pub user_memo: Option<String>,
    /// RFC3339 timestamp.
    pub created_at: String,
}
