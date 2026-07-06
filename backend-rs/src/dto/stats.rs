//! Statistics DTOs (mirror the inline Pydantic models in endpoints/stats.py).

#[derive(Debug, serde::Serialize)]
pub struct CountItem {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct TimeSeriesItem {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct RatingDistribution {
    pub rating: i32,
    pub count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct StatsOverview {
    pub total_images: i64,
    pub total_favorites: i64,
    pub total_rated: i64,
    pub total_unrated: i64,
    pub avg_rating: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
pub struct StatsResponse {
    pub overview: StatsOverview,
    pub by_model_type: Vec<CountItem>,
    pub by_source_tool: Vec<CountItem>,
    pub by_model_name: Vec<CountItem>,
    pub by_sampler: Vec<CountItem>,
    pub by_lora: Vec<CountItem>,
    pub by_rating: Vec<RatingDistribution>,
    pub daily_counts: Vec<TimeSeriesItem>,
    pub daily_updates: Vec<TimeSeriesItem>,
}

#[derive(Debug, serde::Serialize)]
pub struct RatingAnalysisItem {
    pub name: String,
    pub avg_rating: f64,
    pub count: i64,
    pub high_rated_count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct RatingAnalysisResponse {
    pub by_model: Vec<RatingAnalysisItem>,
    pub by_sampler: Vec<RatingAnalysisItem>,
    pub by_lora: Vec<RatingAnalysisItem>,
    pub by_steps: Vec<RatingAnalysisItem>,
    pub by_cfg: Vec<RatingAnalysisItem>,
    pub filtered_by_model: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelListResponse {
    pub models: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct LoraListResponse {
    pub loras: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SamplerListResponse {
    pub samplers: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelRatingDistributionItem {
    pub model_name: String,
    pub rating_0: i64,
    pub rating_1: i64,
    pub rating_2: i64,
    pub rating_3: i64,
    pub rating_4: i64,
    pub rating_5: i64,
    pub total: i64,
    pub avg_rating: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelRatingDistributionResponse {
    pub items: Vec<ModelRatingDistributionItem>,
}
