//! HTTP layer: shared state, router, middleware.

pub mod auth;
pub mod health;
pub mod images;

use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::config::Config;

/// Shared application state, cloned into every handler via `State`.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
}

/// Build the HTTP handler tree.
pub fn router(state: AppState) -> Router {
    let cors = build_cors(&state.config);

    // Static serving for originals and thumbnails. In production nginx serves
    // /storage directly; this exists for direct access and the Falcon
    // DownloadImage integration test (GET baseURL + storage path).
    let storage = ServeDir::new(state.config.storage_path.clone());

    let api = Router::new()
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/images", get(images::list_images))
        .route("/images/{id}", get(images::get_image));

    Router::new()
        .route("/", get(health::root))
        .route("/health", get(health::health))
        .route("/health/db", get(health::health_db))
        .nest("/api", api)
        .nest_service("/storage", storage)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// CORS configured from the comma-separated CORS_ORIGINS list, with credentials
/// enabled. Headers mirror the request so credentials + dynamic headers work.
fn build_cors(cfg: &Config) -> CorsLayer {
    let origins: Vec<HeaderValue> = cfg
        .cors_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(AllowHeaders::mirror_request())
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(300))
}
