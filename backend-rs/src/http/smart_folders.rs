//! Smart folder HTTP handlers (mirror endpoints/smart_folders.py).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::common::{validate_name, MessageResponse};
use crate::dto::smart_folder::{SmartFolderCreate, SmartFolderResponse, SmartFolderUpdate};
use crate::error::AppError;
use crate::smart_folder;

/// GET /api/smart-folders
pub async fn list_folders(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SmartFolderResponse>>, AppError> {
    Ok(Json(smart_folder::list(&state.pool).await?))
}

/// POST /api/smart-folders
pub async fn create_folder(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<SmartFolderCreate>,
) -> Result<(StatusCode, Json<SmartFolderResponse>), AppError> {
    validate_name(&body.name).map_err(AppError::BadRequest)?;
    let resp = smart_folder::create(&state.pool, &body.name, body.icon.as_deref(), &body.filters)
        .await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

/// GET /api/smart-folders/{id}
pub async fn get_folder(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SmartFolderResponse>, AppError> {
    let resp = smart_folder::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Smart folder not found".to_string()))?;
    Ok(Json(resp))
}

/// PUT /api/smart-folders/{id}
pub async fn update_folder(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<SmartFolderUpdate>,
) -> Result<Json<SmartFolderResponse>, AppError> {
    if let Some(name) = &body.name {
        validate_name(name).map_err(AppError::BadRequest)?;
    }
    let resp = smart_folder::update(
        &state.pool,
        id,
        body.name.as_deref(),
        body.icon.as_deref(),
        body.filters.as_ref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Smart folder not found".to_string()))?;
    Ok(Json(resp))
}

/// DELETE /api/smart-folders/{id}
pub async fn delete_folder(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    if !smart_folder::delete(&state.pool, id).await? {
        return Err(AppError::NotFound("Smart folder not found".to_string()));
    }
    Ok(Json(MessageResponse::new(
        "Smart folder deleted successfully",
    )))
}
