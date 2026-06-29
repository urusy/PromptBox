//! Image data access (store): single fetch and dynamic paginated search.

pub mod model;

use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

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

/// Escape LIKE/ILIKE special characters (%, _, \). Mirrors escape_like_pattern
/// in image_service.py.
fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
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
