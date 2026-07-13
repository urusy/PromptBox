//! Bulk image operations (mirror services/batch_service.py). All counts are the
//! number of affected rows; tag operations run per-image inside a transaction.

use serde_json::Value;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

/// Bulk update of rating/favorite/needs_improvement and/or add/remove tags.
/// Returns the number of (non-deleted) images affected. Mirrors
/// BatchService.batch_update.
pub async fn update(
    pool: &PgPool,
    ids: &[Uuid],
    rating: Option<i32>,
    is_favorite: Option<bool>,
    needs_improvement: Option<bool>,
    add_tags: Option<&[String]>,
    remove_tags: Option<&[String]>,
) -> Result<u64, sqlx::Error> {
    // Empty tag lists are treated as "no tag op" (Python: `if add_tags or remove_tags`).
    let add = add_tags.filter(|t| !t.is_empty());
    let remove = remove_tags.filter(|t| !t.is_empty());

    if add.is_some() || remove.is_some() {
        return update_with_tags(pool, ids, rating, is_favorite, needs_improvement, add, remove)
            .await;
    }

    // Without tag ops, a single set-based UPDATE suffices.
    if rating.is_none() && is_favorite.is_none() && needs_improvement.is_none() {
        return Ok(0);
    }
    let mut qb = QueryBuilder::<Postgres>::new("UPDATE images SET ");
    {
        let mut s = qb.separated(", ");
        if let Some(v) = rating {
            s.push("rating = ").push_bind_unseparated(v as i16);
        }
        if let Some(v) = is_favorite {
            s.push("is_favorite = ").push_bind_unseparated(v);
        }
        if let Some(v) = needs_improvement {
            s.push("needs_improvement = ").push_bind_unseparated(v);
        }
    }
    qb.push(" WHERE deleted_at IS NULL AND id = ANY(")
        .push_bind(ids.to_vec())
        .push(")");
    Ok(qb.build().execute(pool).await?.rows_affected())
}

/// Per-image tag merge (set union/difference), preserving existing order and
/// appending new tags. Runs in one transaction; returns the count of images
/// processed. Mirrors BatchService._batch_update_tags.
async fn update_with_tags(
    pool: &PgPool,
    ids: &[Uuid],
    rating: Option<i32>,
    is_favorite: Option<bool>,
    needs_improvement: Option<bool>,
    add: Option<&[String]>,
    remove: Option<&[String]>,
) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let rows: Vec<(Uuid, Value)> = sqlx::query_as(
        "SELECT id, user_tags FROM images WHERE deleted_at IS NULL AND id = ANY($1)",
    )
    .bind(ids.to_vec())
    .fetch_all(&mut *tx)
    .await?;

    let mut count: u64 = 0;
    for (id, tags_val) in rows {
        let mut tags: Vec<String> = serde_json::from_value(tags_val).unwrap_or_default();
        if let Some(rm) = remove {
            tags.retain(|t| !rm.contains(t));
        }
        if let Some(ad) = add {
            for t in ad {
                if !tags.contains(t) {
                    tags.push(t.clone());
                }
            }
        }

        let mut qb = QueryBuilder::<Postgres>::new("UPDATE images SET ");
        {
            let mut s = qb.separated(", ");
            s.push("user_tags = ")
                .push_bind_unseparated(serde_json::json!(tags));
            if let Some(v) = rating {
                s.push("rating = ").push_bind_unseparated(v as i16);
            }
            if let Some(v) = is_favorite {
                s.push("is_favorite = ").push_bind_unseparated(v);
            }
            if let Some(v) = needs_improvement {
                s.push("needs_improvement = ").push_bind_unseparated(v);
            }
        }
        qb.push(" WHERE id = ").push_bind(id);
        qb.build().execute(&mut *tx).await?;
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

/// Bulk soft-delete of the still-live images among `ids`. Mirrors
/// BatchService.batch_delete (permanent=false).
pub async fn soft_delete(pool: &PgPool, ids: &[Uuid]) -> Result<u64, sqlx::Error> {
    let res =
        sqlx::query("UPDATE images SET deleted_at = NOW() WHERE deleted_at IS NULL AND id = ANY($1)")
            .bind(ids.to_vec())
            .execute(pool)
            .await?;
    Ok(res.rows_affected())
}

/// Bulk permanent delete regardless of state, returning each removed row's
/// (storage_path, thumbnail_path) so the caller can delete the objects too.
/// Mirrors BatchService.batch_delete (permanent=true).
pub async fn delete_permanent(
    pool: &PgPool,
    ids: &[Uuid],
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as("DELETE FROM images WHERE id = ANY($1) RETURNING storage_path, thumbnail_path")
        .bind(ids.to_vec())
        .fetch_all(pool)
        .await
}

/// Bulk restore of soft-deleted images. Mirrors BatchService.batch_restore.
pub async fn restore(pool: &PgPool, ids: &[Uuid]) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE images SET deleted_at = NULL WHERE deleted_at IS NOT NULL AND id = ANY($1)",
    )
    .bind(ids.to_vec())
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}
