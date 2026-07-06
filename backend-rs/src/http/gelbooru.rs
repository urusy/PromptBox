//! Gelbooru tag-search proxy handler (mirror endpoints/gelbooru.py).

use axum::extract::{Query, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::dto::gelbooru::GelbooruTagSearchResponse;
use crate::error::AppError;
use crate::gelbooru::{self, GelbooruError};

#[derive(Debug, serde::Deserialize)]
pub struct TagQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    30
}

/// GET /api/gelbooru/tags?q=&limit=30
pub async fn search_tags(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<TagQuery>,
) -> Result<Json<GelbooruTagSearchResponse>, AppError> {
    if query.q.chars().count() < 2 {
        return Err(AppError::BadRequest(
            "q must be at least 2 characters".to_string(),
        ));
    }
    let limit = query.limit.clamp(1, 100);
    let tags = gelbooru::search_tags(
        &state.config.gelbooru_api_key,
        &state.config.gelbooru_user_id,
        &query.q,
        limit,
    )
    .await
    .map_err(|e| match e {
        GelbooruError::RateLimit => AppError::TooManyRequests(
            "Gelbooru API rate limit exceeded. Please try again later.".to_string(),
        ),
        GelbooruError::Unavailable => {
            AppError::ServiceUnavailable("Gelbooru API is temporarily unavailable.".to_string())
        }
        GelbooruError::Upstream => {
            AppError::BadGateway("Gelbooru API returned an unexpected response.".to_string())
        }
    })?;

    Ok(Json(GelbooruTagSearchResponse {
        tags,
        query: query.q,
    }))
}
