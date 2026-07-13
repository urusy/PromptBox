//! Tag listing HTTP handler (mirror endpoints/tags.py, including its 300s
//! TTLCache).

use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::cache::TtlCache;
use crate::error::AppError;
use crate::tag;

static TAG_CACHE: LazyLock<TtlCache<Vec<String>>> =
    LazyLock::new(|| TtlCache::new(Duration::from_secs(300), 100));

#[derive(Debug, serde::Deserialize)]
pub struct TagsQuery {
    pub q: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    10
}

/// GET /api/tags?q=&limit=10
pub async fn list_tags(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<TagsQuery>,
) -> Result<Json<Vec<String>>, AppError> {
    let limit = query.limit.clamp(1, 100);
    let key = format!("{}:{limit}", query.q.as_deref().unwrap_or("all"));
    if let Some(cached) = TAG_CACHE.get(&key) {
        return Ok(Json(cached));
    }
    let tags = tag::list(&state.pool, query.q.as_deref(), limit).await?;
    TAG_CACHE.insert(key, tags.clone());
    Ok(Json(tags))
}
