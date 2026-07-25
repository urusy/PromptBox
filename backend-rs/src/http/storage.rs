//! GET /storage/{*path} — stream originals and thumbnails from object storage.
//!
//! Replaces the previous ServeDir (and nginx's `alias` in production; nginx now
//! proxies /storage/ here). Unauthenticated on purpose: the Falcon integration
//! fetches originals via bare GETs. Session auth can later be added with a
//! `route_layer` on this single route.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{
    CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, ETAG, IF_NONE_MATCH,
};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use super::AppState;
use crate::error::AppError;
use crate::storage;

/// Mirrors the 30-day immutable caching nginx previously applied to /storage/.
/// Keys are content-addressed, so `immutable` is accurate.
const CACHE_VALUE: &str = "public, max-age=2592000, immutable";

pub async fn serve(
    State(state): State<AppState>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let key = storage::parse_key(&path)?;

    let result = match state.storage.get(&key).await {
        Ok(r) => r,
        Err(object_store::Error::NotFound { .. }) => {
            return Err(AppError::NotFound("file not found".to_string()));
        }
        Err(e) => {
            tracing::error!(key = %key, error = %e, "object storage get failed");
            return Err(AppError::BadGateway("storage unavailable".to_string()));
        }
    };

    let etag = result.meta.e_tag.clone();

    // Conditional GET: with immutable content-addressed keys, any ETag match
    // means the client copy is current.
    if let (Some(tag), Some(inm)) = (
        etag.as_deref(),
        headers.get(IF_NONE_MATCH).and_then(|v| v.to_str().ok()),
    ) && inm.split(',').any(|c| c.trim().trim_start_matches("W/") == tag)
    {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        resp.headers_mut()
            .insert(CACHE_CONTROL, CACHE_VALUE.parse().unwrap());
        return Ok(resp);
    }

    let size = result.meta.size;
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type_for(&path))
        .header(CONTENT_LENGTH, size)
        .header(CACHE_CONTROL, CACHE_VALUE);
    if let Some(tag) = etag {
        resp = resp.header(ETAG, tag);
    }
    Ok(resp
        .body(Body::from_stream(result.into_stream()))
        .map_err(anyhow::Error::from)?)
}

/// Content type from the key's extension. Deliberately independent of any
/// metadata MinIO holds, so objects imported via `mc mirror` behave the same
/// as worker uploads.
fn content_type_for(path: &str) -> &'static str {
    match path.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::routing::get;
    use axum::Router;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_router(dir: &std::path::Path) -> Router {
        let mut config = Config::for_test();
        config.storage_path = dir.to_string_lossy().into_owned();
        let storage = crate::storage::build(&config).expect("fs store");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://user:pass@localhost/db")
            .expect("lazy pool");
        let jobs = crate::job::Jobs::new(pool.clone());
        let state = AppState {
            config: Arc::new(config),
            pool,
            storage,
            jobs,
        };
        Router::new()
            .route("/storage/{*path}", get(serve))
            .with_state(state)
    }

    async fn get_response(router: &Router, uri: &str, if_none_match: Option<&str>) -> Response {
        let mut req = axum::http::Request::builder().uri(uri);
        if let Some(v) = if_none_match {
            req = req.header(IF_NONE_MATCH, v);
        }
        router
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn serves_object_with_headers_and_304() {
        let dir = tempdir();
        std::fs::create_dir_all(dir.join("thumbnails/ab/cd")).unwrap();
        std::fs::write(dir.join("thumbnails/ab/cd/x.webp"), b"RIFFdata").unwrap();
        let router = test_router(&dir);

        let resp = get_response(&router, "/storage/thumbnails/ab/cd/x.webp", None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers()[CONTENT_TYPE], "image/webp");
        assert_eq!(resp.headers()[CACHE_CONTROL], CACHE_VALUE);
        let etag = resp.headers()[ETAG].to_str().unwrap().to_string();

        let resp = get_response(
            &router,
            "/storage/thumbnails/ab/cd/x.webp",
            Some(&etag),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn missing_and_invalid_paths_fail() {
        let dir = tempdir();
        let router = test_router(&dir);

        let resp = get_response(&router, "/storage/ab/cd/missing.png", None).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let resp = get_response(&router, "/storage/ab/../secret.png", None).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Unique-per-test temp dir without extra dev-dependencies.
    fn tempdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "promptbox-storage-test-{}",
            uuid::Uuid::now_v7().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
