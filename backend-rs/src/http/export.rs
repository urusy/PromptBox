//! Export HTTP handlers (mirror endpoints/export.py).
//!
//! Uses `axum_extra::extract::Query` (serde_html_form) so repeated `ids` query
//! parameters (`?ids=a&ids=b`) deserialize into a Vec, matching the Python
//! `Query(None)` list contract.

use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use axum_extra::extract::Query;
use uuid::Uuid;

use super::auth::CurrentUser;
use super::AppState;
use crate::error::AppError;
use crate::{export, image};

#[derive(Debug, serde::Deserialize)]
pub struct ExportQuery {
    #[serde(default)]
    pub ids: Vec<Uuid>,
    #[serde(default = "default_format")]
    pub export_format: String,
}

fn default_format() -> String {
    "json".to_string()
}

/// GET /api/export/metadata?ids=&export_format=json|csv
pub async fn export_metadata(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let ids = (!q.ids.is_empty()).then_some(q.ids.as_slice());
    let rows = image::list_for_export(&state.pool, ids).await?;
    let export_rows: Vec<_> = rows.into_iter().map(export::to_export_row).collect();

    let (content, media_type, filename) = if q.export_format == "csv" {
        (
            export::to_csv(&export_rows),
            "text/csv",
            "comfyui_gallery_export.csv",
        )
    } else {
        (
            export::to_json(&export_rows),
            "application/json",
            "comfyui_gallery_export.json",
        )
    };
    Ok(file_response(content, media_type, filename))
}

/// GET /api/export/prompts?ids=
pub async fn export_prompts(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let ids = (!q.ids.is_empty()).then_some(q.ids.as_slice());
    let rows = image::list_for_export(&state.pool, ids).await?;
    let content = export::prompts_text(&rows);
    Ok(file_response(content, "text/plain", "prompts.txt"))
}

/// Build an attachment response with the given body, content type, and filename.
fn file_response(content: String, media_type: &str, filename: &str) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, media_type.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename={filename}"),
            ),
        ],
        content,
    )
}
