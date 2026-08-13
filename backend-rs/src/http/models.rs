//! Model catalog HTTP handlers (mirror endpoints/models.py). The CivitAI lookup
//! endpoint is added with the civitai client (separate task).

use axum::extract::{Path, Query, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::catalog::{ModelDetail, ModelListResponse};
use crate::dto::civitai::CivitaiInfoResponse;
use crate::error::AppError;
use crate::{catalog, civitai};

#[derive(Debug, serde::Deserialize)]
pub struct ModelListQuery {
    pub q: Option<String>,
    pub model_type: Option<String>,
    pub min_count: Option<i64>,
    pub min_rating: Option<f64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/models
pub async fn get_models(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ModelListQuery>,
) -> Result<Json<ModelListResponse>, AppError> {
    let resp = catalog::models_list(
        &state.pool,
        q.q.as_deref(),
        q.model_type.as_deref(),
        q.min_count.unwrap_or(1).max(1),
        q.min_rating,
        q.sort_by.as_deref().unwrap_or("count"),
        q.sort_order.as_deref().unwrap_or("desc"),
        q.limit.unwrap_or(100).clamp(1, 500),
        q.offset.unwrap_or(0).max(0),
    )
    .await?;
    Ok(Json(resp))
}

/// GET /api/models/{model_name}/detail (model_name is the base name)
pub async fn get_model_detail(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(model_name): Path<String>,
) -> Result<Json<ModelDetail>, AppError> {
    Ok(Json(catalog::model_detail(&state.pool, &model_name).await?))
}

/// GET /api/models/{model_name}/civitai
pub async fn get_model_civitai(
    _user: CurrentUser,
    Path(model_name): Path<String>,
) -> Json<CivitaiInfoResponse> {
    let search_name = catalog::strip_extension(&catalog::extract_display_name(&model_name));
    match civitai::get_model_info(&search_name, "Checkpoint").await {
        Some(info) => Json(CivitaiInfoResponse {
            found: true,
            info: Some(info),
            error: None,
        }),
        None => Json(CivitaiInfoResponse {
            found: false,
            info: None,
            error: Some("Model not found on CivitAI".to_string()),
        }),
    }
}
