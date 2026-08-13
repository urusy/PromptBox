//! Shared fixtures for the integration tests.
//!
//! Every test gets its own throwaway database from `#[sqlx::test]`, built by
//! running `migrations/` — so these helpers only need to insert rows.

#![allow(dead_code)] // each test file uses a different subset

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::{DateTime, TimeZone, Utc};
use promptbox::config::Config;
use promptbox::http::AppState;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`
use uuid::Uuid;

/// Build the real router against a throwaway database, using the test config
/// (fs storage under /tmp, watcher disabled).
pub fn test_router(pool: PgPool) -> Router {
    let config = Config::for_test();
    let storage = promptbox::storage::build(&config).expect("fs store");
    let jobs = promptbox::job::Jobs::new(pool.clone());
    promptbox::http::router(AppState {
        config: Arc::new(config),
        pool,
        storage,
        jobs,
    })
}

/// A valid `session` cookie for the test config's secret key.
pub fn session_cookie() -> String {
    let config = Config::for_test();
    let token = promptbox::auth::create_session("admin", &config.secret_key, 1)
        .expect("create session token");
    format!("session={token}")
}

/// Issue a GET and return (status, parsed JSON body). `cookie` is sent as-is
/// when present.
pub async fn get_json(router: Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .uri(uri)
        .method("GET")
        // Stands in for ConnectInfo, which `oneshot` does not provide; the
        // login rate limiter needs a client address to key on.
        .header("x-forwarded-for", "198.51.100.60");
    if let Some(c) = cookie {
        builder = builder.header("cookie", c);
    }
    let response = router
        .oneshot(builder.body(Body::empty()).expect("build request"))
        .await
        .expect("router response");

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read body");
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// A row to insert into `images`. Construct with `..NewImage::default()` and
/// override only what the assertion is about.
pub struct NewImage {
    pub id: Uuid,
    pub source_tool: String,
    pub model_type: Option<String>,
    pub has_metadata: bool,
    pub original_filename: String,
    pub width: i32,
    pub height: i32,
    pub file_size_bytes: i64,
    pub positive_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub model_name: Option<String>,
    pub sampler_name: Option<String>,
    pub scheduler: Option<String>,
    pub steps: Option<i32>,
    /// NUMERIC(5,2) in Postgres; bound as float8 and cast, so the tests do not
    /// need a rust_decimal dependency of their own.
    pub cfg_scale: Option<f64>,
    pub seed: Option<i64>,
    pub loras: Value,
    pub model_params: Value,
    pub rating: i16,
    pub is_favorite: bool,
    pub needs_improvement: bool,
    pub user_tags: Value,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Default for NewImage {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            source_tool: "comfyui".to_string(),
            model_type: Some("sdxl".to_string()),
            has_metadata: true,
            original_filename: "image.png".to_string(),
            width: 1024,
            height: 1024,
            file_size_bytes: 1024,
            positive_prompt: None,
            negative_prompt: None,
            model_name: None,
            sampler_name: None,
            scheduler: None,
            steps: None,
            cfg_scale: None,
            seed: None,
            loras: json!([]),
            model_params: json!({}),
            rating: 0,
            is_favorite: false,
            needs_improvement: false,
            user_tags: json!([]),
            created_at: utc(2026, 1, 1, 0, 0, 0),
            deleted_at: None,
        }
    }
}

/// A fixed UTC instant, for deterministic ordering assertions.
pub fn utc(y: i32, m: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, mi, s).unwrap()
}

/// Insert one image and return its id. `file_hash`, `storage_path` and
/// `thumbnail_path` are derived from the id so the UNIQUE constraint on
/// `file_hash` never collides.
pub async fn insert_image(pool: &PgPool, img: NewImage) -> Uuid {
    let hex = img.id.simple().to_string();
    let file_hash = format!("{hex}{hex}"); // 64 chars, like a sha256
    let storage_path = format!("{}/{}/{}.png", &hex[0..2], &hex[2..4], hex);
    let thumbnail_path = format!("thumbnails/{}/{}/{}.webp", &hex[0..2], &hex[2..4], hex);

    sqlx::query(
        "INSERT INTO images (
            id, source_tool, model_type, has_metadata, original_filename,
            storage_path, thumbnail_path, file_hash, width, height,
            file_size_bytes, positive_prompt, negative_prompt, model_name,
            sampler_name, scheduler, steps, cfg_scale, seed, loras,
            model_params, rating, is_favorite, needs_improvement, user_tags,
            created_at, deleted_at
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
            $16, $17, $18::float8::numeric, $19, $20, $21, $22, $23, $24, $25,
            $26, $27
         )",
    )
    .bind(img.id)
    .bind(&img.source_tool)
    .bind(&img.model_type)
    .bind(img.has_metadata)
    .bind(&img.original_filename)
    .bind(&storage_path)
    .bind(&thumbnail_path)
    .bind(&file_hash)
    .bind(img.width)
    .bind(img.height)
    .bind(img.file_size_bytes)
    .bind(&img.positive_prompt)
    .bind(&img.negative_prompt)
    .bind(&img.model_name)
    .bind(&img.sampler_name)
    .bind(&img.scheduler)
    .bind(img.steps)
    .bind(img.cfg_scale)
    .bind(img.seed)
    .bind(&img.loras)
    .bind(&img.model_params)
    .bind(img.rating)
    .bind(img.is_favorite)
    .bind(img.needs_improvement)
    .bind(&img.user_tags)
    .bind(img.created_at)
    .bind(img.deleted_at)
    .execute(pool)
    .await
    .expect("insert image");

    img.id
}

/// Create a showcase and return its id.
pub async fn insert_showcase(pool: &PgPool, name: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO showcases (id, name) VALUES ($1, $2)")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .expect("insert showcase");
    id
}

/// Add images to a showcase in the given (curated) order.
pub async fn add_to_showcase(pool: &PgPool, showcase_id: Uuid, image_ids: &[Uuid]) {
    for (i, image_id) in image_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO showcase_images (showcase_id, image_id, sort_order) VALUES ($1, $2, $3)",
        )
        .bind(showcase_id)
        .bind(image_id)
        .bind(i as i32)
        .execute(pool)
        .await
        .expect("add to showcase");
    }
}

/// Read back an image's user_tags as a plain Vec<String>.
pub async fn tags_of(pool: &PgPool, id: Uuid) -> Vec<String> {
    let value: Value = sqlx::query_scalar("SELECT user_tags FROM images WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("fetch user_tags");
    serde_json::from_value(value).expect("user_tags is a string array")
}

/// Whether an image is currently soft-deleted.
pub async fn is_deleted(pool: &PgPool, id: Uuid) -> bool {
    let deleted_at: Option<DateTime<Utc>> =
        sqlx::query_scalar("SELECT deleted_at FROM images WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .expect("fetch deleted_at");
    deleted_at.is_some()
}

/// Insert `n` images that differ only in `created_at` (one minute apart,
/// ascending), returning their ids in creation order.
pub async fn insert_sequence(pool: &PgPool, n: u32) -> Vec<Uuid> {
    let mut ids = Vec::new();
    for i in 0..n {
        let id = insert_image(
            pool,
            NewImage {
                original_filename: format!("seq-{i}.png"),
                created_at: utc(2026, 1, 1, 0, i, 0),
                ..Default::default()
            },
        )
        .await;
        ids.push(id);
    }
    ids
}
