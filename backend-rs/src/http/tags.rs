//! Tag listing HTTP handler (mirror endpoints/tags.py).

use axum::extract::{Query, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::error::AppError;
use crate::tag;

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
    let tags = tag::list(&state.pool, query.q.as_deref(), limit).await?;
    Ok(Json(tags))
}
