//! Showcase data access (mirror endpoints/showcases.py).
//!
//! Showcases are curated image collections; membership lives in
//! `showcase_images` with a per-showcase `sort_order`. The `showcases` table has
//! an updated_at trigger, but mutations to `showcase_images` do not fire it, so
//! membership operations bump `showcases.updated_at` explicitly (matching Python).

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::showcase::{
    ShowcaseDetailResponse, ShowcaseImageCheckResult, ShowcaseImageInfo, ShowcaseResponse,
};

#[derive(sqlx::FromRow)]
struct ShowcaseBaseRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    cover_image_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ShowcaseBaseRow {
    fn into_response(self, cover_thumbnail_path: Option<String>, image_count: i64) -> ShowcaseResponse {
        ShowcaseResponse {
            id: self.id,
            name: self.name,
            description: self.description,
            icon: self.icon,
            cover_image_id: self.cover_image_id,
            cover_thumbnail_path,
            image_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ShowcaseListRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    cover_image_id: Option<Uuid>,
    cover_thumbnail_path: Option<String>,
    image_count: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

const BASE_COLS: &str =
    "id, name, description, icon, cover_image_id, created_at, updated_at";

/// List showcases with association counts and cover thumbnails, newest first.
/// `image_count` counts all associations (including soft-deleted images), as in
/// the Python listing.
pub async fn list(pool: &PgPool) -> Result<Vec<ShowcaseResponse>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ShowcaseListRow>(
        "SELECT s.id, s.name, s.description, s.icon, s.cover_image_id, \
                img.thumbnail_path AS cover_thumbnail_path, \
                COALESCE(cnt.image_count, 0) AS image_count, \
                s.created_at, s.updated_at \
         FROM showcases s \
         LEFT JOIN (SELECT showcase_id, count(image_id) AS image_count \
                    FROM showcase_images GROUP BY showcase_id) cnt \
                ON s.id = cnt.showcase_id \
         LEFT JOIN images img ON s.cover_image_id = img.id \
         ORDER BY s.created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ShowcaseResponse {
            id: r.id,
            name: r.name,
            description: r.description,
            icon: r.icon,
            cover_image_id: r.cover_image_id,
            cover_thumbnail_path: r.cover_thumbnail_path,
            image_count: r.image_count,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// For the given image ids, return how many are already in each showcase.
pub async fn check_images(
    pool: &PgPool,
    image_ids: &[Uuid],
) -> Result<Vec<ShowcaseImageCheckResult>, sqlx::Error> {
    let rows: Vec<(Uuid, i64)> = sqlx::query_as(
        "SELECT showcase_id, count(image_id) AS existing_count \
         FROM showcase_images WHERE image_id = ANY($1) GROUP BY showcase_id",
    )
    .bind(image_ids.to_vec())
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(showcase_id, existing_count)| ShowcaseImageCheckResult {
            showcase_id,
            existing_count,
        })
        .collect())
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    icon: Option<&str>,
) -> Result<ShowcaseResponse, sqlx::Error> {
    let row = sqlx::query_as::<_, ShowcaseBaseRow>(
        "INSERT INTO showcases (id, name, description, icon) VALUES ($1, $2, $3, $4) \
         RETURNING id, name, description, icon, cover_image_id, created_at, updated_at",
    )
    .bind(Uuid::now_v7())
    .bind(name)
    .bind(description)
    .bind(icon)
    .fetch_one(pool)
    .await?;
    Ok(row.into_response(None, 0))
}

/// Fetch a showcase with its (non-deleted) images, ordered by sort_order.
pub async fn get_detail(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ShowcaseDetailResponse>, sqlx::Error> {
    let base = sqlx::query_as::<_, ShowcaseBaseRow>(&format!(
        "SELECT {BASE_COLS} FROM showcases WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some(base) = base else {
        return Ok(None);
    };

    let cover_thumbnail_path = cover_thumbnail(pool, base.cover_image_id).await?;

    let rows: Vec<(Uuid, String, String, i32, DateTime<Utc>)> = sqlx::query_as(
        "SELECT i.id, i.storage_path, i.thumbnail_path, si.sort_order, si.added_at \
         FROM images i JOIN showcase_images si ON i.id = si.image_id \
         WHERE si.showcase_id = $1 AND i.deleted_at IS NULL \
         ORDER BY si.sort_order",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let images: Vec<ShowcaseImageInfo> = rows
        .into_iter()
        .map(
            |(id, storage_path, thumbnail_path, sort_order, added_at)| ShowcaseImageInfo {
                id,
                storage_path,
                thumbnail_path,
                sort_order,
                added_at,
            },
        )
        .collect();
    let image_count = images.len() as i64;

    Ok(Some(ShowcaseDetailResponse {
        showcase: base.into_response(cover_thumbnail_path, image_count),
        images,
    }))
}

/// Update mutable fields. `description`/`icon`: `Some("")` clears, `Some(v)`
/// sets, `None` leaves unchanged. `cover_image_id`: `Some` sets, `None` leaves.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    icon: Option<&str>,
    cover_image_id: Option<Uuid>,
) -> Result<Option<ShowcaseResponse>, sqlx::Error> {
    use sqlx::{Postgres, QueryBuilder};

    let mut qb = QueryBuilder::<Postgres>::new("UPDATE showcases SET ");
    {
        let mut s = qb.separated(", ");
        if let Some(n) = name {
            s.push("name = ").push_bind_unseparated(n.to_string());
        }
        if let Some(d) = description {
            let v = (!d.is_empty()).then(|| d.to_string());
            s.push("description = ").push_bind_unseparated(v);
        }
        if let Some(i) = icon {
            let v = (!i.is_empty()).then(|| i.to_string());
            s.push("icon = ").push_bind_unseparated(v);
        }
        if let Some(cid) = cover_image_id {
            s.push("cover_image_id = ").push_bind_unseparated(cid);
        }
        s.push("updated_at = ").push_unseparated("NOW()");
    }
    qb.push(" WHERE id = ")
        .push_bind(id)
        .push(" RETURNING ")
        .push(BASE_COLS);

    let base = qb
        .build_query_as::<ShowcaseBaseRow>()
        .fetch_optional(pool)
        .await?;
    let Some(base) = base else {
        return Ok(None);
    };

    let image_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM showcase_images WHERE showcase_id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
    let cover_thumbnail_path = cover_thumbnail(pool, base.cover_image_id).await?;

    Ok(Some(base.into_response(cover_thumbnail_path, image_count)))
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let affected = sqlx::query("DELETE FROM showcases WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}

/// Add images (skipping ones already present, deduping the request), appending
/// after the current max sort_order. Sets the cover to the first id when unset.
/// Returns the number added, or `None` if the showcase is missing.
pub async fn add_images(
    pool: &PgPool,
    id: Uuid,
    image_ids: &[Uuid],
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let cover: Option<Option<Uuid>> =
        sqlx::query_scalar("SELECT cover_image_id FROM showcases WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(current_cover) = cover else {
        return Ok(None);
    };

    let mut max_order: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), 0) FROM showcase_images WHERE showcase_id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    let existing: Vec<Uuid> =
        sqlx::query_scalar("SELECT image_id FROM showcase_images WHERE showcase_id = $1")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;
    let mut seen: HashSet<Uuid> = existing.into_iter().collect();

    let mut added: i64 = 0;
    for &img in image_ids {
        // insert() is true when newly added — covers both already-present and
        // duplicates within the request.
        if seen.insert(img) {
            max_order += 1;
            sqlx::query(
                "INSERT INTO showcase_images (showcase_id, image_id, sort_order) \
                 VALUES ($1, $2, $3)",
            )
            .bind(id)
            .bind(img)
            .bind(max_order)
            .execute(&mut *tx)
            .await?;
            added += 1;
        }
    }

    if current_cover.is_none() && !image_ids.is_empty() {
        sqlx::query("UPDATE showcases SET cover_image_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(image_ids[0])
            .bind(id)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query("UPDATE showcases SET updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(Some(added))
}

/// Remove images from a showcase; clears the cover if it was removed.
/// Returns the number removed, or `None` if the showcase is missing.
pub async fn remove_images(
    pool: &PgPool,
    id: Uuid,
    image_ids: &[Uuid],
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let cover: Option<Option<Uuid>> =
        sqlx::query_scalar("SELECT cover_image_id FROM showcases WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(current_cover) = cover else {
        return Ok(None);
    };

    let removed = sqlx::query(
        "DELETE FROM showcase_images WHERE showcase_id = $1 AND image_id = ANY($2)",
    )
    .bind(id)
    .bind(image_ids.to_vec())
    .execute(&mut *tx)
    .await?
    .rows_affected();

    let clear_cover = current_cover.is_some_and(|c| image_ids.contains(&c));
    if clear_cover {
        sqlx::query("UPDATE showcases SET cover_image_id = NULL, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query("UPDATE showcases SET updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(Some(removed as i64))
}

/// Set sort_order to the position of each id in `image_ids`.
/// Returns `None` if the showcase is missing.
pub async fn reorder_images(
    pool: &PgPool,
    id: Uuid,
    image_ids: &[Uuid],
) -> Result<Option<()>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM showcases WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Ok(None);
    }

    for (i, &img) in image_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE showcase_images SET sort_order = $1 WHERE showcase_id = $2 AND image_id = $3",
        )
        .bind(i as i32)
        .bind(id)
        .bind(img)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("UPDATE showcases SET updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(Some(()))
}

/// Look up a cover image's thumbnail path (None when there is no cover).
async fn cover_thumbnail(
    pool: &PgPool,
    cover_image_id: Option<Uuid>,
) -> Result<Option<String>, sqlx::Error> {
    let Some(cid) = cover_image_id else {
        return Ok(None);
    };
    sqlx::query_scalar("SELECT thumbnail_path FROM images WHERE id = $1")
        .bind(cid)
        .fetch_optional(pool)
        .await
}
