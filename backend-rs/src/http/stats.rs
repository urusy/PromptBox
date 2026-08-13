//! Statistics HTTP handlers (mirror endpoints/stats.py), including its TTL
//! caches: stats / rating-analysis / model-rating-distribution are cached for
//! 600s (module-level, like the Python TTLCache globals). Expiry by TTL only —
//! the Python invalidate helpers were never called, so they are not ported.

use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::cache::TtlCache;
use crate::dto::stats::{
    LoraListResponse, ModelListResponse, ModelRatingDistributionResponse, RatingAnalysisResponse,
    SamplerListResponse, StatsResponse,
};
use crate::error::AppError;
use crate::stats;

static STATS_CACHE: LazyLock<TtlCache<StatsResponse>> =
    LazyLock::new(|| TtlCache::new(Duration::from_secs(600), 10));
static RATING_CACHE: LazyLock<TtlCache<RatingAnalysisResponse>> =
    LazyLock::new(|| TtlCache::new(Duration::from_secs(600), 50));
static DIST_CACHE: LazyLock<TtlCache<ModelRatingDistributionResponse>> =
    LazyLock::new(|| TtlCache::new(Duration::from_secs(600), 10));

#[derive(Debug, serde::Deserialize)]
pub struct StatsQuery {
    pub days: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct MinCountQuery {
    pub min_count: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RatingAnalysisQuery {
    pub min_count: Option<i64>,
    pub model_name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ModelRatingDistQuery {
    pub min_count: Option<i64>,
    pub limit: Option<i64>,
}

/// GET /api/stats?days=30
pub async fn get_stats(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<StatsResponse>, AppError> {
    let days = q.days.unwrap_or(30);
    let key = format!("stats:{days}");
    if let Some(cached) = STATS_CACHE.get(&key) {
        return Ok(Json(cached));
    }
    let resp = stats::get_stats(&state.pool, days).await?;
    STATS_CACHE.insert(key, resp.clone());
    Ok(Json(resp))
}

/// GET /api/stats/models-for-analysis?min_count=5
pub async fn models_for_analysis(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<MinCountQuery>,
) -> Result<Json<ModelListResponse>, AppError> {
    let models = stats::models_for_analysis(&state.pool, q.min_count.unwrap_or(5)).await?;
    Ok(Json(ModelListResponse { models }))
}

/// GET /api/stats/loras-for-filter?min_count=1
pub async fn loras_for_filter(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<MinCountQuery>,
) -> Result<Json<LoraListResponse>, AppError> {
    let loras = stats::loras_for_filter(&state.pool, q.min_count.unwrap_or(1)).await?;
    Ok(Json(LoraListResponse { loras }))
}

/// GET /api/stats/samplers-for-filter?min_count=1
pub async fn samplers_for_filter(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<MinCountQuery>,
) -> Result<Json<SamplerListResponse>, AppError> {
    let samplers = stats::samplers_for_filter(&state.pool, q.min_count.unwrap_or(1)).await?;
    Ok(Json(SamplerListResponse { samplers }))
}

/// GET /api/stats/rating-analysis?min_count=5&model_name=
pub async fn rating_analysis(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<RatingAnalysisQuery>,
) -> Result<Json<RatingAnalysisResponse>, AppError> {
    // Treat an empty model_name as "no filter" (Python's `if model_name`).
    let model = q.model_name.as_deref().filter(|s| !s.is_empty());
    let min_count = q.min_count.unwrap_or(5);
    let key = format!("rating_analysis:{min_count}:{}", model.unwrap_or("all"));
    if let Some(cached) = RATING_CACHE.get(&key) {
        return Ok(Json(cached));
    }
    let resp = stats::rating_analysis(&state.pool, min_count, model).await?;
    RATING_CACHE.insert(key, resp.clone());
    Ok(Json(resp))
}

/// GET /api/stats/model-rating-distribution?min_count=10&limit=15
pub async fn model_rating_distribution(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ModelRatingDistQuery>,
) -> Result<Json<ModelRatingDistributionResponse>, AppError> {
    let min_count = q.min_count.unwrap_or(10);
    let limit = q.limit.unwrap_or(15);
    let key = format!("model_rating_dist:{min_count}:{limit}");
    if let Some(cached) = DIST_CACHE.get(&key) {
        return Ok(Json(cached));
    }
    let items = stats::model_rating_distribution(&state.pool, min_count, limit).await?;
    let resp = ModelRatingDistributionResponse { items };
    DIST_CACHE.insert(key, resp.clone());
    Ok(Json(resp))
}
