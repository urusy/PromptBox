//! Change feed endpoint (docs/13 A1).

use axum::extract::{Query, State};
use axum::Json;

use super::auth::CurrentUser;
use super::AppState;
use crate::change::{self, ImageEvent, DEFAULT_LIMIT, MAX_LIMIT};
use crate::error::AppError;

#[derive(Debug, serde::Deserialize)]
pub struct ChangesQuery {
    /// Cursor: return events with `seq` strictly greater than this. Start at 0.
    #[serde(default)]
    pub since: i64,
    pub limit: Option<i64>,
    /// Collapse to the newest event per image (see `change::since_compact`).
    #[serde(default)]
    pub compact: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct ChangesResponse {
    pub events: Vec<ImageEvent>,
    /// Cursor to pass as `since` next time.
    pub next_since: i64,
    /// Whether the feed has more beyond `next_since`.
    pub has_more: bool,
    /// Newest sequence in the feed, so a client can see how far behind it is.
    pub latest_seq: i64,
}

/// GET /api/changes
///
/// The feed a downstream (Falcon) polls to stay in sync: creations, edits,
/// soft deletes, restores and purge tombstones, in order.
pub async fn list_changes(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ChangesQuery>,
) -> Result<Json<ChangesResponse>, AppError> {
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let since = q.since.max(0);

    let events = if q.compact {
        change::since_compact(&state.pool, since, limit).await?
    } else {
        change::since(&state.pool, since, limit).await?
    };

    let latest_seq = change::latest_seq(&state.pool).await?;
    // With no events the cursor must not move backwards, so it stays put.
    let next_since = events.last().map(|e| e.seq).unwrap_or(since);

    Ok(Json(ChangesResponse {
        events,
        next_since,
        has_more: next_since < latest_seq,
        latest_seq,
    }))
}
