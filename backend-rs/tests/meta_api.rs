//! Integration tests for the service identity endpoints (docs/13 B14).

mod common;

use axum::http::StatusCode;
use common::{get_json, session_cookie, test_router};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn version_is_public_and_reports_the_build(pool: PgPool) {
    let (status, body) = get_json(test_router(pool), "/api/version", None).await;

    assert_eq!(status, StatusCode::OK, "no session required");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    assert!(
        body["git_sha"].as_str().is_some_and(|s| !s.is_empty()),
        "git_sha falls back to \"unknown\" but is never empty"
    );
    assert!(
        body["built_at"].as_str().is_some_and(|s| s.contains('T')),
        "built_at is RFC3339, got {:?}",
        body["built_at"]
    );
    assert_eq!(body["parser_version"], promptbox::parser::VERSION);
}

/// The reported schema version must be the newest migration actually applied —
/// this is how Falcon detects that it is talking to an older deployment.
#[sqlx::test(migrations = "./migrations")]
async fn version_reports_the_applied_schema_version(pool: PgPool) {
    let expected: i64 = sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("read applied migrations");

    let (_, body) = get_json(test_router(pool), "/api/version", None).await;

    assert_eq!(body["schema_version"], expected);
}

#[sqlx::test(migrations = "./migrations")]
async fn config_requires_a_session(pool: PgPool) {
    let (status, _) = get_json(test_router(pool), "/api/config", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn config_reports_limits_and_features(pool: PgPool) {
    let (status, body) = get_json(
        test_router(pool),
        "/api/config",
        Some(&session_cookie()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // These must match the clamp in list_images and the guard in validate_ids.
    assert_eq!(body["limits"]["max_per_page"], 120);
    assert_eq!(body["limits"]["default_per_page"], 24);
    assert_eq!(body["limits"]["bulk_max_ids"], 500);

    let features = body["features"].as_array().expect("features array");
    assert!(features.iter().any(|f| f == "images"));
    assert!(
        !features.iter().any(|f| f == "import_worker"),
        "the test config runs without the worker, so the flag must be absent"
    );
    assert!(
        !features.iter().any(|f| f == "gelbooru"),
        "gelbooru needs credentials, which the test config has none of"
    );

    assert_eq!(body["storage_backend"], "fs");
    assert_eq!(body["thumbnail_sizes"][0], 300);
}

/// Configuration is not a place to leak credentials.
#[sqlx::test(migrations = "./migrations")]
async fn config_exposes_no_secrets(pool: PgPool) {
    let (_, body) = get_json(
        test_router(pool),
        "/api/config",
        Some(&session_cookie()),
    )
    .await;

    let serialized = body.to_string().to_lowercase();
    for forbidden in [
        "secret",
        "password",
        "access_key",
        "database_url",
        "postgres://",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "config response must not mention {forbidden:?}: {serialized}"
        );
    }
}
