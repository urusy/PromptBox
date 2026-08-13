//! Object-storage access for originals and thumbnails.
//!
//! Object keys ARE the relative paths stored in the DB (`ab/cd/<hash>.png`,
//! `thumbnails/ab/cd/<hash>.webp`), so no DB migration is needed and the
//! `/storage/<key>` URL contract (frontend, Falcon) is preserved. Keys are
//! content-addressed, which makes puts idempotent.

use std::sync::Arc;

use anyhow::{bail, Context, Result};
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::path::Path as ObjectPath;
use object_store::ObjectStore;

use crate::config::Config;
use crate::error::AppError;

/// Build the object store selected by STORAGE_BACKEND: "s3" targets MinIO (or
/// any S3-compatible endpoint), "fs" targets the local storage directory with
/// the same key layout (rollback / offline development).
pub fn build(config: &Config) -> Result<Arc<dyn ObjectStore>> {
    match config.storage_backend.as_str() {
        "s3" => {
            let store = AmazonS3Builder::new()
                .with_endpoint(&config.s3_endpoint)
                // MinIO speaks plain HTTP inside the compose network.
                .with_allow_http(true)
                .with_bucket_name(&config.s3_bucket)
                .with_access_key_id(&config.s3_access_key)
                .with_secret_access_key(&config.s3_secret_key)
                // MinIO ignores the region but the SDK requires one.
                .with_region("us-east-1")
                .build()
                .context("build S3 object store")?;
            Ok(Arc::new(store))
        }
        "fs" => {
            std::fs::create_dir_all(&config.storage_path)
                .with_context(|| format!("create storage dir {}", config.storage_path))?;
            let store = LocalFileSystem::new_with_prefix(&config.storage_path)
                .context("build local object store")?;
            Ok(Arc::new(store))
        }
        other => bail!("unknown storage backend {other:?}"),
    }
}

/// Validate an untrusted request path and turn it into an object key.
///
/// S3 keys are opaque strings, but the LocalFileSystem fallback maps keys onto
/// the real filesystem, so `..`/absolute/empty segments must be rejected here
/// (defense shared by both backends).
pub fn parse_key(raw: &str) -> Result<ObjectPath, AppError> {
    if raw.is_empty() || raw.len() > 512 || raw.contains('\0') || raw.starts_with('/') {
        return Err(AppError::BadRequest("invalid storage path".to_string()));
    }
    if raw
        .split('/')
        .any(|seg| seg.is_empty() || seg == "." || seg == "..")
    {
        return Err(AppError::BadRequest("invalid storage path".to_string()));
    }
    ObjectPath::parse(raw).map_err(|_| AppError::BadRequest("invalid storage path".to_string()))
}

/// Best-effort deletion of an image's original + thumbnail objects after a
/// permanent DB delete. NotFound is ignored; other failures only warn — the DB
/// row is already gone and an orphan object is harmless.
pub async fn delete_image_objects(
    store: &dyn ObjectStore,
    storage_path: &str,
    thumbnail_path: &str,
) {
    for rel in [storage_path, thumbnail_path] {
        let Ok(key) = ObjectPath::parse(rel) else {
            tracing::warn!(path = rel, "unparseable object key on delete");
            continue;
        };
        match store.delete(&key).await {
            Ok(()) | Err(object_store::Error::NotFound { .. }) => {}
            Err(e) => tracing::warn!(path = rel, error = %e, "failed to delete object"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_accepts_storage_layout_paths() {
        for ok in [
            "ab/cd/abcdef.png",
            "thumbnails/ab/cd/abcdef.webp",
            "ab/cd/0195a2-uuid_thumb.webp",
        ] {
            assert!(parse_key(ok).is_ok(), "{ok} should be accepted");
        }
    }

    #[test]
    fn parse_key_rejects_traversal_and_junk() {
        for bad in [
            "",
            "/etc/passwd",
            "../secret",
            "ab/../cd/x.png",
            "ab//cd/x.png",
            "ab/./x.png",
            "ab/cd/x\0.png",
        ] {
            assert!(parse_key(bad).is_err(), "{bad:?} should be rejected");
        }
    }
}
