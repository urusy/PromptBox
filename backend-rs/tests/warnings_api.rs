//! Integration tests for the "don't silently drop things" protocol (docs/13 A3).
//!
//! The failure this prevents is real: Falcon sent `sampler=` where the API
//! expects `sampler_name=`, the filter never applied, and nothing anywhere
//! said so.

mod common;

use axum::http::StatusCode;
use common::{insert_image, session_cookie, test_router, NewImage};
use promptbox::http::warnings::HEADER;
use serde_json::Value;
use sqlx::PgPool;

/// GET with a session, returning (status, body, warnings header). The shared
/// `get_json` helper drops headers, and here they are the point.
async fn get_images(pool: PgPool, query: &str) -> (StatusCode, Value, Option<String>) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let router = test_router(pool);
    let uri = format!("/api/images?{query}");

    let response = router
        .oneshot(
            Request::builder()
                .uri(&uri)
                .method("GET")
                .header("cookie", session_cookie())
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router response");

    let status = response.status();
    let header = response
        .headers()
        .get(HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read body");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body, header)
}

/// A well-formed request must look exactly as it did before this feature: no
/// `warnings` key, no header.
#[sqlx::test(migrations = "./migrations")]
async fn clean_request_carries_no_warnings(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await;

    let (status, body, header) = get_images(pool, "page=1&per_page=10&sort_by=rating").await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.get("warnings").is_none(),
        "the key is omitted when empty so existing clients are unaffected"
    );
    assert_eq!(header, None);
}

/// The regression that motivated A3.
#[sqlx::test(migrations = "./migrations")]
async fn unknown_parameter_is_reported_with_a_hint(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await;

    let (status, body, header) = get_images(pool, "sampler=euler").await;

    assert_eq!(status, StatusCode::OK, "the request still succeeds");
    let warnings = body["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0]["code"], "unknown_param");
    assert_eq!(warnings[0]["param"], "sampler");
    assert_eq!(warnings[0]["hint"], "did you mean sampler_name?");
    assert!(
        header.is_some_and(|h| h.contains("sampler")),
        "the same information is available in the header"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn clamped_per_page_is_reported(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await;

    let (_, body, _) = get_images(pool, "per_page=1000").await;

    let warnings = body["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings[0]["code"], "clamped");
    assert_eq!(warnings[0]["param"], "per_page");
    assert_eq!(body["per_page"], 120, "the clamp itself is unchanged");
}

#[sqlx::test(migrations = "./migrations")]
async fn unsupported_sort_column_is_reported(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await;

    let (_, body, _) = get_images(pool, "sort_by=seed").await;

    let warnings = body["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings[0]["code"], "fallback");
    assert_eq!(warnings[0]["param"], "sort_by");
    assert!(
        warnings[0]["hint"]
            .as_str()
            .is_some_and(|h| h.contains("created_at")),
        "the hint lists the columns that are allowed"
    );
}

/// Falcon's `sort=` alias is a supported spelling and must not warn.
#[sqlx::test(migrations = "./migrations")]
async fn falcon_sort_alias_is_not_a_warning(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await;

    let (_, body, _) = get_images(pool, "sort=rating&order=asc").await;

    assert!(body.get("warnings").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn strict_mode_turns_warnings_into_400(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await;

    let (status, body, _) = get_images(pool.clone(), "sampler=euler&strict=true").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["detail"]
            .as_str()
            .is_some_and(|d| d.contains("sampler")),
        "the error says which parameter was wrong: {body}"
    );

    let (ok_status, _, _) = get_images(pool, "sampler_name=euler&strict=true").await;
    assert_eq!(
        ok_status,
        StatusCode::OK,
        "strict mode passes a correct request through"
    );
}

/// Several problems in one request are all reported, not just the first.
#[sqlx::test(migrations = "./migrations")]
async fn multiple_warnings_accumulate(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await;

    let (_, body, _) = get_images(pool, "sampler=euler&per_page=999&sort_by=nope").await;

    let codes: Vec<&str> = body["warnings"]
        .as_array()
        .expect("warnings array")
        .iter()
        .map(|w| w["code"].as_str().unwrap_or_default())
        .collect();
    assert!(codes.contains(&"unknown_param"));
    assert!(codes.contains(&"clamped"));
    assert!(codes.contains(&"fallback"));
}
