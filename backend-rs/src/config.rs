//! Runtime configuration loaded from environment variables.
//!
//! Mirrors the Python backend's Settings (backend/app/config.py) field for
//! field so the SAME .env works for both implementations during the
//! strangler-fig migration.

use anyhow::{bail, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
// Several fields (import_path, gelbooru_*, thumbnail_*) are consumed in later
// phases (watcher, gelbooru service, thumbnailer).
#[allow(dead_code)]
pub struct Config {
    // Database
    pub database_url: String,

    // Auth
    pub admin_username: String,
    pub admin_password_hash: String,
    pub secret_key: String,
    pub session_expire_hours: i64,

    // Paths
    pub import_path: String,
    pub storage_path: String,

    // Import worker (mirror the Python backend's import_* settings).
    // Filename substrings excluded from import (lowercased); matches are moved
    // to `import/<import_skipped_dir>/` instead of processed.
    pub import_skip_patterns: Vec<String>,
    // After this many *content* failures a file is quarantined to
    // `import/<import_failed_dir>/`. Transient errors (DB/IO) are not counted.
    pub import_max_attempts: u32,
    pub import_failed_dir: String,
    pub import_skipped_dir: String,

    // CORS
    pub cors_origins: Vec<String>,

    // Gelbooru API
    pub gelbooru_api_key: String,
    pub gelbooru_user_id: String,

    // Debug
    pub debug: bool,

    // Thumbnail
    pub thumbnail_size: u32,
    pub thumbnail_quality: u8,

    // HTTP listen address (Rust-specific; not in the Python config).
    pub listen_addr: String,

    // Whether to run the import worker (Rust-specific; lets the Python backend
    // keep ownership of imports during the strangler-fig migration).
    pub watcher_enabled: bool,
}

impl Config {
    /// Load configuration from the environment, apply Python-matching defaults,
    /// normalize the database URL, and validate the secret key.
    pub fn load() -> Result<Self> {
        let debug = get_bool("DEBUG", false);
        let mut cfg = Config {
            database_url: normalize_db_url(&get(
                "DATABASE_URL",
                "postgresql://comfyui:password@db:5432/comfyui_gallery",
            )),
            admin_username: get("ADMIN_USERNAME", "admin"),
            admin_password_hash: get("ADMIN_PASSWORD_HASH", ""),
            secret_key: get("SECRET_KEY", ""),
            session_expire_hours: get_int("SESSION_EXPIRE_HOURS", 24 * 7), // 1 week
            import_path: get("IMPORT_PATH", "/app/import"),
            storage_path: get("STORAGE_PATH", "/app/storage"),
            import_skip_patterns: split_csv(&get("IMPORT_SKIP_PATTERNS", "xyz_grid"))
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect(),
            import_max_attempts: get_int("IMPORT_MAX_ATTEMPTS", 5).max(1) as u32,
            import_failed_dir: get("IMPORT_FAILED_DIR", "failed"),
            import_skipped_dir: get("IMPORT_SKIPPED_DIR", "skipped"),
            cors_origins: split_csv(&get("CORS_ORIGINS", "http://localhost:3000")),
            gelbooru_api_key: get("GELBOORU_API_KEY", ""),
            gelbooru_user_id: get("GELBOORU_USER_ID", ""),
            debug,
            thumbnail_size: get_int("THUMBNAIL_SIZE", 300) as u32,
            thumbnail_quality: get_int("THUMBNAIL_QUALITY", 85) as u8,
            listen_addr: get("LISTEN_ADDR", "0.0.0.0:8000"),
            watcher_enabled: get_bool("WATCHER_ENABLED", true),
        };
        cfg.validate_secret_key()?;
        Ok(cfg)
    }

    /// Require an explicit key in production, generate an ephemeral one in
    /// debug, and enforce a minimum length (mirrors _validate_secret_key).
    fn validate_secret_key(&mut self) -> Result<()> {
        if self.secret_key.is_empty() {
            if !self.debug {
                bail!(
                    "SECRET_KEY must be set explicitly in production (DEBUG=false); \
                     set SECRET_KEY to a random string of at least 32 characters"
                );
            }
            self.secret_key = format!("{}{}", Uuid::now_v7().simple(), Uuid::now_v7().simple());
            tracing::warn!(
                "SECRET_KEY not set; generated an ephemeral key for development. \
                 Sessions will be invalidated on every restart."
            );
        } else if self.secret_key.len() < 32 {
            bail!("SECRET_KEY must be at least 32 characters");
        }
        Ok(())
    }
}

#[cfg(test)]
impl Config {
    /// Minimal configuration for tests that build app components without
    /// touching the database or authenticating.
    pub fn for_test() -> Self {
        Config {
            database_url: "postgres://user:pass@localhost/db".to_string(),
            admin_username: "admin".to_string(),
            admin_password_hash: String::new(),
            secret_key: "x".repeat(32),
            session_expire_hours: 24,
            import_path: "/tmp/import".to_string(),
            storage_path: "/tmp/storage".to_string(),
            import_skip_patterns: vec!["xyz_grid".to_string()],
            import_max_attempts: 5,
            import_failed_dir: "failed".to_string(),
            import_skipped_dir: "skipped".to_string(),
            cors_origins: vec!["http://localhost:3000".to_string()],
            gelbooru_api_key: String::new(),
            gelbooru_user_id: String::new(),
            debug: true,
            thumbnail_size: 300,
            thumbnail_quality: 85,
            listen_addr: "0.0.0.0:8000".to_string(),
            watcher_enabled: false,
        }
    }
}

/// Strip the SQLAlchemy async driver suffix so sqlx can parse the same URL.
fn normalize_db_url(u: &str) -> String {
    u.replacen("postgresql+asyncpg://", "postgresql://", 1)
        .replacen("postgres+asyncpg://", "postgres://", 1)
}

fn get(key: &str, def: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => v,
        _ => def.to_string(),
    }
}

fn get_int(key: &str, def: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(def)
}

fn get_bool(key: &str, def: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => parse_bool(v.trim()).unwrap_or(def),
        Err(_) => def,
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "t" | "yes" | "y" | "on" => Some(true),
        "0" | "false" | "f" | "no" | "n" | "off" | "" => Some(false),
        _ => None,
    }
}

fn split_csv(v: &str) -> Vec<String> {
    v.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}
