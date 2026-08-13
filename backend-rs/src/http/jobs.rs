//! Background job endpoints (docs/13 B5).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::error::AppError;
use crate::job::{JobRow, KINDS};

#[derive(Debug, serde::Deserialize)]
pub struct CreateJobRequest {
    pub kind: String,
    /// Kind-specific parameters; defaults to `{}`.
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ListJobsQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

/// POST /api/jobs — accept the work and return 202 with the queued job. The
/// job runs in the background; poll `GET /api/jobs/{id}` for progress.
pub async fn create_job(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<JobRow>), AppError> {
    // Reject unknown kinds instead of storing work that will never run.
    if !KINDS.contains(&req.kind.as_str()) {
        return Err(AppError::BadRequest(format!(
            "unknown job kind {:?}; supported kinds: {}",
            req.kind,
            KINDS.join(", ")
        )));
    }

    let params = req.params.unwrap_or_else(|| Value::Object(Default::default()));
    let job = state.jobs.enqueue(&req.kind, params).await?;
    Ok((StatusCode::ACCEPTED, Json(job)))
}

/// GET /api/jobs
pub async fn list_jobs(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ListJobsQuery>,
) -> Result<Json<Vec<JobRow>>, AppError> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let jobs = state.jobs.list(q.status.as_deref(), limit).await?;
    Ok(Json(jobs))
}

/// GET /api/jobs/{id}
pub async fn get_job(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobRow>, AppError> {
    state
        .jobs
        .get(id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))
}

/// POST /api/jobs/{id}/cancel
///
/// Cancellation is cooperative: a queued job is cancelled immediately, a
/// running one stops at its next checkpoint, so the returned row may still say
/// `running`.
pub async fn cancel_job(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobRow>, AppError> {
    state
        .jobs
        .cancel(id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))
}
