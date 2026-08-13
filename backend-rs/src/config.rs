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
    /// Forces the `Secure` attribute on the session cookie. `None` (the
    /// default) means "decide per request from the forwarded scheme", which is
    /// what lets the same deployment serve both the plain-HTTP LAN origin and
    /// the HTTPS Cloudflare tunnel.
    pub session_cookie_secure: Option<bool>,

    // Paths
    pub import_path: String,
    // Local filesystem root for the `fs` storage backend (rollback path); also
    // kept for parity with the Python config.
    pub storage_path: String,

    // Object storage. `storage_backend` selects "s3" (MinIO, production) or
    // "fs" (local directory, rollback/offline dev).
    pub storage_backend: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,

    // Import worker (mirror the Python backend's import_* settings).
    // Filename substrings excluded from import (lowercased); matches are moved
    // to `import/<import_skipped_dir>/` instead of processed. Empty by default:
    // grids used to be excluded here, but they are now imported and tagged
    // instead (see `import_grid_patterns`).
    pub import_skip_patterns: Vec<String>,
    // Filename substrings (lowercased) that mark an image as a grid even when
    // its metadata carries no `Script: X/Y/Z plot` — PNG info disabled, or the
    // file re-saved by another tool. Matches set `model_params.is_xyz_grid`;
    // the image is still imported normally.
    pub import_grid_patterns: Vec<String>,
    // Above this pixel count the in-memory decode is skipped in favour of the
    // streaming (scanline) thumbnailer. A full RGBA decode costs ~4 bytes per
    // pixel, so 150MP is already ~600MB against a 1GB container limit.
    pub import_full_decode_max_pixels: u64,
    // Hard ceiling on pixel count. Streaming makes memory a non-issue, so this
    // only stops decoder bombs and corrupt headers from burning CPU forever.
    pub import_max_pixels: u64,
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
            session_cookie_secure: get_opt_bool("SESSION_COOKIE_SECURE"),
            import_path: get("IMPORT_PATH", "/app/import"),
            storage_path: get("STORAGE_PATH", "/app/storage"),
            storage_backend: get("STORAGE_BACKEND", "s3").to_lowercase(),
            s3_endpoint: get("S3_ENDPOINT", "http://minio:9000"),
            s3_bucket: get("S3_BUCKET", "promptbox"),
            s3_access_key: get("S3_ACCESS_KEY", ""),
            s3_secret_key: get("S3_SECRET_KEY", ""),
            import_skip_patterns: split_csv(&get("IMPORT_SKIP_PATTERNS", ""))
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect(),
            import_grid_patterns: split_csv(&get("IMPORT_GRID_PATTERNS", "xyz_grid,^grid-"))
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect(),
            import_full_decode_max_pixels: get_int("IMPORT_FULL_DECODE_MAX_PIXELS", 150_000_000)
                .max(1) as u64,
            import_max_pixels: get_int("IMPORT_MAX_PIXELS", 2_000_000_000).max(1) as u64,
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
        cfg.validate_storage()?;
        Ok(cfg)
    }

    /// The s3 backend needs credentials; fail fast at startup instead of on the
    /// first import.
    fn validate_storage(&self) -> Result<()> {
        match self.storage_backend.as_str() {
            "s3" => {
                if self.s3_access_key.is_empty() || self.s3_secret_key.is_empty() {
                    bail!(
                        "STORAGE_BACKEND=s3 requires S3_ACCESS_KEY and S3_SECRET_KEY \
                         (set them to the MinIO credentials)"
                    );
                }
                Ok(())
            }
            "fs" => Ok(()),
            other => bail!("STORAGE_BACKEND must be \"s3\" or \"fs\", got {other:?}"),
        }
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

impl Config {
    /// Minimal configuration for tests that build app components without
    /// touching the database or authenticating. Not `#[cfg(test)]` because the
    /// integration tests in `tests/` are a separate crate and need it too.
    pub fn for_test() -> Self {
        Config {
            database_url: "postgres://user:pass@localhost/db".to_string(),
            admin_username: "admin".to_string(),
            admin_password_hash: String::new(),
            secret_key: "x".repeat(32),
            session_expire_hours: 24,
            session_cookie_secure: None,
            import_path: "/tmp/import".to_string(),
            storage_path: "/tmp/storage".to_string(),
            storage_backend: "fs".to_string(),
            s3_endpoint: String::new(),
            s3_bucket: String::new(),
            s3_access_key: String::new(),
            s3_secret_key: String::new(),
            import_skip_patterns: Vec::new(),
            import_grid_patterns: vec!["xyz_grid".to_string(), "^grid-".to_string()],
            import_full_decode_max_pixels: 150_000_000,
            import_max_pixels: 2_000_000_000,
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

/// Tri-state boolean: unset/empty means "not configured" rather than `false`,
/// so callers can distinguish an explicit override from the default behaviour.
/// An unparsable value is treated as unset and warned about.
fn get_opt_bool(key: &str) -> Option<bool> {
    let raw = std::env::var(key).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    match parse_bool(raw) {
        Some(v) => Some(v),
        None => {
            tracing::warn!("{key}={raw:?} is not a boolean; ignoring it");
            None
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_csv_drops_empty_entries() {
        // An empty IMPORT_SKIP_PATTERNS must mean "exclude nothing", not a
        // single empty pattern that would match every filename.
        assert!(split_csv("").is_empty());
        assert!(split_csv(" , ,").is_empty());
        assert_eq!(split_csv("xyz_grid, grid-"), vec!["xyz_grid", "grid-"]);
    }

    #[test]
    fn test_config_defaults_import_grids_instead_of_skipping_them() {
        let cfg = Config::for_test();
        assert!(cfg.import_skip_patterns.is_empty());
        assert!(cfg.import_grid_patterns.contains(&"xyz_grid".to_string()));
        assert!(cfg.import_full_decode_max_pixels < cfg.import_max_pixels);
    }
}
