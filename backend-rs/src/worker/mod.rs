//! Import worker (mirror workers/watcher.py).
//!
//! Watches the import folder (filesystem events via `notify`) plus a periodic
//! safety-net scan, and imports each new image: hash → duplicate check → metadata
//! parse → upload original + WebP thumbnail to object storage (content-addressed
//! keys) → DB row → remove the import file. Runs as a background Tokio task;
//! CPU/IO-heavy steps use `spawn_blocking`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use object_store::ObjectStore;
use object_store::path::Path as ObjectPath;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sqlx::PgPool;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use crate::config::Config;
use crate::media;
use crate::parser::{self, ParsedMetadata};

const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];
const PERIODIC_SCAN_INTERVAL: Duration = Duration::from_secs(30);
const STABLE_POLL: Duration = Duration::from_millis(500);
const STABLE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct Worker {
    pool: PgPool,
    config: Arc<Config>,
    storage: Arc<dyn ObjectStore>,
    /// Paths currently being imported, to avoid double-processing across the
    /// event stream and the periodic scan.
    processing: Arc<Mutex<HashSet<PathBuf>>>,
    /// Per-file content-failure counter (mirror ImageImportHandler._failures).
    /// Once a file exceeds `import_max_attempts` it is quarantined to
    /// `import/<import_failed_dir>/` so the periodic scan stops retrying it.
    failures: Arc<Mutex<HashMap<PathBuf, u32>>>,
}

/// How a failed import should be treated (mirror the Python watcher's
/// _PERMANENT_ERRORS / _TRANSIENT_ERRORS split).
enum FailureKind {
    /// The file content itself can never import (decode/format error).
    Permanent,
    /// Environmental (DB down, IO hiccup) — retrying later is correct.
    Transient,
    /// Unknown — retry, but give up after `import_max_attempts`.
    Countable,
}

/// Spawn the import worker as a background task (no-op aside from logging if the
/// import directory can't be created).
pub fn spawn(pool: PgPool, config: Arc<Config>, storage: Arc<dyn ObjectStore>) {
    let worker = Worker {
        pool,
        config,
        storage,
        processing: Arc::new(Mutex::new(HashSet::new())),
        failures: Arc::new(Mutex::new(HashMap::new())),
    };
    tokio::spawn(async move {
        worker.run().await;
    });
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

impl Worker {
    async fn run(self) {
        let import_dir = PathBuf::from(&self.config.import_path);
        if let Err(e) = std::fs::create_dir_all(&import_dir) {
            tracing::error!(error = %e, dir = %import_dir.display(), "cannot create import dir; watcher disabled");
            return;
        }
        tracing::info!(dir = %import_dir.display(), "import watcher started");

        // Import anything already sitting in the folder.
        self.scan_and_process(&import_dir).await;

        // Filesystem events (notify invokes its callback on its own thread; we
        // forward paths onto this async task via a channel).
        let (tx, mut rx) = mpsc::unbounded_channel::<PathBuf>();
        let watcher = match setup_notify(&import_dir, tx) {
            Ok(w) => Some(w),
            Err(e) => {
                tracing::warn!(error = %e, "notify watcher unavailable; periodic scan only");
                None
            }
        };

        // Periodic safety-net scan.
        let scanner = self.clone();
        let scan_dir = import_dir.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(PERIODIC_SCAN_INTERVAL);
            interval.tick().await; // consume the immediate first tick
            loop {
                interval.tick().await;
                scanner.scan_and_process(&scan_dir).await;
            }
        });

        // Drain events (holding `watcher` alive for as long as we're listening).
        while let Some(path) = rx.recv().await {
            self.process_path(path).await;
        }
        drop(watcher);
    }

    async fn scan_and_process(&self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut files: Vec<PathBuf> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_supported(&path) {
                files.push(path);
            }
        }
        for path in files {
            self.process_path(path).await;
        }
    }

    /// (C) True if the filename matches an excluded pattern (e.g. "xyz_grid").
    fn should_skip(&self, path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return false;
        };
        let name = name.to_lowercase();
        self.config
            .import_skip_patterns
            .iter()
            .any(|p| name.contains(p))
    }

    /// Move a file out of the import folder into `import/<subdir>/` so it is no
    /// longer re-scanned. Used for excluded files (C) and permanent failures (B).
    async fn quarantine(&self, path: &Path, subdir: &str, reason: &str) {
        self.failures.lock().await.remove(path);
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            return;
        }
        match self.move_aside(path, subdir).await {
            Ok(dest) => {
                tracing::warn!(from = %path.display(), to = %dest.display(), reason, "quarantined file");
            }
            Err(e) => {
                tracing::error!(path = %path.display(), error = ?e, "failed to quarantine file");
            }
        }
    }

    /// (B) Decide whether a failed file should be quarantined.
    async fn register_failure(&self, path: &Path, err: &anyhow::Error) {
        match classify_failure(err) {
            FailureKind::Permanent => {
                self.quarantine(path, &self.config.import_failed_dir, "permanent error")
                    .await;
            }
            FailureKind::Transient => {
                // Environmental problem — don't penalize the file.
            }
            FailureKind::Countable => {
                let count = {
                    let mut map = self.failures.lock().await;
                    let count = map.entry(path.to_path_buf()).or_insert(0);
                    *count += 1;
                    *count
                };
                if count >= self.config.import_max_attempts {
                    self.quarantine(
                        path,
                        &self.config.import_failed_dir,
                        &format!("failed {count}x"),
                    )
                    .await;
                }
            }
        }
    }

    /// Claim a path (dedup), process it, then release the claim.
    async fn process_path(&self, path: PathBuf) {
        if !is_supported(&path) {
            return;
        }
        // (C) Excluded by filename pattern — move it aside, never process.
        if self.should_skip(&path) {
            self.quarantine(&path, &self.config.import_skipped_dir, "excluded by pattern")
                .await;
            return;
        }
        {
            let mut set = self.processing.lock().await;
            if !set.insert(path.clone()) {
                return;
            }
        }
        if let Err(e) = self.process_file(&path).await {
            tracing::error!(path = %path.display(), error = ?e, "failed to import file");
            self.register_failure(&path, &e).await;
        }
        self.processing.lock().await.remove(&path);
    }

    async fn process_file(&self, path: &Path) -> Result<()> {
        self.wait_for_stable(path).await;
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            tracing::warn!(path = %path.display(), "file vanished before import");
            return Ok(());
        }
        self.import_image(path).await
    }

    /// Wait until the file size stops changing (mirror _wait_for_file).
    async fn wait_for_stable(&self, path: &Path) {
        let start = tokio::time::Instant::now();
        let mut last_size: i64 = -1;
        loop {
            let Ok(meta) = tokio::fs::metadata(path).await else {
                break;
            };
            let size = meta.len() as i64;
            if size == last_size && size > 0 {
                tokio::time::sleep(STABLE_POLL).await;
                break;
            }
            last_size = size;
            if start.elapsed() > STABLE_TIMEOUT {
                break;
            }
            tokio::time::sleep(STABLE_POLL).await;
        }
    }

    async fn import_image(&self, path: &Path) -> Result<()> {
        let owned = path.to_path_buf();

        let hash = run_blocking(&owned, media::file_hash).await?;

        // Duplicate: the content is already stored — move the incoming file aside.
        let existing: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM images WHERE file_hash = $1")
                .bind(&hash)
                .fetch_optional(&self.pool)
                .await?;
        if existing.is_some() {
            tracing::info!(path = %path.display(), "duplicate detected");
            self.move_to_duplicated(path).await?;
            return Ok(());
        }

        let (width, height) = run_blocking(&owned, media::image_dimensions).await?;

        let meta = tokio::fs::metadata(path).await?;
        let file_size = meta.len() as i64;
        let created_at = file_created_at(&meta);

        let png_info = {
            let p = owned.clone();
            tokio::task::spawn_blocking(move || media::read_image_info(&p)).await?
        };
        let parsed = parser::parse(&png_info);

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image")
            .to_string();
        let storage_rel = media::storage_path(&hash, &filename);
        let thumb_rel = media::thumbnail_path(&hash);

        // Read the original once and encode the thumbnail in memory. Decoding
        // happens before any upload, so bad content still fails Permanent-ly
        // without leaving partial objects behind.
        let bytes = tokio::fs::read(path).await?;
        let (bytes, thumb) = {
            let size = self.config.thumbnail_size;
            let quality = self.config.thumbnail_quality;
            tokio::task::spawn_blocking(move || {
                let thumb = media::create_thumbnail_bytes(&bytes, size, quality)?;
                Ok::<_, anyhow::Error>((bytes, thumb))
            })
            .await??
        };

        // Upload both objects before the DB row so a listed image always has
        // its files. Keys are content-addressed → retried puts are idempotent.
        let storage_key = ObjectPath::parse(&storage_rel)?;
        let thumb_key = ObjectPath::parse(&thumb_rel)?;
        self.storage.put(&storage_key, bytes.into()).await?;
        self.storage.put(&thumb_key, thumb.into()).await?;

        self.insert_image(
            &parsed,
            &filename,
            &storage_rel,
            &thumb_rel,
            &hash,
            width as i32,
            height as i32,
            file_size,
            created_at,
        )
        .await?;

        // Everything is durable (objects + DB row); the import file is done.
        tokio::fs::remove_file(path).await?;

        tracing::info!(file = %filename, "imported");
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_image(
        &self,
        parsed: &ParsedMetadata,
        filename: &str,
        storage_rel: &str,
        thumb_rel: &str,
        hash: &str,
        width: i32,
        height: i32,
        file_size: i64,
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        let cfg_scale = parsed.cfg_scale.and_then(Decimal::from_f64);
        sqlx::query(
            "INSERT INTO images (\
                id, source_tool, model_type, has_metadata, original_filename, \
                storage_path, thumbnail_path, file_hash, width, height, file_size_bytes, \
                positive_prompt, negative_prompt, model_name, sampler_name, scheduler, \
                steps, cfg_scale, seed, loras, controlnets, embeddings, model_params, \
                workflow_extras, raw_metadata, created_at, updated_at\
             ) VALUES (\
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, \
                $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27\
             )",
        )
        .bind(Uuid::now_v7())
        .bind(parsed.source_tool.as_str())
        .bind(parsed.model_type.map(|t| t.as_str()))
        .bind(parsed.has_metadata)
        .bind(filename)
        .bind(storage_rel)
        .bind(thumb_rel)
        .bind(hash)
        .bind(width)
        .bind(height)
        .bind(file_size)
        .bind(parsed.positive_prompt.as_deref())
        .bind(parsed.negative_prompt.as_deref())
        .bind(parsed.model_name.as_deref())
        .bind(parsed.sampler_name.as_deref())
        .bind(parsed.scheduler.as_deref())
        .bind(parsed.steps)
        .bind(cfg_scale)
        .bind(parsed.seed)
        .bind(parsed.loras_json())
        .bind(parsed.controlnets_json())
        .bind(parsed.embeddings_json())
        .bind(parsed.model_params_json())
        .bind(parsed.workflow_extras_json())
        .bind(parsed.raw_metadata.clone())
        .bind(created_at)
        .bind(created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Move a duplicate out of the way into `<import>/duplicated/`, renaming on
    /// collision (mirror the Python `_import_image` duplicate branch).
    async fn move_to_duplicated(&self, path: &Path) -> Result<()> {
        let dest = self.move_aside(path, "duplicated").await?;
        tracing::info!(to = %dest.display(), "moved duplicate aside");
        Ok(())
    }

    /// Move a file into `<import>/<subdir>/`, renaming on collision. Returns the
    /// destination path.
    async fn move_aside(&self, path: &Path, subdir: &str) -> Result<PathBuf> {
        let dir = PathBuf::from(&self.config.import_path).join(subdir);
        tokio::fs::create_dir_all(&dir).await?;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        let mut dest = dir.join(name);
        if tokio::fs::try_exists(&dest).await.unwrap_or(false) {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
            let ext = path.extension().and_then(|e| e.to_str());
            let mut counter = 1u32;
            loop {
                let candidate = match ext {
                    Some(e) => dir.join(format!("{stem}_{counter}.{e}")),
                    None => dir.join(format!("{stem}_{counter}")),
                };
                if !tokio::fs::try_exists(&candidate).await.unwrap_or(false) {
                    dest = candidate;
                    break;
                }
                counter += 1;
            }
        }
        move_file(path, &dest).await?;
        Ok(dest)
    }
}

/// Classify an import error (mirror the Python watcher's error split).
///
/// - Decode/format errors from the `image` crate mean the file content itself is
///   bad or unsupported → Permanent (except its IO variant, which is transient).
/// - `sqlx::Error` / `std::io::Error` / `object_store::Error` are environmental
///   (DB down, IO hiccup, MinIO unreachable) → Transient, so a stopped MinIO
///   never quarantines good files to `failed/`.
/// - Anything else is retried up to `import_max_attempts` → Countable.
fn classify_failure(err: &anyhow::Error) -> FailureKind {
    for cause in err.chain() {
        if let Some(img_err) = cause.downcast_ref::<image::ImageError>() {
            return match img_err {
                image::ImageError::IoError(_) => FailureKind::Transient,
                _ => FailureKind::Permanent,
            };
        }
        if cause.downcast_ref::<sqlx::Error>().is_some()
            || cause.downcast_ref::<std::io::Error>().is_some()
            || cause.downcast_ref::<object_store::Error>().is_some()
        {
            return FailureKind::Transient;
        }
    }
    FailureKind::Countable
}

/// Run a blocking media operation on a cloned path off the async runtime.
async fn run_blocking<T, F>(path: &Path, f: F) -> Result<T>
where
    F: FnOnce(&Path) -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    let owned = path.to_path_buf();
    tokio::task::spawn_blocking(move || f(&owned)).await?
}

fn file_created_at(meta: &std::fs::Metadata) -> DateTime<Utc> {
    let ts = meta
        .created()
        .or_else(|_| meta.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    DateTime::<Utc>::from(ts)
}

/// Move a file, falling back to copy+remove across filesystem boundaries
/// (mirror shutil.move).
async fn move_file(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if tokio::fs::rename(src, dst).await.is_ok() {
        return Ok(());
    }
    tokio::fs::copy(src, dst).await?;
    tokio::fs::remove_file(src).await?;
    Ok(())
}

fn setup_notify(
    dir: &Path,
    tx: mpsc::UnboundedSender<PathBuf>,
) -> notify::Result<notify::RecommendedWatcher> {
    use notify::{EventKind, RecursiveMode, Watcher};
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res
            && matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
        {
            for path in event.paths {
                let _ = tx.send(path);
            }
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
