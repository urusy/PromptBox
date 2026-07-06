//! CivitAI model-info DTOs (mirror the Civitai* schemas in schemas/model.py).

#[derive(Debug, Clone, serde::Serialize)]
pub struct CivitaiImage {
    pub url: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub nsfw: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CivitaiRecommendedSettings {
    pub clip_skip: Option<i64>,
    pub steps: Option<i64>,
    pub cfg_scale: Option<f64>,
    pub sampler: Option<String>,
    pub vae: Option<String>,
    pub strength: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CivitaiVersionInfo {
    pub version_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub base_model: Option<String>,
    pub images: Vec<CivitaiImage>,
    pub recommended_settings: Option<CivitaiRecommendedSettings>,
    pub trigger_words: Vec<String>,
    pub download_url: Option<String>,
    pub file_size_kb: Option<f64>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CivitaiModelInfo {
    pub civitai_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub r#type: String,
    pub nsfw: bool,
    pub creator: Option<String>,
    pub civitai_url: Option<String>,
    pub is_exact_match: bool,
    pub versions: Vec<CivitaiVersionInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct CivitaiInfoResponse {
    pub found: bool,
    pub info: Option<CivitaiModelInfo>,
    pub error: Option<String>,
}
