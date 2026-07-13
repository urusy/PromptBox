//! Image data access (store): single fetch and dynamic paginated search.

pub mod model;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::dto::image::ImageUpdate;
use crate::util::escape_like;
use model::ImageRow;

/// Search/filter parameters at the store level. Mirrors image_service.py
/// ImageSearchParams. Filters are ported incrementally.
#[derive(Debug)]
pub struct SearchParams {
    pub source_tool: Option<String>,
    pub model_type: Option<String>,
    pub min_rating: Option<i16>,
    pub exact_rating: Option<i16>,
    pub max_rating: Option<i16>,
    pub is_favorite: Option<bool>,
    pub needs_improvement: Option<bool>,
    pub model_name: Option<String>,
    pub sampler_name: Option<String>,
    pub file_type: Option<String>,
    pub tags: Vec<String>,
    pub lora_name: Option<String>,
    /// Full-text search over positive_prompt (Postgres tsvector).
    pub q: Option<String>,
    /// XYZ-grid images: model_params->>'is_xyz_grid'.
    pub is_xyz_grid: Option<bool>,
    /// Upscaled images: presence of model_params->>'hires_upscaler'.
    pub is_upscaled: Option<bool>,
    /// "portrait" | "landscape" | "square" (others ignored).
    pub orientation: Option<String>,
    pub min_width: Option<i32>,
    pub min_height: Option<i32>,
    /// ISO date/datetime lower bound on created_at (invalid values ignored).
    pub date_from: Option<String>,
    pub seed: Option<i64>,
    /// When > 0, search seed within +/- this tolerance instead of exact match.
    pub seed_tolerance: Option<i64>,
    /// Restrict to images belonging to this showcase (membership subquery).
    pub showcase_id: Option<Uuid>,
    pub include_deleted: bool,
    pub page: i64,
    pub per_page: i64,
    pub sort_by: String,
    pub sort_order: String,
}

/// Result of a paginated list query.
pub struct ListResult {
    pub items: Vec<ImageRow>,
    pub total: i64,
}

/// Fetch a single image by id (including soft-deleted ones).
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ImageRow>, sqlx::Error> {
    sqlx::query_as::<_, ImageRow>("SELECT * FROM images WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Apply a partial update (only the fields present in the PATCH body) and return
/// the updated row, or `None` if the image does not exist. Mirrors
/// image_service.update_image: absent fields are left untouched and `updated_at`
/// is bumped by the `trigger_images_updated_at` DB trigger. With no fields
/// supplied, the current row is returned unchanged (empty `SET` is not emitted).
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    u: &ImageUpdate,
) -> Result<Option<ImageRow>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new("UPDATE images SET ");
    let mut any = false;
    {
        let mut set = qb.separated(", ");
        if let Some(v) = u.rating {
            set.push("rating = ").push_bind_unseparated(v as i16);
            any = true;
        }
        if let Some(v) = u.is_favorite {
            set.push("is_favorite = ").push_bind_unseparated(v);
            any = true;
        }
        if let Some(v) = u.needs_improvement {
            set.push("needs_improvement = ").push_bind_unseparated(v);
            any = true;
        }
        if let Some(v) = &u.user_tags {
            set.push("user_tags = ")
                .push_bind_unseparated(serde_json::json!(v));
            any = true;
        }
        // user_memo is nullable: Some(None) clears it, Some(Some(s)) sets it.
        if let Some(memo) = &u.user_memo {
            set.push("user_memo = ").push_bind_unseparated(memo.clone());
            any = true;
        }
    }
    if !any {
        return get_by_id(pool, id).await;
    }
    qb.push(" WHERE id = ").push_bind(id).push(" RETURNING *");
    qb.build_query_as::<ImageRow>().fetch_optional(pool).await
}

/// Soft-delete an image by setting `deleted_at = NOW()`. Returns whether a row
/// was affected. Mirrors image_service.delete_image.
pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let affected = sqlx::query("UPDATE images SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}

/// Permanently delete an image row, returning its (storage_path,
/// thumbnail_path) so the caller can delete the objects too, or `None` if the
/// image does not exist.
pub async fn delete_permanent(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "DELETE FROM images WHERE id = $1 RETURNING storage_path, thumbnail_path",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Restore a soft-deleted image. Returns `false` if the image does not exist or
/// is not currently deleted. Mirrors image_service.restore_image.
pub async fn restore(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let affected =
        sqlx::query("UPDATE images SET deleted_at = NULL WHERE id = $1 AND deleted_at IS NOT NULL")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();
    Ok(affected > 0)
}

/// Push the WHERE conditions shared by the count and page queries (everything
/// after a leading "WHERE 1=1"). Ports image_service.py:_build_search_query.
fn push_filters(qb: &mut QueryBuilder<Postgres>, p: &SearchParams) {
    if p.include_deleted {
        qb.push(" AND deleted_at IS NOT NULL");
    } else {
        qb.push(" AND deleted_at IS NULL");
    }
    if let Some(v) = &p.source_tool {
        qb.push(" AND source_tool = ").push_bind(v.clone());
    }
    if let Some(v) = &p.model_type {
        qb.push(" AND model_type = ").push_bind(v.clone());
    }
    // rating: exact takes precedence over min (mirrors image_service.py).
    if let Some(v) = p.exact_rating {
        qb.push(" AND rating = ").push_bind(v);
    } else if let Some(v) = p.min_rating {
        qb.push(" AND rating >= ").push_bind(v);
    }
    // max_rating is a new Falcon filter (rating <= n), independent of the above.
    if let Some(v) = p.max_rating {
        qb.push(" AND rating <= ").push_bind(v);
    }
    if let Some(v) = p.is_favorite {
        qb.push(" AND is_favorite = ").push_bind(v);
    }
    if let Some(v) = p.needs_improvement {
        qb.push(" AND needs_improvement = ").push_bind(v);
    }
    if let Some(v) = &p.model_name {
        qb.push(" AND model_name ILIKE ")
            .push_bind(format!("%{}%", escape_like(v)))
            .push(" ESCAPE '\\'");
    }
    if let Some(v) = &p.sampler_name {
        qb.push(" AND sampler_name = ").push_bind(v.clone());
    }
    if let Some(v) = &p.file_type {
        let ext = v.trim_start_matches('.').to_lowercase();
        if ext == "jpg" {
            // jpg matches both .jpg and .jpeg (mirrors image_service.py).
            qb.push(" AND (original_filename ILIKE ")
                .push_bind("%.jpg".to_string())
                .push(" ESCAPE '\\' OR original_filename ILIKE ")
                .push_bind("%.jpeg".to_string())
                .push(" ESCAPE '\\')");
        } else {
            qb.push(" AND original_filename ILIKE ")
                .push_bind(format!("%.{}", escape_like(&ext)))
                .push(" ESCAPE '\\'");
        }
    }
    // user_tags: every tag must be present (AND), via JSONB containment.
    for tag in &p.tags {
        qb.push(" AND user_tags @> ")
            .push_bind(serde_json::json!([tag]));
    }
    // lora_name: JSONB containment on the loras array ([{"name": ...}]).
    if let Some(v) = &p.lora_name {
        qb.push(" AND loras @> ")
            .push_bind(serde_json::json!([{ "name": v }]));
    }
    // Full-text search over positive_prompt. Mirrors image_service.py: spaces
    // become AND operators and the result is fed to to_tsquery. The query text
    // is a bound parameter (injection-safe); malformed tsquery syntax surfaces
    // the same way it does in Python.
    if let Some(q) = &p.q
        && !q.is_empty()
    {
        let terms = q.replace(' ', " & ");
        qb.push(
            " AND to_tsvector('english', coalesce(positive_prompt, '')) @@ to_tsquery('english', ",
        )
        .push_bind(terms)
        .push(")");
    }
    // XYZ grid flag stored as text in model_params.
    if let Some(b) = p.is_xyz_grid {
        if b {
            qb.push(" AND model_params->>'is_xyz_grid' = 'true'");
        } else {
            qb.push(
                " AND (model_params->>'is_xyz_grid' IS NULL OR model_params->>'is_xyz_grid' <> 'true')",
            );
        }
    }
    // Upscaled = a hires_upscaler key is present in model_params.
    if let Some(b) = p.is_upscaled {
        if b {
            qb.push(" AND model_params->>'hires_upscaler' IS NOT NULL");
        } else {
            qb.push(" AND model_params->>'hires_upscaler' IS NULL");
        }
    }
    // Orientation: only the three known values add a condition.
    if let Some(o) = &p.orientation {
        match o.as_str() {
            "portrait" => {
                qb.push(" AND height > width");
            }
            "landscape" => {
                qb.push(" AND width > height");
            }
            "square" => {
                qb.push(" AND width = height");
            }
            _ => {}
        }
    }
    if let Some(w) = p.min_width {
        qb.push(" AND width >= ").push_bind(w);
    }
    if let Some(h) = p.min_height {
        qb.push(" AND height >= ").push_bind(h);
    }
    // created_at lower bound; unparseable date strings are ignored (as in Python).
    if let Some(df) = p.date_from.as_deref().and_then(parse_date_from) {
        qb.push(" AND created_at >= ").push_bind(df);
    }
    // Seed: exact match, or a +/- tolerance window when tolerance > 0.
    if let Some(seed) = p.seed {
        match p.seed_tolerance {
            Some(tol) if tol > 0 => {
                qb.push(" AND seed IS NOT NULL AND seed >= ")
                    .push_bind(seed.saturating_sub(tol))
                    .push(" AND seed <= ")
                    .push_bind(seed.saturating_add(tol));
            }
            _ => {
                qb.push(" AND seed = ").push_bind(seed);
            }
        }
    }
    // Showcase membership. A subquery (rather than a JOIN) keeps the
    // "FROM images WHERE 1=1" shape that the count/page/neighbor builders share;
    // the (showcase_id, image_id) primary key guarantees at most one match.
    if let Some(sid) = p.showcase_id {
        qb.push(" AND id IN (SELECT image_id FROM showcase_images WHERE showcase_id = ")
            .push_bind(sid)
            .push(")");
    }
}

/// Parse an ISO date or datetime into a UTC instant for the `date_from` filter.
/// Accepts RFC3339 (`2024-01-02T03:04:05Z`), naive datetime
/// (`2024-01-02T03:04:05`), or date-only (`2024-01-02`, treated as midnight UTC).
/// Returns `None` for anything else, so invalid input is silently ignored.
fn parse_date_from(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let ndt = d.and_hms_opt(0, 0, 0)?;
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
    }
    None
}

/// Resolve the sort column (whitelisted) and direction. Mirrors
/// _ALLOWED_SORT_COLUMNS in image_service.py. The column is a fixed literal
/// (never user input), so pushing it as raw SQL is injection-safe.
fn sort_clause(p: &SearchParams) -> (&'static str, &'static str) {
    let col = match p.sort_by.as_str() {
        "updated_at" => "updated_at",
        "rating" => "rating",
        "model_name" => "model_name",
        "file_size_bytes" => "file_size_bytes",
        "width" => "width",
        "height" => "height",
        _ => "created_at",
    };
    let dir = if p.sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };
    (col, dir)
}

/// Search images with filters, sorting and pagination.
pub async fn list(pool: &PgPool, p: &SearchParams) -> Result<ListResult, sqlx::Error> {
    let mut cb = QueryBuilder::<Postgres>::new("SELECT count(*) FROM images WHERE 1=1");
    push_filters(&mut cb, p);
    let total: i64 = cb.build_query_scalar::<i64>().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM images WHERE 1=1");
    push_filters(&mut qb, p);
    let (col, dir) = sort_clause(p);
    qb.push(" ORDER BY ").push(col).push(" ").push(dir);
    let offset = (p.page - 1) * p.per_page;
    qb.push(" OFFSET ")
        .push_bind(offset)
        .push(" LIMIT ")
        .push_bind(p.per_page);

    let items = qb.build_query_as::<ImageRow>().fetch_all(pool).await?;
    Ok(ListResult { items, total })
}

/// Fetch non-deleted images for export, newest first. With `ids`, restrict to
/// those ids; otherwise return all. Mirrors export_service.py.
pub async fn list_for_export(
    pool: &PgPool,
    ids: Option<&[Uuid]>,
) -> Result<Vec<ImageRow>, sqlx::Error> {
    match ids {
        Some(ids) => {
            sqlx::query_as::<_, ImageRow>(
                "SELECT * FROM images WHERE deleted_at IS NULL AND id = ANY($1) \
                 ORDER BY created_at DESC",
            )
            .bind(ids.to_vec())
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, ImageRow>(
                "SELECT * FROM images WHERE deleted_at IS NULL ORDER BY created_at DESC",
            )
            .fetch_all(pool)
            .await
        }
    }
}

/// Compute (prev, next) image ids within the search context, mirroring
/// image_service.py:get_image_with_neighbors. Row-value comparison
/// `(sort_col, id) <cmp> (subquery)` avoids handling the sort column's type.
///
/// Direction (matches image_service.py):
///   asc  listing: prev `<` ORDER BY DESC, next `>` ORDER BY ASC
///   desc listing: prev `>` ORDER BY ASC,  next `<` ORDER BY DESC
pub async fn neighbors(
    pool: &PgPool,
    p: &SearchParams,
    current_id: Uuid,
) -> Result<(Option<Uuid>, Option<Uuid>), sqlx::Error> {
    // Within a showcase, navigation follows the curated sort_order rather than
    // the listing sort (mirrors image_service.py:_get_showcase_neighbors).
    if let Some(sid) = p.showcase_id {
        return showcase_neighbors(pool, sid, current_id).await;
    }

    let (col, _) = sort_clause(p);
    let asc = p.sort_order.eq_ignore_ascii_case("asc");
    let prev = neighbor_one(
        pool,
        p,
        current_id,
        col,
        if asc { "<" } else { ">" },
        if asc { "DESC" } else { "ASC" },
    )
    .await?;
    let next = neighbor_one(
        pool,
        p,
        current_id,
        col,
        if asc { ">" } else { "<" },
        if asc { "ASC" } else { "DESC" },
    )
    .await?;
    Ok((prev, next))
}

async fn neighbor_one(
    pool: &PgPool,
    p: &SearchParams,
    current_id: Uuid,
    col: &str,
    cmp: &str,
    order_dir: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new("SELECT id FROM images WHERE 1=1");
    push_filters(&mut qb, p);
    qb.push(" AND (")
        .push(col)
        .push(", id) ")
        .push(cmp)
        .push(" (SELECT s.")
        .push(col)
        .push(", s.id FROM images s WHERE s.id = ")
        .push_bind(current_id)
        .push(") ORDER BY ")
        .push(col)
        .push(" ")
        .push(order_dir)
        .push(", id ")
        .push(order_dir)
        .push(" LIMIT 1");
    qb.build_query_scalar::<Uuid>().fetch_optional(pool).await
}

/// Compute (prev, next) within a showcase, ordered by showcase_images.sort_order
/// (the curated order), independent of the listing sort and other filters.
/// Returns (None, None) if the image is not a member. Mirrors
/// image_service.py:_get_showcase_neighbors.
async fn showcase_neighbors(
    pool: &PgPool,
    showcase_id: Uuid,
    current_id: Uuid,
) -> Result<(Option<Uuid>, Option<Uuid>), sqlx::Error> {
    let current_order: Option<i32> = sqlx::query_scalar(
        "SELECT sort_order FROM showcase_images WHERE showcase_id = $1 AND image_id = $2",
    )
    .bind(showcase_id)
    .bind(current_id)
    .fetch_optional(pool)
    .await?;

    let Some(order) = current_order else {
        return Ok((None, None));
    };

    let prev: Option<Uuid> = sqlx::query_scalar(
        "SELECT image_id FROM showcase_images \
         WHERE showcase_id = $1 AND sort_order < $2 \
         ORDER BY sort_order DESC LIMIT 1",
    )
    .bind(showcase_id)
    .bind(order)
    .fetch_optional(pool)
    .await?;

    let next: Option<Uuid> = sqlx::query_scalar(
        "SELECT image_id FROM showcase_images \
         WHERE showcase_id = $1 AND sort_order > $2 \
         ORDER BY sort_order ASC LIMIT 1",
    )
    .bind(showcase_id)
    .bind(order)
    .fetch_optional(pool)
    .await?;

    Ok((prev, next))
}
