//! Lightweight background jobs (docs/13 B5).
//!
//! Everything long-running that this backend will grow — re-parsing after a
//! parser fix, pHash backfill, orphan GC, trash purge, statistics refresh —
//! has the same shape: start it over HTTP, poll its progress, maybe cancel it.
//! Building that once means those features are job kinds, not seven separate
//! endpoints with seven progress mechanisms.
//!
//! Scope is deliberately small, because PromptBox is a single instance:
//!
//!   * one `jobs` row per run, updated in place — no message broker,
//!   * a `Semaphore(1)` so heavy jobs never run concurrently with each other,
//!   * cancellation as a flag the running job checks (cooperative), and
//!   * a startup sweep that marks anything left `running` as `interrupted`,
//!     because a process that died mid-job cannot report on itself.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_SUCCEEDED: &str = "succeeded";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_CANCELLED: &str = "cancelled";
pub const STATUS_INTERRUPTED: &str = "interrupted";

/// A no-op job used to exercise the machinery (progress, cancellation,
/// serialisation) without needing one of the real kinds to exist yet.
/// Parameters: `steps` (default 3) and `sleep_ms` (default 0) per step.
pub const KIND_NOOP: &str = "noop";

/// Job kinds this build can run. `POST /api/jobs` rejects anything else with a
/// 400 rather than accepting work it will never do.
pub const KINDS: &[&str] = &[KIND_NOOP];

/// One row of the `jobs` table.
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct JobRow {
    pub id: Uuid,
    pub kind: String,
    pub status: String,
    pub params: Value,
    pub progress_current: i64,
    pub progress_total: Option<i64>,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

/// How a job finished.
enum Outcome {
    Succeeded(Value),
    Cancelled,
    Failed(String),
}

/// Job registry: owns the concurrency permit and the cancellation flags of
/// currently running jobs.
pub struct Jobs {
    pool: PgPool,
    /// One at a time. Jobs are I/O- and CPU-heavy sweeps over the whole
    /// library; running two at once would just make both slower and fight over
    /// the connection pool.
    permits: Semaphore,
    cancels: Mutex<HashMap<Uuid, Arc<AtomicBool>>>,
}

impl Jobs {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self {
            pool,
            permits: Semaphore::new(1),
            cancels: Mutex::new(HashMap::new()),
        })
    }

    /// Mark jobs left behind by a crash or restart. A process that died while
    /// running a job cannot update its own row, so without this sweep those
    /// rows stay `running` forever and clients poll them indefinitely.
    ///
    /// Returns how many rows were swept.
    pub async fn recover_interrupted(&self) -> Result<u64, sqlx::Error> {
        let affected = sqlx::query(
            "UPDATE jobs SET status = $1, finished_at = NOW(), \
             error = COALESCE(error, 'interrupted by a backend restart') \
             WHERE status IN ($2, $3)",
        )
        .bind(STATUS_INTERRUPTED)
        .bind(STATUS_QUEUED)
        .bind(STATUS_RUNNING)
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(affected)
    }

    /// Record a job and start it in the background. Returns immediately with
    /// the `queued` row.
    pub async fn enqueue(
        self: &Arc<Self>,
        kind: &str,
        params: Value,
    ) -> Result<JobRow, sqlx::Error> {
        let row: JobRow = sqlx::query_as(
            "INSERT INTO jobs (id, kind, status, params) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(Uuid::now_v7())
        .bind(kind)
        .bind(STATUS_QUEUED)
        .bind(&params)
        .fetch_one(&self.pool)
        .await?;

        // Register the cancellation flag *before* spawning. Doing it inside the
        // task would lose a cancel that arrives while the task is still waiting
        // for the concurrency permit: the flag would not exist yet, and the row
        // would no longer be `queued` by the time the task looked.
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancels.lock().await.insert(row.id, Arc::clone(&cancel));

        let registry = Arc::clone(self);
        let id = row.id;
        let kind = row.kind.clone();
        let params = row.params.clone();
        tokio::spawn(async move {
            registry.run(id, kind, params, cancel).await;
        });

        Ok(row)
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<JobRow>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM jobs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// Most recent jobs first, optionally filtered by status.
    pub async fn list(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<JobRow>, sqlx::Error> {
        match status {
            Some(s) => {
                sqlx::query_as(
                    "SELECT * FROM jobs WHERE status = $1 ORDER BY created_at DESC LIMIT $2",
                )
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as("SELECT * FROM jobs ORDER BY created_at DESC LIMIT $1")
                    .bind(limit)
                    .fetch_all(&self.pool)
                    .await
            }
        }
    }

    /// Cancel a job. A `queued` job is cancelled outright; a `running` one is
    /// asked to stop at its next checkpoint (cooperative — a job that never
    /// checks would keep going, which is why every kind must check).
    /// Terminal jobs are returned unchanged.
    pub async fn cancel(&self, id: Uuid) -> Result<Option<JobRow>, sqlx::Error> {
        if let Some(flag) = self.cancels.lock().await.get(&id) {
            flag.store(true, Ordering::Relaxed);
        }

        // Only queued jobs flip here; a running one is finalised by its own
        // task so the result/finished_at stay consistent.
        sqlx::query(
            "UPDATE jobs SET status = $1, finished_at = NOW() WHERE id = $2 AND status = $3",
        )
        .bind(STATUS_CANCELLED)
        .bind(id)
        .bind(STATUS_QUEUED)
        .execute(&self.pool)
        .await?;

        self.get(id).await
    }

    /// Body of the spawned task: take the permit, run the kind, record how it
    /// ended. Errors are stored on the row rather than propagated — nobody is
    /// awaiting this task.
    async fn run(self: Arc<Self>, id: Uuid, kind: String, params: Value, cancel: Arc<AtomicBool>) {
        let _permit = match self.permits.acquire().await {
            Ok(permit) => permit,
            // Only happens if the semaphore is closed, i.e. during shutdown.
            Err(_) => return,
        };

        // Claim the job. A cancel that landed while we waited for the permit
        // has already moved it out of `queued`, so this affects zero rows and
        // there is nothing to run.
        let claimed = sqlx::query(
            "UPDATE jobs SET status = $1, started_at = NOW() WHERE id = $2 AND status = $3",
        )
        .bind(STATUS_RUNNING)
        .bind(id)
        .bind(STATUS_QUEUED)
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .unwrap_or(false);

        if !claimed {
            self.cancels.lock().await.remove(&id);
            return;
        }

        // Cancelled between the claim and here (or while queueing): stop before
        // doing any work.
        if cancel.load(Ordering::Relaxed) {
            self.finish(id, Outcome::Cancelled).await;
            self.cancels.lock().await.remove(&id);
            return;
        }

        let outcome = match kind.as_str() {
            KIND_NOOP => run_noop(&self.pool, id, &params, &cancel).await,
            // enqueue() validates the kind, so this is only reachable if a row
            // was written by hand.
            other => Outcome::Failed(format!("unknown job kind {other:?}")),
        };

        self.finish(id, outcome).await;
        self.cancels.lock().await.remove(&id);
    }

    /// Write the terminal state of a job.
    async fn finish(&self, id: Uuid, outcome: Outcome) {
        let query = match outcome {
            Outcome::Succeeded(result) => sqlx::query(
                "UPDATE jobs SET status = $1, result = $2, finished_at = NOW() WHERE id = $3",
            )
            .bind(STATUS_SUCCEEDED)
            .bind(result)
            .bind(id),
            Outcome::Cancelled => {
                sqlx::query("UPDATE jobs SET status = $1, finished_at = NOW() WHERE id = $2")
                    .bind(STATUS_CANCELLED)
                    .bind(id)
            }
            Outcome::Failed(message) => sqlx::query(
                "UPDATE jobs SET status = $1, error = $2, finished_at = NOW() WHERE id = $3",
            )
            .bind(STATUS_FAILED)
            .bind(message)
            .bind(id),
        };

        if let Err(e) = query.execute(&self.pool).await {
            tracing::error!(job_id = %id, error = %e, "failed to record job outcome");
        }
    }
}

/// Publish how much work a job has in total (before it starts doing it).
async fn set_total(pool: &PgPool, id: Uuid, total: i64) {
    let _ = sqlx::query("UPDATE jobs SET progress_total = $1 WHERE id = $2")
        .bind(total)
        .bind(id)
        .execute(pool)
        .await;
}

/// Publish progress. Best-effort: a failed progress write must not fail the job.
async fn set_progress(pool: &PgPool, id: Uuid, current: i64) {
    let _ = sqlx::query("UPDATE jobs SET progress_current = $1 WHERE id = $2")
        .bind(current)
        .bind(id)
        .execute(pool)
        .await;
}

/// The `noop` kind: count to `steps`, sleeping `sleep_ms` between them, and
/// stop early when cancelled. It exists so the infrastructure itself can be
/// tested end to end.
async fn run_noop(pool: &PgPool, id: Uuid, params: &Value, cancel: &AtomicBool) -> Outcome {
    let steps = params
        .get("steps")
        .and_then(Value::as_i64)
        .unwrap_or(3)
        .clamp(1, 10_000);
    let sleep_ms = params
        .get("sleep_ms")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .min(60_000);

    set_total(pool, id, steps).await;

    for step in 1..=steps {
        if cancel.load(Ordering::Relaxed) {
            return Outcome::Cancelled;
        }
        if sleep_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
        }
        set_progress(pool, id, step).await;
    }

    Outcome::Succeeded(json!({ "steps": steps }))
}
