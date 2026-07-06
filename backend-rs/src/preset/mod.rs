//! Search preset data access (CRUD on the `search_presets` table).
//!
//! Mirrors endpoints/search_presets.py. `filters` is stored as JSONB; the
//! `trigger_search_presets_updated_at` DB trigger maintains `updated_at`.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::dto::preset::{SearchFilters, SearchPresetResponse};

#[derive(sqlx::FromRow)]
struct PresetRow {
    id: Uuid,
    name: String,
    filters: Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PresetRow {
    fn into_response(self) -> SearchPresetResponse {
        SearchPresetResponse {
            id: self.id,
            name: self.name,
            // Stored JSONB always originates from a SearchFilters; default on the
            // (unreachable) decode error rather than failing the request.
            filters: serde_json::from_value(self.filters).unwrap_or_default(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

fn filters_json(f: &SearchFilters) -> Value {
    serde_json::to_value(f).unwrap_or_else(|_| serde_json::json!({}))
}

pub async fn list(pool: &PgPool) -> Result<Vec<SearchPresetResponse>, sqlx::Error> {
    let rows =
        sqlx::query_as::<_, PresetRow>("SELECT * FROM search_presets ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(PresetRow::into_response).collect())
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    filters: &SearchFilters,
) -> Result<SearchPresetResponse, sqlx::Error> {
    let row = sqlx::query_as::<_, PresetRow>(
        "INSERT INTO search_presets (id, name, filters) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(name)
    .bind(filters_json(filters))
    .fetch_one(pool)
    .await?;
    Ok(row.into_response())
}

/// Update name and/or filters. Returns `None` if the preset does not exist.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    filters: Option<&SearchFilters>,
) -> Result<Option<SearchPresetResponse>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new("UPDATE search_presets SET ");
    {
        let mut s = qb.separated(", ");
        let mut any = false;
        if let Some(n) = name {
            s.push("name = ").push_bind_unseparated(n.to_string());
            any = true;
        }
        if let Some(f) = filters {
            s.push("filters = ").push_bind_unseparated(filters_json(f));
            any = true;
        }
        // Empty body still touches the row so updated_at advances (matches Python).
        if !any {
            s.push("updated_at = ").push_unseparated("NOW()");
        }
    }
    qb.push(" WHERE id = ").push_bind(id).push(" RETURNING *");
    let row = qb.build_query_as::<PresetRow>().fetch_optional(pool).await?;
    Ok(row.map(PresetRow::into_response))
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let affected = sqlx::query("DELETE FROM search_presets WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}
