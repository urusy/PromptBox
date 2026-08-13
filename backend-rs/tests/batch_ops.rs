//! Integration tests for the bulk operations (`batch::*`).
//!
//! These run multi-row SQL and a per-image tag merge inside a transaction; a
//! regression here silently corrupts many images at once, so the semantics
//! (which rows are affected, what happens to existing tags) are pinned.

mod common;

use common::{insert_image, is_deleted, tags_of, utc, NewImage};
use promptbox::batch;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn update_sets_scalar_fields_and_skips_deleted(pool: PgPool) {
    let live = insert_image(&pool, NewImage::default()).await;
    let trashed = insert_image(
        &pool,
        NewImage {
            deleted_at: Some(utc(2026, 2, 1, 0, 0, 0)),
            ..Default::default()
        },
    )
    .await;

    let count = batch::update(
        &pool,
        &[live, trashed],
        Some(4),
        Some(true),
        None,
        None,
        None,
    )
    .await
    .expect("batch update");

    assert_eq!(count, 1, "soft-deleted images are not updated");

    let (rating, favorite): (i16, bool) =
        sqlx::query_as("SELECT rating, is_favorite FROM images WHERE id = $1")
            .bind(live)
            .fetch_one(&pool)
            .await
            .expect("fetch updated row");
    assert_eq!((rating, favorite), (4, true));
}

/// With no fields and no tag operations there is nothing to do — and in
/// particular no `UPDATE images SET` with an empty SET clause.
#[sqlx::test(migrations = "./migrations")]
async fn update_with_nothing_to_do_is_a_noop(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;

    let count = batch::update(&pool, &[id], None, None, None, None, None)
        .await
        .expect("batch update");
    assert_eq!(count, 0);

    let empty_tags = batch::update(&pool, &[id], None, None, None, Some(&[]), Some(&[]))
        .await
        .expect("batch update");
    assert_eq!(empty_tags, 0, "empty tag lists mean 'no tag operation'");
}

/// Adding tags preserves the existing order, appends only new ones, and never
/// duplicates.
#[sqlx::test(migrations = "./migrations")]
async fn add_tags_merges_without_duplicates(pool: PgPool) {
    let id = insert_image(
        &pool,
        NewImage {
            user_tags: json!(["keep", "existing"]),
            ..Default::default()
        },
    )
    .await;

    let count = batch::update(
        &pool,
        &[id],
        None,
        None,
        None,
        Some(&["existing".to_string(), "fresh".to_string()]),
        None,
    )
    .await
    .expect("batch update");

    assert_eq!(count, 1);
    assert_eq!(tags_of(&pool, id).await, vec!["keep", "existing", "fresh"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn remove_tags_drops_only_the_listed_ones(pool: PgPool) {
    let id = insert_image(
        &pool,
        NewImage {
            user_tags: json!(["a", "b", "c"]),
            ..Default::default()
        },
    )
    .await;

    batch::update(
        &pool,
        &[id],
        None,
        None,
        None,
        None,
        Some(&["b".to_string(), "missing".to_string()]),
    )
    .await
    .expect("batch update");

    assert_eq!(tags_of(&pool, id).await, vec!["a", "c"]);
}

/// A remove+add in the same call removes first, so a tag present in both lists
/// ends up added (and moved to the end).
#[sqlx::test(migrations = "./migrations")]
async fn remove_runs_before_add(pool: PgPool) {
    let id = insert_image(
        &pool,
        NewImage {
            user_tags: json!(["x", "y"]),
            ..Default::default()
        },
    )
    .await;

    batch::update(
        &pool,
        &[id],
        None,
        None,
        None,
        Some(&["x".to_string()]),
        Some(&["x".to_string()]),
    )
    .await
    .expect("batch update");

    assert_eq!(tags_of(&pool, id).await, vec!["y", "x"]);
}

/// Tag updates can carry scalar changes in the same call (the tag path has its
/// own UPDATE statement, so this is easy to break).
#[sqlx::test(migrations = "./migrations")]
async fn tag_update_also_applies_scalar_fields(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;

    batch::update(
        &pool,
        &[id],
        Some(5),
        None,
        Some(true),
        Some(&["tagged".to_string()]),
        None,
    )
    .await
    .expect("batch update");

    let (rating, needs): (i16, bool) =
        sqlx::query_as("SELECT rating, needs_improvement FROM images WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("fetch updated row");
    assert_eq!((rating, needs), (5, true));
    assert_eq!(tags_of(&pool, id).await, vec!["tagged"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn soft_delete_only_affects_live_images(pool: PgPool) {
    let live = insert_image(&pool, NewImage::default()).await;
    let already = insert_image(
        &pool,
        NewImage {
            deleted_at: Some(utc(2026, 2, 1, 0, 0, 0)),
            ..Default::default()
        },
    )
    .await;

    let count = batch::soft_delete(&pool, &[live, already])
        .await
        .expect("soft delete");

    assert_eq!(count, 1, "an already-trashed image is not counted again");
    assert!(is_deleted(&pool, live).await);
}

#[sqlx::test(migrations = "./migrations")]
async fn restore_only_affects_deleted_images(pool: PgPool) {
    let live = insert_image(&pool, NewImage::default()).await;
    let trashed = insert_image(
        &pool,
        NewImage {
            deleted_at: Some(utc(2026, 2, 1, 0, 0, 0)),
            ..Default::default()
        },
    )
    .await;

    let count = batch::restore(&pool, &[live, trashed])
        .await
        .expect("restore");

    assert_eq!(count, 1);
    assert!(!is_deleted(&pool, trashed).await);
}

/// Permanent delete ignores the trash state and returns the object paths so the
/// caller can clean up storage.
#[sqlx::test(migrations = "./migrations")]
async fn delete_permanent_removes_rows_and_returns_paths(pool: PgPool) {
    let live = insert_image(&pool, NewImage::default()).await;
    let trashed = insert_image(
        &pool,
        NewImage {
            deleted_at: Some(utc(2026, 2, 1, 0, 0, 0)),
            ..Default::default()
        },
    )
    .await;

    let paths = batch::delete_permanent(&pool, &[live, trashed])
        .await
        .expect("permanent delete");

    assert_eq!(paths.len(), 2, "both live and trashed rows are removed");
    for (storage_path, thumbnail_path) in &paths {
        assert!(storage_path.ends_with(".png"));
        assert!(thumbnail_path.starts_with("thumbnails/"));
    }

    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM images")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(remaining, 0);
}

/// Ids that do not exist are simply not matched — no error, no partial write.
#[sqlx::test(migrations = "./migrations")]
async fn unknown_ids_are_ignored(pool: PgPool) {
    let id = insert_image(&pool, NewImage::default()).await;
    let ghost = uuid::Uuid::now_v7();

    let count = batch::update(&pool, &[id, ghost], Some(1), None, None, None, None)
        .await
        .expect("batch update");
    assert_eq!(count, 1);

    let deleted = batch::soft_delete(&pool, &[ghost]).await.expect("delete");
    assert_eq!(deleted, 0);
}
