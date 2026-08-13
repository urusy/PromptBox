//! Duplicate-file management HTTP handlers (mirror endpoints/duplicates.py).

use std::io;

use axum::extract::{Path, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::duplicate::{DeleteFileResult, DeleteResult, DuplicatesInfo};
use crate::duplicate;
use crate::error::AppError;

/// GET /api/duplicates
pub async fn get_info(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<DuplicatesInfo>, AppError> {
    Ok(Json(duplicate::info(&state.config.import_path).await?))
}

/// DELETE /api/duplicates
pub async fn delete_all(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<DeleteResult>, AppError> {
    Ok(Json(duplicate::delete_all(&state.config.import_path).await?))
}

/// DELETE /api/duplicates/{filename}
pub async fn delete_file(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<DeleteFileResult>, AppError> {
    if !duplicate::is_safe_filename(&filename) {
        return Err(AppError::BadRequest("Invalid filename".to_string()));
    }
    match duplicate::delete_one(&state.config.import_path, &filename).await {
        Ok(Some(freed_bytes)) => Ok(Json(DeleteFileResult {
            deleted: filename,
            freed_bytes,
        })),
        Ok(None) => Err(AppError::NotFound("File not found".to_string())),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            Err(AppError::Forbidden("Permission denied".to_string()))
        }
        Err(e) => Err(e.into()),
    }
}
