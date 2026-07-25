//! Integration tests for the request-guard middleware (docs/13 B11).
//!
//! Before this, nothing bounded a request: no body size limit, no timeout, no
//! concurrency cap, and an unlimited number of login attempts.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use common::{session_cookie, test_router};
use sqlx::PgPool;
use tower::ServiceExt;

/// POST with an explicit body, optionally authenticated. `client_ip` feeds the
/// rate limiter's key extractor (nginx sets this header in production).
async fn post(
    router: Router,
    uri: &str,
    body: String,
    cookie: Option<String>,
    client_ip: &str,
) -> StatusCode {
    let mut builder = Request::builder()
        .uri(uri)
        .method("POST")
        .header("content-type", "application/json")
        .header("x-forwarded-for", client_ip);
    if let Some(c) = cookie {
        builder = builder.header("cookie", c);
    }
    router
        .oneshot(builder.body(Body::from(body)).expect("build request"))
        .await
        .expect("router response")
        .status()
}

/// A body over the 1 MB limit is rejected before it reaches the handler.
#[sqlx::test(migrations = "./migrations")]
async fn oversized_json_body_is_rejected(pool: PgPool) {
    let huge = format!(r#"{{"ids":[],"padding":"{}"}}"#, "x".repeat(2 * 1024 * 1024));

    let status = post(
        test_router(pool),
        "/api/bulk/update",
        huge,
        Some(session_cookie()),
        "10.0.0.1",
    )
    .await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

/// A normal-sized body still goes through (the limit is not accidentally tiny).
#[sqlx::test(migrations = "./migrations")]
async fn normal_json_body_passes(pool: PgPool) {
    let ids: Vec<String> = (0..500)
        .map(|_| uuid::Uuid::now_v7().to_string())
        .collect();
    let body = serde_json::json!({ "ids": ids, "rating": 3 }).to_string();
    assert!(body.len() < 1024 * 1024, "the max bulk request fits the limit");

    let status = post(
        test_router(pool),
        "/api/bulk/update",
        body,
        Some(session_cookie()),
        "10.0.0.2",
    )
    .await;

    // 404 = "no such images", which means the request itself was accepted.
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Login is rate-limited per client. The burst is consumed first, then further
/// attempts are refused with 429 — this is the brute-force guard.
#[sqlx::test(migrations = "./migrations")]
async fn login_attempts_are_rate_limited_per_client(pool: PgPool) {
    let router = test_router(pool);
    let credentials = r#"{"username":"admin","password":"wrong"}"#.to_string();

    let mut statuses = Vec::new();
    for _ in 0..20 {
        statuses.push(
            post(
                router.clone(),
                "/api/auth/login",
                credentials.clone(),
                None,
                "203.0.113.10",
            )
            .await,
        );
    }

    assert!(
        statuses.contains(&StatusCode::UNAUTHORIZED),
        "the first attempts are evaluated normally"
    );
    assert!(
        statuses.contains(&StatusCode::TOO_MANY_REQUESTS),
        "a burst of 20 attempts must hit the limit: {statuses:?}"
    );
}

/// The limit is per client, so one attacker must not lock out everyone else.
#[sqlx::test(migrations = "./migrations")]
async fn rate_limit_is_scoped_to_the_client_ip(pool: PgPool) {
    let router = test_router(pool);
    let credentials = r#"{"username":"admin","password":"wrong"}"#.to_string();

    for _ in 0..20 {
        post(
            router.clone(),
            "/api/auth/login",
            credentials.clone(),
            None,
            "203.0.113.20",
        )
        .await;
    }

    let other_client = post(
        router,
        "/api/auth/login",
        credentials,
        None,
        "203.0.113.21",
    )
    .await;

    assert_eq!(
        other_client,
        StatusCode::UNAUTHORIZED,
        "a different client still gets a normal answer"
    );
}

/// Object streaming is deliberately outside the JSON limits: nothing about a
/// GET on /storage/ should be capped by the API's body limit or timeout.
#[sqlx::test(migrations = "./migrations")]
async fn storage_route_is_still_reachable(pool: PgPool) {
    let status = test_router(pool)
        .oneshot(
            Request::builder()
                .uri("/storage/ab/cd/missing.png")
                .method("GET")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router response")
        .status();

    // The object does not exist, but the route answered — it was not blocked.
    assert_eq!(status, StatusCode::NOT_FOUND);
}
