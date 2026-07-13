//! PromptBox Rust backend entry point.

mod auth;
mod batch;
mod cache;
mod catalog;
mod civitai;
mod config;
mod db;
mod dto;
mod duplicate;
mod error;
mod export;
mod gelbooru;
mod http;
mod image;
mod media;
mod parser;
mod preset;
mod showcase;
mod smart_folder;
mod stats;
mod storage;
mod tag;
mod util;
mod worker;

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cfg = Arc::new(config::Config::load()?);
    let listen_addr = cfg.listen_addr.clone();

    let pool = db::create_pool(&cfg.database_url).await?;

    // Apply pending schema migrations (mirror the Python entrypoint's
    // `alembic upgrade head`). The baseline is a no-op — the initial schema
    // comes from db/init/*.sql on a fresh database.
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("database migrations up to date");

    let store = storage::build(&cfg)?;
    // Reachability probe only — a missing key is fine, an unreachable MinIO is
    // worth a warning (imports will retry, serving will 502 until it's up).
    match store
        .head(&object_store::path::Path::from("startup-probe"))
        .await
    {
        Ok(_) | Err(object_store::Error::NotFound { .. }) => {
            tracing::info!(backend = %cfg.storage_backend, "object storage reachable");
        }
        Err(e) => {
            tracing::warn!(backend = %cfg.storage_backend, error = %e, "object storage unreachable at startup");
        }
    }

    // Background import worker (folder watch + periodic scan). Disable via
    // WATCHER_ENABLED=false (e.g. while the Python backend still owns imports).
    if cfg.watcher_enabled {
        worker::spawn(pool.clone(), cfg.clone(), store.clone());
    }

    let state = http::AppState {
        config: cfg,
        pool,
        storage: store,
    };
    let app = http::router(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!(addr = %listen_addr, "starting server");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,promptbox=debug,tower_http=info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
