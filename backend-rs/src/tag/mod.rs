//! Tag listing (derived from images.user_tags). Mirrors endpoints/tags.py.
//!
//! Tags are not a table: they live in each image's `user_tags` JSONB array.
//! The list endpoint unnests them, groups, optionally filters by substring, and
//! orders by most-recent usage (max updated_at).

use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::util::escape_like;

/// List distinct tags, newest-used first. `q` filters by case-insensitive
/// substring; `limit` caps the result count.
pub async fn list(pool: &PgPool, q: Option<&str>, limit: i64) -> Result<Vec<String>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT tag FROM (\
            SELECT jsonb_array_elements_text(user_tags) AS tag, updated_at \
            FROM images \
            WHERE deleted_at IS NULL AND jsonb_array_length(user_tags) > 0\
         ) sub",
    );
    if let Some(q) = q
        && !q.is_empty()
    {
        qb.push(" WHERE tag ILIKE ")
            .push_bind(format!("%{}%", escape_like(q)))
            .push(" ESCAPE '\\'");
    }
    qb.push(" GROUP BY tag ORDER BY max(updated_at) DESC LIMIT ")
        .push_bind(limit);

    qb.build_query_scalar::<String>().fetch_all(pool).await
}
