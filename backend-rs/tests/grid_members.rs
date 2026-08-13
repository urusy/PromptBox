//! Integration tests for `GET /api/images/{id}/grid-members` (docs/16).
//!
//! Nothing in the data links a grid to the images it was built from, so the
//! endpoint infers membership. These tests pin that inference: what counts as a
//! member, what must stay out, and how the reported confidence follows from
//! what could be matched.

mod common;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use common::{get_json, insert_image, session_cookie, test_router, utc, NewImage};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

/// When the montage was saved. Cells are generated before this moment.
fn grid_time() -> DateTime<Utc> {
    utc(2026, 3, 1, 12, 0, 0)
}

/// The settings a grid and its cells share (everything the axes do not vary).
fn base(filename: &str) -> NewImage {
    NewImage {
        original_filename: filename.to_string(),
        model_name: Some("animagine_v31".to_string()),
        sampler_name: Some("Euler a".to_string()),
        steps: Some(20),
        cfg_scale: Some(7.0),
        seed: Some(1234),
        created_at: grid_time(),
        ..Default::default()
    }
}

/// The grid image itself, carrying the axis metadata under test.
async fn insert_grid(pool: &PgPool, model_params: Value) -> Uuid {
    insert_image(
        pool,
        NewImage {
            model_params,
            ..base("xyz_grid-0001.png")
        },
    )
    .await
}

/// One cell: the base settings with a CFG value applied, generated
/// `minutes_before` the montage was written.
async fn insert_cell(pool: &PgPool, filename: &str, cfg: f64, minutes_before: i64) -> Uuid {
    insert_image(
        pool,
        NewImage {
            cfg_scale: Some(cfg),
            created_at: grid_time() - Duration::minutes(minutes_before),
            ..base(filename)
        },
    )
    .await
}

/// Fetch the members of `grid`, with an optional query string.
async fn members_of(pool: PgPool, grid: Uuid, query: &str) -> (StatusCode, Value) {
    let uri = format!("/api/images/{grid}/grid-members{query}");
    get_json(test_router(pool), &uri, Some(&session_cookie())).await
}

/// Member ids in response order.
fn member_ids(body: &Value) -> Vec<String> {
    body["members"]
        .as_array()
        .expect("members array")
        .iter()
        .map(|m| m["id"].as_str().expect("member id").to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// The happy path
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn members_are_matched_by_axis_values_and_shared_parameters(pool: PgPool) {
    let grid = insert_grid(
        &pool,
        json!({
            "is_xyz_grid": true,
            "xyz_x_type": "CFG Scale",
            "xyz_x_values": "5,7,9",
        }),
    )
    .await;

    let cfg5 = insert_cell(&pool, "cell-cfg5.png", 5.0, 30).await;
    let cfg7 = insert_cell(&pool, "cell-cfg7.png", 7.0, 25).await;
    let cfg9 = insert_cell(&pool, "cell-cfg9.png", 9.0, 20).await;

    // A different run: same settings, different seed — the invariant the grid
    // did not vary rules it out.
    insert_image(
        &pool,
        NewImage {
            cfg_scale: Some(5.0),
            seed: Some(999),
            created_at: grid_time() - Duration::minutes(10),
            ..base("other-run.png")
        },
    )
    .await;
    // A CFG value that is not on the axis.
    insert_cell(&pool, "cell-cfg12.png", 12.0, 15).await;
    // Yesterday's identical experiment, outside the 24h window.
    insert_cell(&pool, "yesterday.png", 5.0, 25 * 60).await;
    // Another grid — grids are never members.
    insert_image(
        &pool,
        NewImage {
            cfg_scale: Some(5.0),
            created_at: grid_time() - Duration::minutes(5),
            model_params: json!({ "is_xyz_grid": true }),
            ..base("xyz_grid-0002.png")
        },
    )
    .await;

    let (status, body) = members_of(pool, grid, "").await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(body["matched"], 3);
    assert_eq!(body["expected_cells"], 3);
    assert_eq!(body["confidence"], "exact");
    assert_eq!(body["window_hours"], 24);
    // Ordered along the axis, not by time.
    assert_eq!(
        member_ids(&body),
        vec![cfg5.to_string(), cfg7.to_string(), cfg9.to_string()]
    );
    assert_eq!(body["members"][0]["position"]["x"], 0);
    assert_eq!(body["members"][2]["position"]["x"], 2);
    assert_eq!(body["members"][2]["axis_values"]["x"], "9");
    // The grid itself rides along so a client can render from one request.
    assert_eq!(body["grid"]["id"], grid.to_string());
    assert_eq!(body["axes"]["x"]["type"], "CFG Scale");
    assert_eq!(body["axes"]["x"]["column"], "cfg_scale");
    assert!(body["axes"]["y"].is_null());
    // Members carry the normal list fields, so the same card component works.
    assert!(body["members"][0]["thumbnail_url"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn two_axes_place_members_on_both(pool: PgPool) {
    let grid = insert_grid(
        &pool,
        json!({
            "is_xyz_grid": true,
            "xyz_x_type": "CFG Scale",
            "xyz_x_values": "5,7",
            "xyz_y_type": "Sampler",
            "xyz_y_values": "\"Euler a\", \"DPM++ 2M\"",
        }),
    )
    .await;

    for (cfg, sampler, minutes) in [
        (5.0, "Euler a", 40),
        (7.0, "Euler a", 35),
        (5.0, "DPM++ 2M", 30),
        (7.0, "DPM++ 2M", 25),
    ] {
        insert_image(
            &pool,
            NewImage {
                cfg_scale: Some(cfg),
                sampler_name: Some(sampler.to_string()),
                created_at: grid_time() - Duration::minutes(minutes),
                ..base(&format!("cell-{cfg}-{sampler}.png"))
            },
        )
        .await;
    }

    let (status, body) = members_of(pool, grid, "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["matched"], 4);
    assert_eq!(body["expected_cells"], 4);
    assert_eq!(body["confidence"], "exact");

    // Reading order: rows (y) first, columns (x) within a row.
    let positions: Vec<(i64, i64)> = body["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["position"]["x"].as_i64().unwrap(),
                m["position"]["y"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(positions, vec![(0, 0), (1, 0), (0, 1), (1, 1)]);
    assert_eq!(body["members"][2]["axis_values"]["y"], "DPM++ 2M");
}

// ---------------------------------------------------------------------------
// Partial and unknown answers
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_cell_makes_the_answer_partial(pool: PgPool) {
    let grid = insert_grid(
        &pool,
        json!({
            "is_xyz_grid": true,
            "xyz_x_type": "CFG Scale",
            "xyz_x_values": "5,7,9",
        }),
    )
    .await;

    insert_cell(&pool, "cell-cfg5.png", 5.0, 30).await;
    insert_cell(&pool, "cell-cfg7.png", 7.0, 25).await;
    insert_image(
        &pool,
        NewImage {
            cfg_scale: Some(9.0),
            created_at: grid_time() - Duration::minutes(20),
            deleted_at: Some(grid_time()),
            ..base("cell-cfg9-trashed.png")
        },
    )
    .await;

    let (status, body) = members_of(pool, grid, "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["matched"], 2);
    assert_eq!(body["expected_cells"], 3);
    assert_eq!(body["confidence"], "partial");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_grid_without_axis_metadata_reports_nothing_rather_than_guessing(pool: PgPool) {
    // Tagged by filename during import (PNG info disabled), so the flag is
    // there but the axes are not.
    let grid = insert_grid(&pool, json!({ "is_xyz_grid": true })).await;
    insert_cell(&pool, "some-image.png", 7.0, 10).await;

    let (status, body) = members_of(pool, grid, "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["confidence"], "none");
    assert_eq!(body["matched"], 0);
    assert!(body["axes"].is_null());
    assert!(body["expected_cells"].is_null());
    assert_eq!(body["warnings"][0]["code"], "no_axis_metadata");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unsupported_axis_type_falls_back_to_the_invariants(pool: PgPool) {
    let grid = insert_grid(
        &pool,
        json!({
            "is_xyz_grid": true,
            "xyz_x_type": "Prompt S/R",
            "xyz_x_values": "cat, dog",
        }),
    )
    .await;

    // Same everything (the axis varies the prompt, which is not a column the
    // matcher filters on), so both are candidates.
    insert_cell(&pool, "cell-cat.png", 7.0, 20).await;
    insert_cell(&pool, "cell-dog.png", 7.0, 15).await;
    // A different seed is still ruled out by the invariants.
    insert_image(
        &pool,
        NewImage {
            seed: Some(4321),
            created_at: grid_time() - Duration::minutes(10),
            ..base("other-run.png")
        },
    )
    .await;

    let (status, body) = members_of(pool, grid, "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["confidence"], "heuristic");
    assert_eq!(body["matched"], 2);
    // The cell count cannot be checked when an axis type is not understood.
    assert!(body["expected_cells"].is_null());
    assert_eq!(body["warnings"][0]["code"], "unsupported_axis_type");
    assert_eq!(body["axes"]["x"]["column"], Value::Null);
}

// ---------------------------------------------------------------------------
// The time window
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn the_window_bounds_how_far_back_cells_are_taken_from(pool: PgPool) {
    let grid = insert_grid(
        &pool,
        json!({
            "is_xyz_grid": true,
            "xyz_x_type": "CFG Scale",
            "xyz_x_values": "5,7",
        }),
    )
    .await;

    let recent = insert_cell(&pool, "cell-recent.png", 5.0, 30).await;
    // Three hours earlier: inside the default window, outside a one-hour one.
    insert_cell(&pool, "cell-old.png", 7.0, 180).await;

    let (_, wide) = members_of(pool.clone(), grid, "").await;
    assert_eq!(wide["matched"], 2);
    assert_eq!(wide["confidence"], "exact");

    let (status, narrow) = members_of(pool, grid, "?window_hours=1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(narrow["window_hours"], 1);
    assert_eq!(narrow["matched"], 1);
    assert_eq!(member_ids(&narrow), vec![recent.to_string()]);
    assert_eq!(narrow["confidence"], "partial");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_out_of_range_window_is_clamped_and_reported(pool: PgPool) {
    let grid = insert_grid(
        &pool,
        json!({
            "is_xyz_grid": true,
            "xyz_x_type": "Steps",
            "xyz_x_values": "20,30",
        }),
    )
    .await;

    let (status, body) = members_of(pool, grid, "?window_hours=0").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["window_hours"], 1);
    assert_eq!(body["warnings"][0]["code"], "clamped");
}

// ---------------------------------------------------------------------------
// Contract errors
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn asking_a_normal_image_for_its_members_is_a_400(pool: PgPool) {
    let image = insert_image(&pool, NewImage::default()).await;
    let (status, _) = members_of(pool, image, "").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_missing_image_is_a_404(pool: PgPool) {
    let (status, _) = members_of(pool, Uuid::now_v7(), "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn members_require_a_session(pool: PgPool) {
    let grid = insert_grid(&pool, json!({ "is_xyz_grid": true })).await;
    let uri = format!("/api/images/{grid}/grid-members");
    let (status, _) = get_json(test_router(pool), &uri, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn unknown_parameters_warn_and_fail_under_strict(pool: PgPool) {
    let grid = insert_grid(
        &pool,
        json!({
            "is_xyz_grid": true,
            "xyz_x_type": "Steps",
            "xyz_x_values": "20,30",
        }),
    )
    .await;

    let (status, body) = members_of(pool.clone(), grid, "?windowhours=3").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["warnings"][0]["code"], "unknown_param");
    assert_eq!(body["warnings"][0]["param"], "windowhours");

    let (status, _) = members_of(pool, grid, "?windowhours=3&strict=true").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
