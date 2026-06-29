//! Image read endpoints.

use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::image::{ImageDetail, ImageListItem, ImageListResponse, Pagination};
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
pub async fn get_image(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ImageDetail>, AppError> {
    let row = image::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;
    Ok(Json(row.into_detail(None, None)))
}
