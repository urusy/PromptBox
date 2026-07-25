//! HTTP layer: shared state, router, middleware.

pub mod auth;
pub mod bulk;
pub mod changes;
pub mod duplicates;
pub mod export;
pub mod gelbooru;
pub mod health;
pub mod images;
pub mod jobs;
pub mod loras;
pub mod manifest;
pub mod meta;
pub mod models;
pub mod presets;
pub mod showcases;
pub mod smart_folders;
pub mod stats;
pub mod storage;
pub mod tags;
pub mod warnings;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{HeaderValue, Method, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::Router;
use object_store::ObjectStore;
use sqlx::PgPool;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;

/// Largest JSON request accepted. The biggest legitimate body is a bulk
/// operation with 500 UUIDs plus tags — well under 100 KB.
const MAX_JSON_BODY_BYTES: usize = 1024 * 1024;

/// How long a JSON request may run before the client gets a 408. Object
/// streaming is excluded (see `router`).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Concurrent JSON requests in flight. Beyond this, requests queue rather than
/// pile onto the 30-connection database pool.
const MAX_CONCURRENT_JSON_REQUESTS: usize = 64;

/// Shared application state, cloned into every handler via `State`.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub storage: Arc<dyn ObjectStore>,
    /// Background job registry (concurrency permit + cancellation flags).
    pub jobs: Arc<crate::job::Jobs>,
}

/// Build the HTTP handler tree.
pub fn router(state: AppState) -> Router {
    let cors = build_cors(&state.config);

    let api = Router::new()
        // The Python backend served health under /api; keep both spellings so
        // existing monitoring keeps working after the cutover.
        .route("/health", get(health::health))
        .route("/health/db", get(health::health_db))
        // Service identity. /version is unauthenticated (compatibility check
        // before login); /config requires a session.
        .route("/version", get(meta::version))
        .route("/config", get(meta::config))
        // Route table for clients that mirror this router (Falcon). See
        // manifest.rs — a test keeps it in sync with the calls below.
        .route("/_manifest", get(manifest::manifest))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/images", get(images::list_images))
        .route(
            "/images/{id}",
            get(images::get_image)
                .patch(images::update_image)
                .delete(images::delete_image),
        )
        .route("/images/{id}/restore", post(images::restore_image))
        .route(
            "/search-presets",
            get(presets::list_presets).post(presets::create_preset),
        )
        .route(
            "/search-presets/{id}",
            put(presets::update_preset).delete(presets::delete_preset),
        )
        .route(
            "/smart-folders",
            get(smart_folders::list_folders).post(smart_folders::create_folder),
        )
        .route(
            "/smart-folders/{id}",
            get(smart_folders::get_folder)
                .put(smart_folders::update_folder)
                .delete(smart_folders::delete_folder),
        )
        .route("/tags", get(tags::list_tags))
        .route("/changes", get(changes::list_changes))
        .route("/jobs", get(jobs::list_jobs).post(jobs::create_job))
        .route("/jobs/{id}", get(jobs::get_job))
        .route("/jobs/{id}/cancel", post(jobs::cancel_job))
        .route("/bulk/update", post(bulk::batch_update))
        .route("/bulk/delete", post(bulk::batch_delete))
        .route("/bulk/restore", post(bulk::batch_restore))
        .route(
            "/duplicates",
            get(duplicates::get_info).delete(duplicates::delete_all),
        )
        .route("/duplicates/{filename}", delete(duplicates::delete_file))
        .route("/export/metadata", get(export::export_metadata))
        .route("/export/prompts", get(export::export_prompts))
        .route(
            "/showcases",
            get(showcases::list_showcases).post(showcases::create_showcase),
        )
        .route("/showcases/check-images", post(showcases::check_images))
        .route(
            "/showcases/{id}",
            get(showcases::get_showcase)
                .put(showcases::update_showcase)
                .delete(showcases::delete_showcase),
        )
        .route(
            "/showcases/{id}/images",
            post(showcases::add_images).delete(showcases::remove_images),
        )
        .route(
            "/showcases/{id}/images/reorder",
            put(showcases::reorder_images),
        )
        .route("/stats", get(stats::get_stats))
        .route(
            "/stats/models-for-analysis",
            get(stats::models_for_analysis),
        )
        .route("/stats/loras-for-filter", get(stats::loras_for_filter))
        .route(
            "/stats/samplers-for-filter",
            get(stats::samplers_for_filter),
        )
        .route("/stats/rating-analysis", get(stats::rating_analysis))
        .route(
            "/stats/model-rating-distribution",
            get(stats::model_rating_distribution),
        )
        .route("/models", get(models::get_models))
        .route("/models/{model_name}/detail", get(models::get_model_detail))
        .route("/models/{model_name}/civitai", get(models::get_model_civitai))
        .route("/loras", get(loras::get_loras))
        .route("/loras/{lora_name}/detail", get(loras::get_lora_detail))
        .route("/loras/{lora_name}/civitai", get(loras::get_lora_civitai))
        .route("/gelbooru/tags", get(gelbooru::search_tags));

    // Login is the one endpoint an anonymous caller can hammer, so it gets its
    // own bucket: ~10 attempts up front, then one every 2 seconds per client.
    // SmartIpKeyExtractor reads X-Forwarded-For / X-Real-IP (set by nginx and
    // by Cloudflare for tunnelled traffic) before falling back to the peer
    // address, which main.rs supplies via ConnectInfo.
    let login_rate_limit = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(10)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        .expect("valid login rate-limit config");
    let api = api.merge(
        Router::new()
            .route("/auth/login", post(auth::login))
            .layer(GovernorLayer::new(login_rate_limit)),
    );

    // JSON endpoints: bounded body, bounded time, bounded concurrency.
    let json_routes = Router::new()
        .route("/", get(health::root))
        .route("/health", get(health::health))
        .route("/health/db", get(health::health_db))
        .nest("/api", api)
        .layer(RequestBodyLimitLayer::new(MAX_JSON_BODY_BYTES))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(GlobalConcurrencyLimitLayer::new(MAX_CONCURRENT_JSON_REQUESTS));

    // Originals and thumbnails, streamed from object storage. In production
    // nginx proxies /storage/ here; also used directly by the Falcon
    // DownloadImage integration (GET baseURL + storage path).
    //
    // Deliberately outside the layers above: a 10 MB original on a slow client
    // would trip the request timeout, and image-heavy pages open far more
    // concurrent requests than the API limit allows.
    let storage_routes = Router::new().route("/storage/{*path}", get(storage::serve));

    json_routes
        .merge(storage_routes)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    /// Building the full router must not panic on route conflicts. `connect_lazy`
    /// constructs the pool without opening a connection (but needs a Tokio
    /// context for its background task), so no database is required.
    #[tokio::test]
    async fn router_builds_without_conflicts() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://user:pass@localhost/db")
            .expect("lazy pool construction");
        let config = Config::for_test();
        let storage = crate::storage::build(&config).expect("fs store");
        let jobs = crate::job::Jobs::new(pool.clone());
        let state = AppState {
            config: Arc::new(config),
            pool,
            storage,
            jobs,
        };
        let _ = router(state);
    }
}
