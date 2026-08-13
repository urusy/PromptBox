//! PromptBox Rust backend entry point.
//!
//! All logic lives in the `promptbox` library crate (`src/lib.rs`); this file
//! only wires up tracing, config, the pool, the worker and the HTTP server.

use anyhow::Result;
use promptbox::{config, db, http, job, storage, worker};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cfg = Arc::new(config::Config::load()?);
    let listen_addr = cfg.listen_addr.clone();

    let pool = db::create_pool(&cfg.database_url).await?;

    // Apply pending schema migrations. `migrations/` is the single source of
    // truth: on an empty database it builds the whole schema from
    // 20260711000000_initial_schema.sql; on an existing one every version is
    // already recorded in _sqlx_migrations and this is a no-op.
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

    // Jobs that were running when the process last stopped can never finish;
    // mark them so clients stop polling a row that will never change.
    let jobs = job::Jobs::new(pool.clone());
    match jobs.recover_interrupted().await {
        Ok(0) => {}
        Ok(n) => tracing::warn!(count = n, "marked unfinished jobs as interrupted"),
        Err(e) => tracing::error!(error = %e, "failed to sweep unfinished jobs"),
    }

    let state = http::AppState {
        config: cfg,
        pool,
        storage: store,
        jobs,
    };
    let app = http::router(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!(addr = %listen_addr, "starting server");

    // ConnectInfo carries the peer address, which the login rate limiter falls
    // back to when no proxy header is present (direct access, or a
    // misconfigured reverse proxy).
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
