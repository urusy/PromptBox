//! Image read endpoints.

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::HeaderMap;
use axum::Json;
use chrono::Duration;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::warnings::{self, Warnings};
use super::AppState;
use crate::dto::common::MessageResponse;
use crate::dto::grid::{
    GridAxes, GridAxis, GridAxisValues, GridMember, GridMembersResponse, GridPosition,
};
use crate::dto::image::{ImageDetail, ImageListItem, ImageListResponse, ImageUpdate, Pagination};
use crate::error::AppError;
use crate::image::{self, grid, SearchParams};
use crate::storage;

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

    /// Reject the request (400) instead of silently ignoring anything the
    /// server did not understand. Intended for client CI/staging (docs/13 A3).
    pub strict: Option<bool>,
}

impl ListQuery {
    /// Every parameter this endpoint understands. **Keep in sync with the
    /// struct above** — an entry missing here makes a valid parameter look like
    /// a typo; an extra entry hides a real one.
    pub const KNOWN_PARAMS: &'static [&'static str] = &[
        "page",
        "per_page",
        "source_tool",
        "model_type",
        "min_rating",
        "exact_rating",
        "max_rating",
        "is_favorite",
        "needs_improvement",
        "model_name",
        "sampler_name",
        "file_type",
        "tags",
        "lora_name",
        "q",
        "is_xyz_grid",
        "is_upscaled",
        "orientation",
        "min_width",
        "min_height",
        "date_from",
        "seed",
        "seed_tolerance",
        "showcase_id",
        "include_deleted",
        "sort_by",
        "sort_order",
        "sort",
        "order",
        "strict",
    ];
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

/// Collect everything about this request that will not be honoured literally:
/// unknown parameters, out-of-range page/per_page, an unsupported sort column.
fn collect_warnings(raw_query: Option<&str>, q: &ListQuery, page: i64, per_page: i64) -> Warnings {
    let mut warnings = Warnings::default();

    for key in warnings::unknown_params(raw_query, ListQuery::KNOWN_PARAMS) {
        warnings.unknown_param(&key, ListQuery::KNOWN_PARAMS);
    }
    if q.page != page {
        warnings.clamped("page", q.page, page);
    }
    if q.per_page != per_page {
        warnings.clamped("per_page", q.per_page, per_page);
    }
    // sort/order are Falcon's aliases for the same thing.
    if let Some(requested) = q.sort_by.as_deref().or(q.sort.as_deref())
        && !image::ALLOWED_SORT_COLUMNS.contains(&requested)
    {
        warnings.fallback(
            "sort_by",
            requested,
            "created_at",
            image::ALLOWED_SORT_COLUMNS,
        );
    }

    warnings
}

/// GET /api/images
pub async fn list_images(
    _user: CurrentUser,
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
    Query(q): Query<ListQuery>,
) -> Result<(HeaderMap, Json<ImageListResponse>), AppError> {
    let page = q.page.max(1);
    let per_page = q.per_page.clamp(1, 120);

    let warnings = collect_warnings(raw_query.as_deref(), &q, page, per_page);
    if q.strict.unwrap_or(false) && !warnings.is_empty() {
        return Err(AppError::BadRequest(format!(
            "strict mode: {}",
            warnings.summary()
        )));
    }
    let headers = warnings.headers();

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

    Ok((
        headers,
        Json(ImageListResponse {
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
            warnings: warnings.into_vec(),
        }),
    ))
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

/// Default matching window. Wide enough that a grid which took an hour to
/// render still reaches its first cells; narrow enough that yesterday's run with
/// the same settings does not bleed in.
const DEFAULT_GRID_WINDOW_HOURS: i64 = 24;
const MAX_GRID_WINDOW_HOURS: i64 = 24 * 30;

/// Query for the grid members endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct GridMembersQuery {
    /// How far back from the grid to look for cells.
    pub window_hours: Option<i64>,
    /// Reject (400) instead of reporting `warnings[]` (docs/13 A3).
    pub strict: Option<bool>,
}

impl GridMembersQuery {
    /// **Keep in sync with the struct above** (see `ListQuery::KNOWN_PARAMS`).
    pub const KNOWN_PARAMS: &'static [&'static str] = &["window_hours", "strict"];
}

/// Convert the matcher's axes into the response shape (x/y/z slots).
fn axes_dto(axes: &[grid::Axis]) -> GridAxes {
    let mut dto = GridAxes::default();
    for axis in axes {
        let slot = Some(GridAxis {
            axis_type: axis.axis_type.clone(),
            values: axis.values.clone(),
            column: axis.column.map(|c| c.as_str()),
        });
        match axis.name {
            "x" => dto.x = slot,
            "y" => dto.y = slot,
            _ => dto.z = slot,
        }
    }
    dto
}

/// GET /api/images/{id}/grid-members
///
/// The images an XYZ grid was assembled from. Neither A1111 nor the grid file
/// records that link, so membership is **inferred** — see `image::grid` — and
/// the response says how confident the answer is.
pub async fn grid_members(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    RawQuery(raw_query): RawQuery,
    Query(q): Query<GridMembersQuery>,
) -> Result<(HeaderMap, Json<GridMembersResponse>), AppError> {
    let row = image::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;
    if !grid::is_grid(&row.model_params) {
        return Err(AppError::BadRequest(
            "image is not a grid; this endpoint applies to XYZ grid images only".to_string(),
        ));
    }

    let mut warnings = Warnings::default();
    for key in warnings::unknown_params(raw_query.as_deref(), GridMembersQuery::KNOWN_PARAMS) {
        warnings.unknown_param(&key, GridMembersQuery::KNOWN_PARAMS);
    }
    let requested = q.window_hours.unwrap_or(DEFAULT_GRID_WINDOW_HOURS);
    let window_hours = requested.clamp(1, MAX_GRID_WINDOW_HOURS);
    if requested != window_hours {
        warnings.clamped("window_hours", requested, window_hours);
    }

    let axes = grid::axes_of(&row.model_params);
    for axis in &axes {
        if axis.column.is_none() {
            warnings.note(
                "unsupported_axis_type",
                &format!("xyz_{}_type", axis.name),
                format!(
                    "axis type {:?} is not one this matcher understands; members were narrowed \
                     by the remaining parameters only",
                    axis.axis_type
                ),
            );
        }
    }
    if axes.is_empty() {
        warnings.note(
            "no_axis_metadata",
            "model_params",
            "this grid records no X/Y/Z axis metadata, so its member images cannot be identified"
                .to_string(),
        );
    }

    // With no axes there is nothing to match on — every image in the window
    // would qualify, which is worse than admitting the answer is unknown.
    let rows = if axes.is_empty() {
        Vec::new()
    } else {
        grid::find_members(&state.pool, &row, &axes, Duration::hours(window_hours)).await?
    };

    let mut placed: Vec<_> = rows
        .into_iter()
        .map(|r| (grid::place(&r, &axes), r))
        .collect();
    // Reading order of the montage: pages (z), then rows (y), then columns (x).
    placed.sort_by_key(|(p, _)| (p.index[2], p.index[1], p.index[0]));

    let matched = placed.len();
    if matched as i64 >= grid::MAX_MEMBERS {
        warnings.note(
            "truncated",
            "members",
            format!("stopped after {} candidates; widen nothing, narrow the window instead", grid::MAX_MEMBERS),
        );
    }

    if q.strict.unwrap_or(false) && !warnings.is_empty() {
        return Err(AppError::BadRequest(format!(
            "strict mode: {}",
            warnings.summary()
        )));
    }
    let headers = warnings.headers();

    let expected_cells = grid::expected_cells(&axes);
    let confidence = if axes.is_empty() {
        "none"
    } else if axes.iter().all(|a| a.column.is_none()) {
        "heuristic"
    } else if expected_cells == Some(matched) {
        "exact"
    } else {
        "partial"
    };

    let axes_response = if axes.is_empty() {
        None
    } else {
        Some(axes_dto(&axes))
    };

    let members = placed
        .into_iter()
        .map(|(placement, row)| {
            let [x, y, z] = placement.values;
            GridMember {
                image: row.into_list_item(),
                position: GridPosition {
                    x: placement.index[0],
                    y: placement.index[1],
                    z: placement.index[2],
                },
                axis_values: GridAxisValues { x, y, z },
            }
        })
        .collect();

    Ok((
        headers,
        Json(GridMembersResponse {
            grid: row.into_list_item(),
            axes: axes_response,
            members,
            expected_cells,
            matched,
            confidence,
            window_hours,
            warnings: warnings.into_vec(),
        }),
    ))
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
/// `?permanent=true`. A permanent delete also removes the original and
/// thumbnail objects from storage (DB row first — a failed object delete only
/// leaves a harmless orphan).
pub async fn delete_image(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeleteQuery>,
) -> Result<Json<MessageResponse>, AppError> {
    let action = if q.permanent {
        let Some((storage_path, thumbnail_path)) =
            image::delete_permanent(&state.pool, id).await?
        else {
            return Err(AppError::NotFound("Image not found".to_string()));
        };
        storage::delete_image_objects(state.storage.as_ref(), &storage_path, &thumbnail_path)
            .await;
        "permanently deleted"
    } else {
        if !image::soft_delete(&state.pool, id).await? {
            return Err(AppError::NotFound("Image not found".to_string()));
        }
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
