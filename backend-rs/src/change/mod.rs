//! Change feed reader (docs/13 A1).
//!
//! Events are produced by database triggers (see
//! migrations/20260725000003_image_events.sql); this module only reads them.
//!
//! Cursor semantics: `seq` is a BIGSERIAL, so it is monotonic per writer.
//! PromptBox has a single writer process, so a client that stores the
//! `next_since` it was given and asks for the next page will not miss events.
//! (With concurrent writers, a sequence can become visible out of order — not
//! a situation this deployment has.)

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Largest page a client may request.
pub const MAX_LIMIT: i64 = 1000;
pub const DEFAULT_LIMIT: i64 = 100;

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct ImageEvent {
    pub seq: i64,
    pub image_id: Uuid,
    pub kind: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: Value,
}

/// Events after `since`, oldest first.
pub async fn since(pool: &PgPool, since: i64, limit: i64) -> Result<Vec<ImageEvent>, sqlx::Error> {
    sqlx::query_as(
        "SELECT seq, image_id, kind, occurred_at, payload FROM image_events \
         WHERE seq > $1 ORDER BY seq ASC LIMIT $2",
    )
    .bind(since)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Compact form: at most one event per image — the newest one in the range.
///
/// A client that only wants "what does this image look like now" does not care
/// that it was rated three times; sending one event per image keeps a busy
/// afternoon of edits from turning into a long replay. The chosen event is the
/// latest, so a delete or purge always wins over the updates before it.
pub async fn since_compact(
    pool: &PgPool,
    since: i64,
    limit: i64,
) -> Result<Vec<ImageEvent>, sqlx::Error> {
    // DISTINCT ON must order by image_id first, so the compaction happens in a
    // subquery and the page is cut in `seq` order — otherwise `limit` would
    // slice the feed alphabetically by id and the cursor would be meaningless.
    sqlx::query_as(
        "SELECT seq, image_id, kind, occurred_at, payload FROM ( \
             SELECT DISTINCT ON (image_id) seq, image_id, kind, occurred_at, payload \
             FROM image_events WHERE seq > $1 \
             ORDER BY image_id, seq DESC \
         ) latest \
         ORDER BY seq ASC LIMIT $2",
    )
    .bind(since)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Highest sequence currently in the feed (0 when empty). This is the cursor a
/// client should keep when it is up to date.
pub async fn latest_seq(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let seq: Option<i64> = sqlx::query_scalar("SELECT max(seq) FROM image_events")
        .fetch_one(pool)
        .await?;
    Ok(seq.unwrap_or(0))
}
