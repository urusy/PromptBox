//! Showcase HTTP handlers (mirror endpoints/showcases.py).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::common::{validate_name, MessageResponse};
use crate::dto::showcase::{
    ShowcaseCreate, ShowcaseDetailResponse, ShowcaseImageCheckResult, ShowcaseImageIds,
    ShowcaseResponse, ShowcaseUpdate,
};
use crate::error::AppError;
use crate::showcase;

/// Mirror Field(min_length=1[, max_length=max]) on an image-id list.
fn validate_image_ids(ids: &[Uuid], max: Option<usize>) -> Result<(), AppError> {
    if ids.is_empty() {
        return Err(AppError::BadRequest(
            "image_ids must contain at least 1 item".to_string(),
        ));
    }
    if let Some(max) = max
        && ids.len() > max
    {
        return Err(AppError::BadRequest(format!(
            "image_ids must contain at most {max} items"
        )));
    }
    Ok(())
}

/// GET /api/showcases
pub async fn list_showcases(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ShowcaseResponse>>, AppError> {
    Ok(Json(showcase::list(&state.pool).await?))
}

/// POST /api/showcases/check-images
pub async fn check_images(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<ShowcaseImageIds>,
) -> Result<Json<Vec<ShowcaseImageCheckResult>>, AppError> {
    validate_image_ids(&body.image_ids, Some(100))?;
    Ok(Json(
        showcase::check_images(&state.pool, &body.image_ids).await?,
    ))
}

/// POST /api/showcases
pub async fn create_showcase(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<ShowcaseCreate>,
) -> Result<(StatusCode, Json<ShowcaseResponse>), AppError> {
    validate_name(&body.name).map_err(AppError::BadRequest)?;
    let resp = showcase::create(
        &state.pool,
        &body.name,
        body.description.as_deref(),
        body.icon.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

/// GET /api/showcases/{id}
pub async fn get_showcase(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ShowcaseDetailResponse>, AppError> {
    let resp = showcase::get_detail(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Showcase not found".to_string()))?;
    Ok(Json(resp))
}

/// PUT /api/showcases/{id}
pub async fn update_showcase(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ShowcaseUpdate>,
) -> Result<Json<ShowcaseResponse>, AppError> {
    if let Some(name) = &body.name {
        validate_name(name).map_err(AppError::BadRequest)?;
    }
    let resp = showcase::update(
        &state.pool,
        id,
        body.name.as_deref(),
        body.description.as_deref(),
        body.icon.as_deref(),
        body.cover_image_id,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Showcase not found".to_string()))?;
    Ok(Json(resp))
}

/// DELETE /api/showcases/{id}
pub async fn delete_showcase(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    if !showcase::delete(&state.pool, id).await? {
        return Err(AppError::NotFound("Showcase not found".to_string()));
    }
    Ok(Json(MessageResponse::new("Showcase deleted successfully")))
}

/// POST /api/showcases/{id}/images
pub async fn add_images(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ShowcaseImageIds>,
) -> Result<Json<MessageResponse>, AppError> {
    validate_image_ids(&body.image_ids, Some(100))?;
    let added = showcase::add_images(&state.pool, id, &body.image_ids)
        .await?
        .ok_or_else(|| AppError::NotFound("Showcase not found".to_string()))?;
    Ok(Json(MessageResponse::new(format!(
        "Added {added} images to showcase"
    ))))
}

/// DELETE /api/showcases/{id}/images
pub async fn remove_images(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ShowcaseImageIds>,
) -> Result<Json<MessageResponse>, AppError> {
    validate_image_ids(&body.image_ids, Some(100))?;
    let removed = showcase::remove_images(&state.pool, id, &body.image_ids)
        .await?
        .ok_or_else(|| AppError::NotFound("Showcase not found".to_string()))?;
    Ok(Json(MessageResponse::new(format!(
        "Removed {removed} images from showcase"
    ))))
}

/// PUT /api/showcases/{id}/images/reorder
pub async fn reorder_images(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ShowcaseImageIds>,
) -> Result<Json<MessageResponse>, AppError> {
    validate_image_ids(&body.image_ids, None)?;
    showcase::reorder_images(&state.pool, id, &body.image_ids)
        .await?
        .ok_or_else(|| AppError::NotFound("Showcase not found".to_string()))?;
    Ok(Json(MessageResponse::new("Images reordered successfully")))
}
