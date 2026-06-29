//! PostgreSQL connection pool.

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Create a connection pool and verify connectivity.
///
/// Pool sizing mirrors the Python backend (database.py: pool_size=10,
/// max_overflow=20, pool_recycle=300s) by capping total connections at 30 and
/// recycling idle connections after 5 minutes.
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(30)
        .min_connections(2)
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .context("connect to database")?;
    Ok(pool)
}
