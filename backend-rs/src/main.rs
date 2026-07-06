//! PromptBox Rust backend entry point.

mod auth;
mod batch;
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

    // Background import worker (folder watch + periodic scan). Disable via
    // WATCHER_ENABLED=false (e.g. while the Python backend still owns imports).
    if cfg.watcher_enabled {
        worker::spawn(pool.clone(), cfg.clone());
    }

    let state = http::AppState {
        config: cfg,
        pool,
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
