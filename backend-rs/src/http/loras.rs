//! LoRA catalog HTTP handlers (mirror endpoints/loras.py). The CivitAI lookup
//! endpoint is added with the civitai client (separate task).

use axum::extract::{Path, Query, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::catalog::{LoraDetail, LoraListResponse};
use crate::dto::civitai::CivitaiInfoResponse;
use crate::error::AppError;
use crate::{catalog, civitai};

#[derive(Debug, serde::Deserialize)]
pub struct LoraListQuery {
    pub q: Option<String>,
    pub min_count: Option<i64>,
    pub min_rating: Option<f64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/loras
pub async fn get_loras(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<LoraListQuery>,
) -> Result<Json<LoraListResponse>, AppError> {
    let resp = catalog::loras_list(
        &state.pool,
        q.q.as_deref(),
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

/// GET /api/loras/{lora_name}/detail
pub async fn get_lora_detail(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(lora_name): Path<String>,
) -> Result<Json<LoraDetail>, AppError> {
    Ok(Json(catalog::lora_detail(&state.pool, &lora_name).await?))
}

/// GET /api/loras/{lora_name}/civitai
///
/// Tries a hash lookup first (most accurate), then falls back to name search.
pub async fn get_lora_civitai(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(lora_name): Path<String>,
) -> Result<Json<CivitaiInfoResponse>, AppError> {
    if let Some(hash) = catalog::lora_hash(&state.pool, &lora_name).await?
        && let Some(info) = civitai::get_model_by_hash(&hash).await
    {
        return Ok(Json(CivitaiInfoResponse {
            found: true,
            info: Some(info),
            error: None,
        }));
    }

    let search_name = catalog::strip_extension(&catalog::extract_display_name(&lora_name));
    let resp = match civitai::get_model_info(&search_name, "LORA").await {
        Some(info) => CivitaiInfoResponse {
            found: true,
            info: Some(info),
            error: None,
        },
        None => CivitaiInfoResponse {
            found: false,
            info: None,
            error: Some("LoRA not found on CivitAI".to_string()),
        },
    };
    Ok(Json(resp))
}
