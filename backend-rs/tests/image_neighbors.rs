//! Integration tests for prev/next navigation (`image::neighbors`).
//!
//! The row-value comparison `(sort_col, id) <cmp> (subquery)` is subtle enough
//! that a direction mix-up would swap the arrows in the detail view without any
//! error — and the same construct is the basis for keyset pagination (docs/13
//! A8), so it is pinned here in all four directions.

mod common;

use common::{add_to_showcase, insert_sequence, insert_showcase, NewImage};
use promptbox::image::{self, SearchParams};
use sqlx::PgPool;

/// Newest-first listing: `prev` is the newer image, `next` is the older one.
#[sqlx::test(migrations = "./migrations")]
async fn desc_listing_prev_is_newer(pool: PgPool) {
    let seq = insert_sequence(&pool, 3).await; // [oldest, middle, newest]
    let p = SearchParams {
        sort_order: "desc".to_string(),
        ..Default::default()
    };

    let (prev, next) = image::neighbors(&pool, &p, seq[1]).await.expect("neighbors");
    assert_eq!(prev, Some(seq[2]), "prev goes towards the top of the list");
    assert_eq!(next, Some(seq[0]), "next goes towards the bottom");
}

/// Oldest-first listing reverses both directions.
#[sqlx::test(migrations = "./migrations")]
async fn asc_listing_prev_is_older(pool: PgPool) {
    let seq = insert_sequence(&pool, 3).await;
    let p = SearchParams {
        sort_order: "asc".to_string(),
        ..Default::default()
    };

    let (prev, next) = image::neighbors(&pool, &p, seq[1]).await.expect("neighbors");
    assert_eq!(prev, Some(seq[0]));
    assert_eq!(next, Some(seq[2]));
}

/// The first and last images have no neighbour on the outward side.
#[sqlx::test(migrations = "./migrations")]
async fn edges_have_no_neighbour(pool: PgPool) {
    let seq = insert_sequence(&pool, 3).await;
    let p = SearchParams::default(); // created_at DESC

    let (prev_newest, next_newest) =
        image::neighbors(&pool, &p, seq[2]).await.expect("neighbors");
    assert_eq!(prev_newest, None, "newest image is the first in a DESC list");
    assert_eq!(next_newest, Some(seq[1]));

    let (prev_oldest, next_oldest) =
        image::neighbors(&pool, &p, seq[0]).await.expect("neighbors");
    assert_eq!(prev_oldest, Some(seq[1]));
    assert_eq!(next_oldest, None, "oldest image is the last in a DESC list");
}

/// Navigation stays inside the current search context: an image filtered out of
/// the listing is skipped rather than being stepped onto.
#[sqlx::test(migrations = "./migrations")]
async fn neighbours_respect_the_active_filters(pool: PgPool) {
    let rated_old = common::insert_image(
        &pool,
        NewImage {
            rating: 5,
            created_at: common::utc(2026, 1, 1, 0, 0, 0),
            ..Default::default()
        },
    )
    .await;
    let unrated_middle = common::insert_image(
        &pool,
        NewImage {
            rating: 0,
            created_at: common::utc(2026, 1, 1, 0, 1, 0),
            ..Default::default()
        },
    )
    .await;
    let rated_new = common::insert_image(
        &pool,
        NewImage {
            rating: 5,
            created_at: common::utc(2026, 1, 1, 0, 2, 0),
            ..Default::default()
        },
    )
    .await;

    let unfiltered = SearchParams {
        sort_order: "asc".to_string(),
        ..Default::default()
    };
    let (_, next) = image::neighbors(&pool, &unfiltered, rated_old)
        .await
        .expect("neighbors");
    assert_eq!(next, Some(unrated_middle));

    let filtered = SearchParams {
        sort_order: "asc".to_string(),
        min_rating: Some(5),
        ..Default::default()
    };
    let (_, next_filtered) = image::neighbors(&pool, &filtered, rated_old)
        .await
        .expect("neighbors");
    assert_eq!(
        next_filtered,
        Some(rated_new),
        "the unrated image is not part of the filtered listing"
    );
}

/// Soft-deleted images are never stepped onto from the normal listing.
#[sqlx::test(migrations = "./migrations")]
async fn deleted_images_are_skipped(pool: PgPool) {
    let first = common::insert_image(
        &pool,
        NewImage {
            created_at: common::utc(2026, 1, 1, 0, 0, 0),
            ..Default::default()
        },
    )
    .await;
    common::insert_image(
        &pool,
        NewImage {
            created_at: common::utc(2026, 1, 1, 0, 1, 0),
            deleted_at: Some(common::utc(2026, 2, 1, 0, 0, 0)),
            ..Default::default()
        },
    )
    .await;
    let third = common::insert_image(
        &pool,
        NewImage {
            created_at: common::utc(2026, 1, 1, 0, 2, 0),
            ..Default::default()
        },
    )
    .await;

    let p = SearchParams {
        sort_order: "asc".to_string(),
        ..Default::default()
    };
    let (_, next) = image::neighbors(&pool, &p, first).await.expect("neighbors");
    assert_eq!(next, Some(third));
}

/// Inside a showcase, navigation follows the curated `sort_order` instead of
/// the listing sort — even when the listing sort would give a different order.
#[sqlx::test(migrations = "./migrations")]
async fn showcase_navigation_follows_curated_order(pool: PgPool) {
    let seq = insert_sequence(&pool, 3).await; // oldest .. newest
    let showcase = insert_showcase(&pool, "curated").await;
    // Curated order is deliberately the reverse of creation order.
    add_to_showcase(&pool, showcase, &[seq[2], seq[1], seq[0]]).await;

    let p = SearchParams {
        showcase_id: Some(showcase),
        sort_order: "desc".to_string(),
        ..Default::default()
    };

    let (prev, next) = image::neighbors(&pool, &p, seq[1]).await.expect("neighbors");
    assert_eq!(prev, Some(seq[2]), "previous slot in the curated order");
    assert_eq!(next, Some(seq[0]), "next slot in the curated order");
}

/// An image that is not a member of the showcase has no neighbours.
#[sqlx::test(migrations = "./migrations")]
async fn showcase_navigation_for_non_member(pool: PgPool) {
    let seq = insert_sequence(&pool, 2).await;
    let showcase = insert_showcase(&pool, "curated").await;
    add_to_showcase(&pool, showcase, &[seq[0]]).await;

    let p = SearchParams {
        showcase_id: Some(showcase),
        ..Default::default()
    };
    let (prev, next) = image::neighbors(&pool, &p, seq[1]).await.expect("neighbors");
    assert_eq!((prev, next), (None, None));
}

/// The showcase filter also restricts the listing itself.
#[sqlx::test(migrations = "./migrations")]
async fn showcase_filter_restricts_listing(pool: PgPool) {
    let seq = insert_sequence(&pool, 3).await;
    let showcase = insert_showcase(&pool, "curated").await;
    add_to_showcase(&pool, showcase, &[seq[0], seq[2]]).await;

    let result = image::list(
        &pool,
        &SearchParams {
            showcase_id: Some(showcase),
            ..Default::default()
        },
    )
    .await
    .expect("list");
    assert_eq!(result.total, 2);
}
