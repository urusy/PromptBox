//! HTTP layer: shared state, router, middleware.

pub mod auth;
pub mod bulk;
pub mod duplicates;
pub mod export;
pub mod gelbooru;
pub mod health;
pub mod images;
pub mod loras;
pub mod models;
pub mod presets;
pub mod showcases;
pub mod smart_folders;
pub mod stats;
pub mod storage;
pub mod tags;

use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use axum::routing::{delete, get, post, put};
use axum::Router;
use object_store::ObjectStore;
use sqlx::PgPool;
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Config;

/// Shared application state, cloned into every handler via `State`.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub storage: Arc<dyn ObjectStore>,
}

/// Build the HTTP handler tree.
pub fn router(state: AppState) -> Router {
    let cors = build_cors(&state.config);

    let api = Router::new()
        // The Python backend served health under /api; keep both spellings so
        // existing monitoring keeps working after the cutover.
        .route("/health", get(health::health))
        .route("/health/db", get(health::health_db))
        .route("/auth/login", post(auth::login))
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

    Router::new()
        .route("/", get(health::root))
        .route("/health", get(health::health))
        .route("/health/db", get(health::health_db))
        .nest("/api", api)
        // Originals and thumbnails, streamed from object storage. In production
        // nginx proxies /storage/ here; also used directly by the Falcon
        // DownloadImage integration (GET baseURL + storage path).
        .route("/storage/{*path}", get(storage::serve))
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
        let state = AppState {
            config: Arc::new(config),
            pool,
            storage,
        };
        let _ = router(state);
    }
}
