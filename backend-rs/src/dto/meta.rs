//! Response shapes for the service-identity endpoints (docs/13 B14).

use serde::Serialize;

/// `GET /api/version` — who this backend is. Unauthenticated: Falcon needs to
/// be able to check compatibility before it has a session.
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    /// Crate version (Cargo.toml).
    pub version: String,
    /// Short commit the binary was built from, or "unknown".
    pub git_sha: String,
    /// Build time, RFC3339 UTC.
    pub built_at: String,
    /// Highest applied sqlx migration version, or null if the database is
    /// unreachable (this endpoint stays up either way).
    pub schema_version: Option<i64>,
    /// Metadata parser generation (`parser::VERSION`).
    pub parser_version: i32,
}

/// `GET /api/config` — how this backend is configured. Requires a session: it
/// exposes operational settings (never secrets).
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    /// Capability flags a client can branch on.
    pub features: Vec<&'static str>,
    pub limits: Limits,
    /// Object storage backend in use: "s3" or "fs".
    pub storage_backend: String,
    /// Longest edge of the generated thumbnails, in pixels.
    pub thumbnail_sizes: Vec<u32>,
}

/// Hard limits a client must respect. Mirrored from the handlers so callers do
/// not have to hard-code them (Falcon chunks bulk requests by `bulk_max_ids`).
#[derive(Debug, Serialize)]
pub struct Limits {
    pub max_per_page: i64,
    pub default_per_page: i64,
    pub bulk_max_ids: usize,
}
