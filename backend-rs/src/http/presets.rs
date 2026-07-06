//! Search preset HTTP handlers (mirror endpoints/search_presets.py).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::common::{validate_name, MessageResponse};
use crate::dto::preset::{SearchPresetCreate, SearchPresetResponse, SearchPresetUpdate};
use crate::error::AppError;
use crate::preset;

/// GET /api/search-presets
pub async fn list_presets(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SearchPresetResponse>>, AppError> {
    Ok(Json(preset::list(&state.pool).await?))
}

/// POST /api/search-presets
pub async fn create_preset(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<SearchPresetCreate>,
) -> Result<(StatusCode, Json<SearchPresetResponse>), AppError> {
    validate_name(&body.name).map_err(AppError::BadRequest)?;
    let resp = preset::create(&state.pool, &body.name, &body.filters).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

/// PUT /api/search-presets/{id}
pub async fn update_preset(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<SearchPresetUpdate>,
) -> Result<Json<SearchPresetResponse>, AppError> {
    if let Some(name) = &body.name {
        validate_name(name).map_err(AppError::BadRequest)?;
    }
    let resp = preset::update(&state.pool, id, body.name.as_deref(), body.filters.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("Search preset not found".to_string()))?;
    Ok(Json(resp))
}

/// DELETE /api/search-presets/{id}
pub async fn delete_preset(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    if !preset::delete(&state.pool, id).await? {
        return Err(AppError::NotFound("Search preset not found".to_string()));
    }
    Ok(Json(MessageResponse::new(
        "Search preset deleted successfully",
    )))
}
