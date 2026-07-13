//! Bulk operation HTTP handlers (mirror endpoints/batch.py, prefix /bulk).

use axum::extract::State;
use axum::Json;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::batch;
use crate::dto::batch::{BatchDeleteRequest, BatchRestoreRequest, BatchUpdateRequest};
use crate::dto::common::MessageResponse;
use crate::error::AppError;

/// Mirror Field(min_length=1, max_length=500) on the id list.
fn validate_ids(ids: &[Uuid]) -> Result<(), AppError> {
    if ids.is_empty() {
        return Err(AppError::BadRequest(
            "ids must contain at least 1 item".to_string(),
        ));
    }
    if ids.len() > 500 {
        return Err(AppError::BadRequest(
            "ids must contain at most 500 items".to_string(),
        ));
    }
    Ok(())
}

/// POST /api/bulk/update
pub async fn batch_update(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<BatchUpdateRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    validate_ids(&req.ids)?;
    if let Some(r) = req.rating
        && !(0..=5).contains(&r)
    {
        return Err(AppError::BadRequest(
            "rating must be between 0 and 5".to_string(),
        ));
    }

    let count = batch::update(
        &state.pool,
        &req.ids,
        req.rating,
        req.is_favorite,
        req.needs_improvement,
        req.add_tags.as_deref(),
        req.remove_tags.as_deref(),
    )
    .await?;

    if count == 0 {
        return Err(AppError::NotFound("No images found".to_string()));
    }
    Ok(Json(MessageResponse::new(format!("Updated {count} images"))))
}

/// POST /api/bulk/delete
///
/// Permanent deletes also remove each image's original + thumbnail objects
/// from storage (DB rows first; object deletion is best-effort).
pub async fn batch_delete(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    validate_ids(&req.ids)?;
    let (count, action) = if req.permanent {
        let paths = batch::delete_permanent(&state.pool, &req.ids).await?;
        for (storage_path, thumbnail_path) in &paths {
            crate::storage::delete_image_objects(
                state.storage.as_ref(),
                storage_path,
                thumbnail_path,
            )
            .await;
        }
        (paths.len() as u64, "permanently deleted")
    } else {
        (
            batch::soft_delete(&state.pool, &req.ids).await?,
            "moved to trash",
        )
    };
    if count == 0 {
        return Err(AppError::NotFound("No images found".to_string()));
    }
    Ok(Json(MessageResponse::new(format!(
        "{count} images {action}"
    ))))
}

/// POST /api/bulk/restore
pub async fn batch_restore(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<BatchRestoreRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    validate_ids(&req.ids)?;
    let count = batch::restore(&state.pool, &req.ids).await?;
    if count == 0 {
        return Err(AppError::NotFound("No deleted images found".to_string()));
    }
    Ok(Json(MessageResponse::new(format!("Restored {count} images"))))
}
