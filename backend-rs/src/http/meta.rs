//! Service identity endpoints: what this backend is and how it is configured.
//!
//! Falcon mirrors this backend's routes by hand and had no way to tell which
//! build it was talking to (docs/13 B14) — a version mismatch could only be
//! discovered as a 404 or a silently dropped parameter.

use axum::extract::State;
use axum::Json;
use chrono::DateTime;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::meta::{ConfigResponse, Limits, VersionResponse};
use crate::error::AppError;
use crate::parser;

/// Highest applied migration version, i.e. the schema generation this database
/// is at. Returns `None` when the database is unreachable so `/api/version`
/// still answers during an outage.
async fn schema_version(state: &AppState) -> Option<i64> {
    sqlx::query_scalar::<_, Option<i64>>("SELECT max(version) FROM _sqlx_migrations")
        .fetch_one(&state.pool)
        .await
        .ok()
        .flatten()
}

/// Build timestamp as RFC3339 UTC (the build script stores epoch seconds).
fn built_at() -> String {
    env!("PROMPTBOX_BUILT_AT_EPOCH")
        .parse::<i64>()
        .ok()
        .and_then(|secs| DateTime::from_timestamp(secs, 0))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string())
}

/// GET /api/version — unauthenticated on purpose: a client has to be able to
/// check compatibility before logging in.
pub async fn version(State(state): State<AppState>) -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: env!("PROMPTBOX_GIT_SHA").to_string(),
        built_at: built_at(),
        schema_version: schema_version(&state).await,
        parser_version: parser::VERSION,
    })
}

/// GET /api/config — operational settings for clients that would otherwise
/// hard-code them. Authenticated, and deliberately free of credentials: no
/// database URL, no S3 keys, no password hash.
pub async fn config(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<ConfigResponse>, AppError> {
    let cfg = &state.config;

    let mut features = vec!["images", "search", "showcases", "export", "duplicates"];
    if cfg.watcher_enabled {
        features.push("import_worker");
    }
    if !cfg.gelbooru_api_key.is_empty() && !cfg.gelbooru_user_id.is_empty() {
        features.push("gelbooru");
    }
    // CivitAI needs no credentials, so it is always available.
    features.push("civitai");

    Ok(Json(ConfigResponse {
        features,
        limits: Limits {
            // Mirrors the clamp in http::images::list_images and the guard in
            // http::bulk::validate_ids — keep these in sync.
            max_per_page: 120,
            default_per_page: 24,
            bulk_max_ids: 500,
        },
        storage_backend: cfg.storage_backend.clone(),
        thumbnail_sizes: vec![cfg.thumbnail_size],
    }))
}
