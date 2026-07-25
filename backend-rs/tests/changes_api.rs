//! Integration tests for the change feed (docs/13 A1).
//!
//! This is what finally lets Falcon stay in sync instead of importing each
//! image exactly once and never hearing about it again. The events come from
//! database triggers, so the tests exercise the real write paths (store
//! functions and bulk operations) rather than inserting events by hand.

mod common;

use common::{insert_image, session_cookie, test_router, utc, NewImage};
use promptbox::dto::image::ImageUpdate;
use promptbox::{batch, image};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Fetch the feed and return the parsed body.
async fn changes(pool: PgPool, query: &str) -> Value {
    let (status, body) = common::get_json(
        test_router(pool),
        &format!("/api/changes?{query}"),
        Some(&session_cookie()),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");
    body
}

/// Event kinds in order.
fn kinds(body: &Value) -> Vec<String> {
    body["events"]
        .as_array()
        .expect("events array")
        .iter()
        .map(|e| e["kind"].as_str().unwrap_or_default().to_string())
        .collect()
}

fn changed_fields(event: &Value) -> Vec<String> {
    event["payload"]["changed"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[sqlx::test(migrations = "./migrations")]
async fn inserting_an_image_emits_created(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;

    let body = changes(pool, "since=0").await;

    assert_eq!(kinds(&body), vec!["created"]);
    assert_eq!(body["events"][0]["image_id"], id.to_string());
    assert_eq!(body["events"][0]["seq"], 1);
    assert_eq!(body["next_since"], 1);
    assert_eq!(body["has_more"], false);
}

/// The point of the feed for a PromptBox-is-master setup: a rating change must
/// be visible downstream, and it must say what changed.
#[sqlx::test(migrations = "./migrations")]
async fn rating_change_is_reported_with_the_changed_field(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;

    image::update(
        &pool,
        id,
        &ImageUpdate {
            rating: Some(5),
            ..Default::default()
        },
    )
    .await
    .expect("update");

    let body = changes(pool, "since=1").await;

    assert_eq!(kinds(&body), vec!["updated"]);
    assert_eq!(changed_fields(&body["events"][0]), vec!["rating"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn tag_and_memo_changes_are_reported(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;

    image::update(
        &pool,
        id,
        &ImageUpdate {
            user_tags: Some(vec!["wallpaper".to_string()]),
            user_memo: Some(Some("keep".to_string())),
            ..Default::default()
        },
    )
    .await
    .expect("update");

    let body = changes(pool, "since=1").await;

    let changed = changed_fields(&body["events"][0]);
    assert!(changed.contains(&"user_tags".to_string()));
    assert!(changed.contains(&"user_memo".to_string()));
}

/// An update that changes nothing must not generate an event, otherwise the
/// feed fills with noise every time a client PATCHes the same values back.
#[sqlx::test(migrations = "./migrations")]
async fn a_no_op_update_emits_nothing(pool: PgPool) {
    let id = insert_image(
        &pool,
        NewImage {
            rating: 3,
            ..Default::default()
        },
    )
    .await;

    image::update(
        &pool,
        id,
        &ImageUpdate {
            rating: Some(3), // same value
            ..Default::default()
        },
    )
    .await
    .expect("update");

    let body = changes(pool, "since=1").await;

    assert!(
        body["events"].as_array().expect("array").is_empty(),
        "no change means no event: {body}"
    );
}

/// A re-parse rewrites metadata columns rather than user fields; the feed still
/// has to report it (docs/13 B4 will rely on this).
#[sqlx::test(migrations = "./migrations")]
async fn metadata_only_changes_are_reported_as_metadata(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;

    sqlx::query("UPDATE images SET model_name = $1 WHERE id = $2")
        .bind("RewrittenByReparse")
        .bind(id)
        .execute(&pool)
        .await
        .expect("metadata update");

    let body = changes(pool, "since=1").await;

    assert_eq!(kinds(&body), vec!["updated"]);
    assert_eq!(changed_fields(&body["events"][0]), vec!["metadata"]);
}

/// Soft delete, restore and purge are distinct kinds — a downstream needs to
/// tell "hidden" from "gone".
#[sqlx::test(migrations = "./migrations")]
async fn delete_restore_and_purge_are_distinct(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;

    image::soft_delete(&pool, id).await.expect("soft delete");
    image::restore(&pool, id).await.expect("restore");
    let paths = image::delete_permanent(&pool, id)
        .await
        .expect("purge")
        .expect("row existed");

    let body = changes(pool, "since=1").await;

    assert_eq!(kinds(&body), vec!["deleted", "restored", "purged"]);
    // The tombstone carries what the downstream needs to find its own copy.
    let purged = &body["events"][2];
    assert_eq!(purged["payload"]["storage_path"], paths.0);
    assert!(purged["payload"]["file_hash"].is_string());
}

/// Bulk operations go through different SQL than the single-image path, so the
/// trigger has to cover them too — that is why it lives in the database.
#[sqlx::test(migrations = "./migrations")]
async fn bulk_operations_emit_events_per_image(pool: PgPool) {
    let a = insert_image(&pool, NewImage::default()).await;
    let b = insert_image(&pool, NewImage::default()).await;

    batch::update(&pool, &[a, b], Some(4), None, None, None, None)
        .await
        .expect("bulk update");
    batch::soft_delete(&pool, &[a, b])
        .await
        .expect("bulk delete");

    let body = changes(pool, "since=2").await;

    assert_eq!(
        kinds(&body),
        vec!["updated", "updated", "deleted", "deleted"]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn cursor_paginates_without_gaps_or_repeats(pool: PgPool) {
    for _ in 0..5 {
        insert_image(&pool, NewImage::default()).await;
    }

    let first = changes(pool.clone(), "since=0&limit=2").await;
    assert_eq!(first["events"].as_array().expect("array").len(), 2);
    assert_eq!(first["has_more"], true);
    assert_eq!(first["latest_seq"], 5);

    let next_since = first["next_since"].as_i64().expect("cursor");
    let second = changes(pool.clone(), &format!("since={next_since}&limit=2")).await;
    let second_seqs: Vec<i64> = second["events"]
        .as_array()
        .expect("array")
        .iter()
        .map(|e| e["seq"].as_i64().unwrap_or_default())
        .collect();
    assert_eq!(second_seqs, vec![3, 4], "continues exactly where it stopped");

    let last = changes(pool, "since=5").await;
    assert!(last["events"].as_array().expect("array").is_empty());
    assert_eq!(last["has_more"], false);
    assert_eq!(
        last["next_since"], 5,
        "an empty page must not move the cursor backwards"
    );
}

/// Compact mode answers "what do these images look like now" in one event per
/// image, so a busy editing session does not replay as dozens of updates.
#[sqlx::test(migrations = "./migrations")]
async fn compact_mode_returns_the_newest_event_per_image(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;
    for rating in [1, 2, 3, 4, 5] {
        image::update(
            &pool,
            id,
            &ImageUpdate {
                rating: Some(rating),
                ..Default::default()
            },
        )
        .await
        .expect("update");
    }
    image::soft_delete(&pool, id).await.expect("soft delete");

    let full = changes(pool.clone(), "since=0").await;
    assert_eq!(
        full["events"].as_array().expect("array").len(),
        7,
        "created + 5 updates + delete"
    );

    let compact = changes(pool, "since=0&compact=true").await;
    let events = compact["events"].as_array().expect("array");
    assert_eq!(events.len(), 1, "one event for the one image");
    assert_eq!(
        events[0]["kind"], "deleted",
        "the newest state wins, so a delete is never masked by earlier edits"
    );
    assert_eq!(events[0]["image_id"], id.to_string());
}

/// Images that already existed before this migration must appear in the feed,
/// or a downstream starting from since=0 would only ever see future changes.
#[sqlx::test(migrations = "./migrations")]
async fn backfilled_events_cover_pre_existing_images(pool: PgPool) {
    // Simulate a library that predates the feed: insert with the trigger off.
    sqlx::query("ALTER TABLE images DISABLE TRIGGER trigger_images_change_feed")
        .execute(&pool)
        .await
        .expect("disable trigger");
    let legacy = insert_image(
        &pool,
        NewImage {
            created_at: utc(2025, 1, 1, 0, 0, 0),
            ..Default::default()
        },
    )
    .await;
    sqlx::query("ALTER TABLE images ENABLE TRIGGER trigger_images_change_feed")
        .execute(&pool)
        .await
        .expect("enable trigger");

    // Re-run the backfill statement from the migration.
    sqlx::query(
        "INSERT INTO image_events (image_id, kind, occurred_at, payload) \
         SELECT id, 'created', created_at, jsonb_build_object('backfilled', true) \
         FROM images WHERE deleted_at IS NULL",
    )
    .execute(&pool)
    .await
    .expect("backfill");

    let body = changes(pool, "since=0").await;

    let events = body["events"].as_array().expect("array");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["image_id"], legacy.to_string());
    assert_eq!(events[0]["kind"], "created");
    assert_eq!(events[0]["payload"]["backfilled"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn feed_requires_a_session(pool: PgPool) {
    let (status, _) = common::get_json(test_router(pool), "/api/changes?since=0", None).await;

    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);
}

/// An unknown image id in the feed is impossible, but a purge tombstone must
/// survive the row it describes — the FK is deliberately absent.
#[sqlx::test(migrations = "./migrations")]
async fn tombstone_outlives_the_row(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;
    image::delete_permanent(&pool, id).await.expect("purge");

    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM images WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(remaining, 0);

    let events: Vec<Uuid> =
        sqlx::query_scalar("SELECT image_id FROM image_events WHERE kind = 'purged'")
            .fetch_all(&pool)
            .await
            .expect("events");
    assert_eq!(events, vec![id]);
}
