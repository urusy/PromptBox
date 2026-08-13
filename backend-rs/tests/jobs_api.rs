//! Integration tests for the background job infrastructure (docs/13 B5).
//!
//! Seven planned features (re-parse, pHash backfill, GC, trash purge, stats
//! refresh, …) will run on this, so the guarantees it makes — progress is
//! visible, cancellation works, a crash never leaves a job "running" forever —
//! are pinned here rather than per feature.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use common::{session_cookie, test_router};
use promptbox::job::{Jobs, KIND_NOOP, STATUS_INTERRUPTED};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Duration;
use tower::ServiceExt;

async fn send(
    router: Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder()
        .uri(uri)
        .method(method)
        .header("content-type", "application/json")
        .header("cookie", session_cookie());
    let request = match body {
        Some(v) => builder.body(Body::from(v.to_string())),
        None => builder.body(Body::empty()),
    }
    .expect("build request");

    let response = router.oneshot(request).await.expect("router response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read body");
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// Poll a job until it reaches a terminal state (or the attempts run out).
async fn wait_for_terminal(router: &Router, id: &str) -> Value {
    for _ in 0..100 {
        let (_, job) = send(router.clone(), "GET", &format!("/api/jobs/{id}"), None).await;
        let status = job["status"].as_str().unwrap_or_default();
        if matches!(
            status,
            "succeeded" | "failed" | "cancelled" | "interrupted"
        ) {
            return job;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("job {id} never reached a terminal state");
}

#[sqlx::test(migrations = "./migrations")]
async fn creating_a_job_returns_202_and_runs_it(pool: PgPool) {
    let router = test_router(pool);

    let (status, job) = send(
        router.clone(),
        "POST",
        "/api/jobs",
        Some(json!({"kind": KIND_NOOP, "params": {"steps": 3}})),
    )
    .await;

    assert_eq!(status, StatusCode::ACCEPTED, "the work is accepted, not done");
    assert_eq!(job["kind"], KIND_NOOP);
    assert_eq!(job["status"], "queued");

    let id = job["id"].as_str().expect("job id").to_string();
    let finished = wait_for_terminal(&router, &id).await;

    assert_eq!(finished["status"], "succeeded");
    assert_eq!(finished["progress_current"], 3);
    assert_eq!(finished["progress_total"], 3);
    assert_eq!(finished["result"]["steps"], 3);
    assert!(finished["started_at"].is_string());
    assert!(finished["finished_at"].is_string());
}

/// Accepting work the backend cannot do would leave a row nobody ever
/// finishes, so unknown kinds are rejected up front.
#[sqlx::test(migrations = "./migrations")]
async fn unknown_kind_is_rejected(pool: PgPool) {
    let (status, body) = send(
        test_router(pool),
        "POST",
        "/api/jobs",
        Some(json!({"kind": "reticulate_splines"})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["detail"]
            .as_str()
            .is_some_and(|d| d.contains(KIND_NOOP)),
        "the error lists the kinds that do exist: {body}"
    );
}

/// Progress must be observable while the job is still running — that is the
/// whole point of the table.
#[sqlx::test(migrations = "./migrations")]
async fn progress_is_visible_while_running(pool: PgPool) {
    let router = test_router(pool);

    let (_, job) = send(
        router.clone(),
        "POST",
        "/api/jobs",
        Some(json!({"kind": KIND_NOOP, "params": {"steps": 20, "sleep_ms": 25}})),
    )
    .await;
    let id = job["id"].as_str().expect("job id").to_string();

    // `progress_total` is published by the job itself, just after it starts, so
    // the two observations are tracked separately rather than assumed to be
    // visible in the same snapshot.
    let mut saw_running = false;
    let mut saw_total = false;
    let mut saw_partial_progress = false;
    for _ in 0..100 {
        let (_, snapshot) = send(router.clone(), "GET", &format!("/api/jobs/{id}"), None).await;
        saw_running |= snapshot["status"] == "running";
        saw_total |= snapshot["progress_total"] == 20;
        saw_partial_progress |= (1..20).contains(&snapshot["progress_current"].as_i64().unwrap_or(0));
        if saw_running && saw_total && saw_partial_progress {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(saw_running, "the job never reported itself as running");
    assert!(saw_total, "the job never published how much work it had");
    assert!(
        saw_partial_progress,
        "progress was never observable mid-flight"
    );

    wait_for_terminal(&router, &id).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_running_job_can_be_cancelled(pool: PgPool) {
    let router = test_router(pool);

    let (_, job) = send(
        router.clone(),
        "POST",
        "/api/jobs",
        Some(json!({"kind": KIND_NOOP, "params": {"steps": 200, "sleep_ms": 20}})),
    )
    .await;
    let id = job["id"].as_str().expect("job id").to_string();

    let (cancel_status, _) = send(
        router.clone(),
        "POST",
        &format!("/api/jobs/{id}/cancel"),
        None,
    )
    .await;
    assert_eq!(cancel_status, StatusCode::OK);

    let finished = wait_for_terminal(&router, &id).await;
    assert_eq!(finished["status"], "cancelled");
    assert!(
        finished["progress_current"].as_i64().unwrap_or(0) < 200,
        "cancellation stopped it early: {finished}"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn listing_filters_by_status(pool: PgPool) {
    let router = test_router(pool);

    for _ in 0..3 {
        let (_, job) = send(
            router.clone(),
            "POST",
            "/api/jobs",
            Some(json!({"kind": KIND_NOOP, "params": {"steps": 1}})),
        )
        .await;
        let id = job["id"].as_str().expect("job id").to_string();
        wait_for_terminal(&router, &id).await;
    }

    let (status, all) = send(router.clone(), "GET", "/api/jobs", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(all.as_array().expect("array").len(), 3);

    let (_, succeeded) = send(router.clone(), "GET", "/api/jobs?status=succeeded", None).await;
    assert_eq!(succeeded.as_array().expect("array").len(), 3);

    let (_, failed) = send(router, "GET", "/api/jobs?status=failed", None).await;
    assert!(failed.as_array().expect("array").is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn missing_job_is_404(pool: PgPool) {
    let ghost = uuid::Uuid::now_v7();
    let (status, _) = send(
        test_router(pool),
        "GET",
        &format!("/api/jobs/{ghost}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// A process that dies mid-job cannot update its own row. Without the startup
/// sweep those rows stay `running` and clients poll them forever.
#[sqlx::test(migrations = "./migrations")]
async fn startup_sweep_marks_abandoned_jobs_as_interrupted(pool: PgPool) {
    // Simulate the rows a crashed process leaves behind.
    for status in ["queued", "running"] {
        sqlx::query("INSERT INTO jobs (id, kind, status) VALUES ($1, $2, $3)")
            .bind(uuid::Uuid::now_v7())
            .bind(KIND_NOOP)
            .bind(status)
            .execute(&pool)
            .await
            .expect("insert stale job");
    }
    // …plus one that finished properly and must not be touched.
    let done = uuid::Uuid::now_v7();
    sqlx::query("INSERT INTO jobs (id, kind, status, finished_at) VALUES ($1, $2, 'succeeded', NOW())")
        .bind(done)
        .bind(KIND_NOOP)
        .execute(&pool)
        .await
        .expect("insert finished job");

    let swept = Jobs::new(pool.clone())
        .recover_interrupted()
        .await
        .expect("sweep");
    assert_eq!(swept, 2);

    let interrupted: i64 =
        sqlx::query_scalar("SELECT count(*) FROM jobs WHERE status = $1")
            .bind(STATUS_INTERRUPTED)
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(interrupted, 2);

    let untouched: String = sqlx::query_scalar("SELECT status FROM jobs WHERE id = $1")
        .bind(done)
        .fetch_one(&pool)
        .await
        .expect("fetch finished job");
    assert_eq!(untouched, "succeeded", "terminal jobs are left alone");
}

/// Jobs are heavy sweeps; running two at once would just make both slower and
/// compete for the connection pool. The semaphore is what prevents that.
#[sqlx::test(migrations = "./migrations")]
async fn jobs_do_not_run_concurrently(pool: PgPool) {
    let router = test_router(pool.clone());

    let mut ids = Vec::new();
    for _ in 0..3 {
        let (_, job) = send(
            router.clone(),
            "POST",
            "/api/jobs",
            Some(json!({"kind": KIND_NOOP, "params": {"steps": 10, "sleep_ms": 15}})),
        )
        .await;
        ids.push(job["id"].as_str().expect("job id").to_string());
    }

    // Sample the number of simultaneously running jobs while they work through
    // the queue.
    let mut max_running = 0i64;
    for _ in 0..60 {
        let running: i64 =
            sqlx::query_scalar("SELECT count(*) FROM jobs WHERE status = 'running'")
                .fetch_one(&pool)
                .await
                .expect("count running");
        max_running = max_running.max(running);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    assert!(
        max_running <= 1,
        "at most one job may run at a time, saw {max_running}"
    );

    for id in ids {
        wait_for_terminal(&router, &id).await;
    }
}
