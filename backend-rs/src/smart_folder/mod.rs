//! Smart folder data access (CRUD on the `smart_folders` table).
//!
//! Mirrors endpoints/smart_folders.py. Like search presets but with an `icon`
//! column and a get-by-id endpoint. NOTE: `smart_folders` has no updated_at
//! trigger (see db/init/03_smart_folders.sql), so updates set `updated_at`
//! explicitly.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::dto::preset::SearchFilters;
use crate::dto::smart_folder::SmartFolderResponse;

#[derive(sqlx::FromRow)]
struct FolderRow {
    id: Uuid,
    name: String,
    icon: Option<String>,
    filters: Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl FolderRow {
    fn into_response(self) -> SmartFolderResponse {
        SmartFolderResponse {
            id: self.id,
            name: self.name,
            icon: self.icon,
            filters: serde_json::from_value(self.filters).unwrap_or_default(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

fn filters_json(f: &SearchFilters) -> Value {
    serde_json::to_value(f).unwrap_or_else(|_| serde_json::json!({}))
}

pub async fn list(pool: &PgPool) -> Result<Vec<SmartFolderResponse>, sqlx::Error> {
    let rows =
        sqlx::query_as::<_, FolderRow>("SELECT * FROM smart_folders ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(FolderRow::into_response).collect())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<SmartFolderResponse>, sqlx::Error> {
    let row = sqlx::query_as::<_, FolderRow>("SELECT * FROM smart_folders WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(FolderRow::into_response))
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    icon: Option<&str>,
    filters: &SearchFilters,
) -> Result<SmartFolderResponse, sqlx::Error> {
    let row = sqlx::query_as::<_, FolderRow>(
        "INSERT INTO smart_folders (id, name, icon, filters) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(name)
    .bind(icon)
    .bind(filters_json(filters))
    .fetch_one(pool)
    .await?;
    Ok(row.into_response())
}

/// Update name, icon, and/or filters. Returns `None` if the folder is missing.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    icon: Option<&str>,
    filters: Option<&SearchFilters>,
) -> Result<Option<SmartFolderResponse>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new("UPDATE smart_folders SET ");
    {
        let mut s = qb.separated(", ");
        if let Some(n) = name {
            s.push("name = ").push_bind_unseparated(n.to_string());
        }
        if let Some(i) = icon {
            s.push("icon = ").push_bind_unseparated(i.to_string());
        }
        if let Some(f) = filters {
            s.push("filters = ").push_bind_unseparated(filters_json(f));
        }
        // No trigger on this table: always advance updated_at.
        s.push("updated_at = ").push_unseparated("NOW()");
    }
    qb.push(" WHERE id = ").push_bind(id).push(" RETURNING *");
    let row = qb.build_query_as::<FolderRow>().fetch_optional(pool).await?;
    Ok(row.map(FolderRow::into_response))
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let affected = sqlx::query("DELETE FROM smart_folders WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}
