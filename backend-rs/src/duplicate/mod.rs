//! Duplicate-file management on disk (mirror endpoints/duplicates.py).
//!
//! The import worker moves files it rejects as duplicates into
//! `<import_path>/duplicated/`. These functions inspect and clean that folder;
//! they touch the filesystem, not the database.

use std::io;
use std::path::{Component, Path, PathBuf};

use crate::dto::duplicate::{DeleteResult, DuplicatesInfo};

fn dup_dir(import_path: &str) -> PathBuf {
    Path::new(import_path).join("duplicated")
}

/// True if `name` is a single, ordinary path segment (no separators, no `..`,
/// not absolute) — the guard against path traversal for per-file deletion.
pub fn is_safe_filename(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut comps = Path::new(name).components();
    matches!(
        (comps.next(), comps.next()),
        (Some(Component::Normal(_)), None)
    )
}

/// List duplicate files (regular, non-hidden), sorted by name, with total size.
/// A missing directory yields an empty result.
pub async fn info(import_path: &str) -> io::Result<DuplicatesInfo> {
    let dir = dup_dir(import_path);
    let mut rd = match tokio::fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(DuplicatesInfo {
                count: 0,
                total_size_bytes: 0,
                files: Vec::new(),
            });
        }
        Err(e) => return Err(e),
    };

    let mut files = Vec::new();
    let mut total: i64 = 0;
    while let Some(entry) = rd.next_entry().await? {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let md = entry.metadata().await?;
        if md.is_file() {
            total += md.len() as i64;
            files.push(name);
        }
    }
    files.sort();
    Ok(DuplicatesInfo {
        count: files.len() as i64,
        total_size_bytes: total,
        files,
    })
}

/// Delete every duplicate file, skipping (and logging) individual failures.
/// A missing directory yields a zero result.
pub async fn delete_all(import_path: &str) -> io::Result<DeleteResult> {
    let dir = dup_dir(import_path);
    let mut rd = match tokio::fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(DeleteResult {
                deleted_count: 0,
                freed_bytes: 0,
            });
        }
        Err(e) => return Err(e),
    };

    let mut deleted_count: i64 = 0;
    let mut freed_bytes: i64 = 0;
    while let Some(entry) = rd.next_entry().await? {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let Ok(md) = entry.metadata().await else {
            continue;
        };
        if !md.is_file() {
            continue;
        }
        let size = md.len() as i64;
        match tokio::fs::remove_file(entry.path()).await {
            Ok(()) => {
                freed_bytes += size;
                deleted_count += 1;
            }
            Err(e) => tracing::warn!(file = %name, error = %e, "failed to delete duplicate"),
        }
    }
    Ok(DeleteResult {
        deleted_count,
        freed_bytes,
    })
}

/// Delete one duplicate file by name. Returns the freed byte count, or `None`
/// if the file does not exist. The caller must validate `filename` first.
pub async fn delete_one(import_path: &str, filename: &str) -> io::Result<Option<i64>> {
    let path = dup_dir(import_path).join(filename);
    let md = match tokio::fs::metadata(&path).await {
        Ok(md) => md,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let size = md.len() as i64;
    tokio::fs::remove_file(&path).await?;
    Ok(Some(size))
}
