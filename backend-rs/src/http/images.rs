//! Image read endpoints.

use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::common::MessageResponse;
use crate::dto::image::{ImageDetail, ImageListItem, ImageListResponse, ImageUpdate, Pagination};
use crate::error::AppError;
use crate::image::{self, SearchParams};

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    24
}

/// Split a comma-separated query value into trimmed, non-empty parts.
fn split_csv(v: &str) -> Vec<String> {
    v.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Query parameters for the list endpoint. Accepts the current frontend names
/// (sort_by/sort_order) and Falcon's aliases (sort/order). `tags` is a
/// comma-separated string (Falcon sends a single string).
#[derive(Debug, serde::Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,

    pub source_tool: Option<String>,
    pub model_type: Option<String>,

    pub min_rating: Option<i16>,
    pub exact_rating: Option<i16>,
    pub max_rating: Option<i16>, // Falcon
    pub is_favorite: Option<bool>,
    pub needs_improvement: Option<bool>,

    pub model_name: Option<String>,
    pub sampler_name: Option<String>,
    pub file_type: Option<String>,

    pub tags: Option<String>,
    pub lora_name: Option<String>,

    // Advanced filters (current React frontend).
    pub q: Option<String>,
    pub is_xyz_grid: Option<bool>,
    pub is_upscaled: Option<bool>,
    pub orientation: Option<String>,
    pub min_width: Option<i32>,
    pub min_height: Option<i32>,
    pub date_from: Option<String>,
    pub seed: Option<i64>,
    pub seed_tolerance: Option<i64>,
    pub showcase_id: Option<Uuid>,

    pub include_deleted: Option<bool>,

    // Current frontend names.
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    // Falcon aliases.
    pub sort: Option<String>,
    pub order: Option<String>,
}

impl ListQuery {
    fn into_params(self, page: i64, per_page: i64) -> SearchParams {
        SearchParams {
            source_tool: self.source_tool,
            model_type: self.model_type,
            min_rating: self.min_rating,
            exact_rating: self.exact_rating,
            max_rating: self.max_rating,
            is_favorite: self.is_favorite,
            needs_improvement: self.needs_improvement,
            model_name: self.model_name,
            sampler_name: self.sampler_name,
            file_type: self.file_type,
            tags: self.tags.as_deref().map(split_csv).unwrap_or_default(),
            lora_name: self.lora_name,
            q: self.q,
            is_xyz_grid: self.is_xyz_grid,
            is_upscaled: self.is_upscaled,
            orientation: self.orientation,
            min_width: self.min_width,
            min_height: self.min_height,
            date_from: self.date_from,
            seed: self.seed,
            seed_tolerance: self.seed_tolerance,
            showcase_id: self.showcase_id,
            include_deleted: self.include_deleted.unwrap_or(false),
            page,
            per_page,
            sort_by: self
                .sort_by
                .or(self.sort)
                .unwrap_or_else(|| "created_at".to_string()),
            sort_order: self
                .sort_order
                .or(self.order)
                .unwrap_or_else(|| "desc".to_string()),
        }
    }
}

/// GET /api/images
pub async fn list_images(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ImageListResponse>, AppError> {
    let page = q.page.max(1);
    let per_page = q.per_page.clamp(1, 120);
    let params = q.into_params(page, per_page);

    let result = image::list(&state.pool, &params).await?;
    let total = result.total;
    // i64::div_ceil is still unstable (int_roundings); per_page is clamped >= 1.
    let total_pages = (total + per_page - 1) / per_page;

    let items: Vec<ImageListItem> = result
        .items
        .into_iter()
        .map(|r| r.into_list_item())
        .collect();

    Ok(Json(ImageListResponse {
        items,
        total,
        page,
        per_page,
        total_pages,
        pagination: Pagination {
            page,
            per_page,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
    }))
}

/// GET /api/images/{id}
///
/// prev/next are computed within the search context supplied as query params
/// (same filters/sort as the listing), mirroring get_image_with_neighbors.
pub async fn get_image(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ImageDetail>, AppError> {
    let row = image::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;
    // page/per_page are unused when computing neighbors.
    let params = q.into_params(1, 1);
    let (prev, next) = image::neighbors(&state.pool, &params, id).await?;
    Ok(Json(row.into_detail(
        prev.map(|u| u.to_string()),
        next.map(|u| u.to_string()),
    )))
}

/// PATCH /api/images/{id}
///
/// Partial update of user-editable metadata. Returns the updated image without
/// prev/next navigation context (no search params are involved), matching the
/// Python endpoint.
pub async fn update_image(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ImageUpdate>,
) -> Result<Json<ImageDetail>, AppError> {
    // Mirror Pydantic's Field(ge=0, le=5) on rating; the DB CHECK constraint is
    // the backstop, but rejecting early yields a clean 400 instead of a 500.
    if let Some(r) = body.rating
        && !(0..=5).contains(&r)
    {
        return Err(AppError::BadRequest(
            "rating must be between 0 and 5".to_string(),
        ));
    }

    let row = image::update(&state.pool, id, &body)
        .await?
        .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;
    Ok(Json(row.into_detail(None, None)))
}

/// Query for DELETE: `?permanent=true` performs a physical delete.
#[derive(Debug, serde::Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub permanent: bool,
}

/// DELETE /api/images/{id}
///
/// Soft delete by default (sets `deleted_at`), or physical delete with
/// `?permanent=true`.
pub async fn delete_image(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeleteQuery>,
) -> Result<Json<MessageResponse>, AppError> {
    let deleted = image::delete(&state.pool, id, q.permanent).await?;
    if !deleted {
        return Err(AppError::NotFound("Image not found".to_string()));
    }
    let action = if q.permanent {
        "permanently deleted"
    } else {
        "moved to trash"
    };
    Ok(Json(MessageResponse::new(format!("Image {action}"))))
}

/// POST /api/images/{id}/restore
///
/// Restore a soft-deleted image. 404 if it does not exist or is not deleted.
pub async fn restore_image(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let restored = image::restore(&state.pool, id).await?;
    if !restored {
        return Err(AppError::NotFound(
            "Image not found or not deleted".to_string(),
        ));
    }
    Ok(Json(MessageResponse::new("Image restored")))
}
