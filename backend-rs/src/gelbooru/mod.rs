//! Gelbooru tag-search client (mirror services/gelbooru_service.py).
//!
//! Proxies tag lookups to the public Gelbooru API. Errors are classified so the
//! HTTP layer can map them to 429 / 503 / 502 like the Python service.
//! NOTE: the Python service caches results (5 min TTL); omitted here.

use std::time::Duration;

use serde_json::Value;

use crate::dto::gelbooru::GelbooruTag;
use crate::util::http_client;

const GELBOORU_API_BASE: &str = "https://gelbooru.com/index.php";

#[derive(Debug)]
pub enum GelbooruError {
    /// HTTP 429.
    RateLimit,
    /// Non-2xx (other than 429) or an unparseable response body.
    Upstream,
    /// Network-level failure (timeout, connection error).
    Unavailable,
}

fn as_i64_loose(v: Option<&Value>) -> i64 {
    match v {
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0),
        Some(Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn as_bool_loose(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => matches!(s.as_str(), "true" | "1"),
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    }
}

/// Search Gelbooru tags by partial (substring) match, ordered by usage count.
pub async fn search_tags(
    api_key: &str,
    user_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<GelbooruTag>, GelbooruError> {
    let name_pattern = format!("%{query}%");
    let limit_s = limit.to_string();
    let resp = http_client()
        .get(GELBOORU_API_BASE)
        .timeout(Duration::from_secs(10))
        .query(&[
            ("page", "dapi"),
            ("s", "tag"),
            ("q", "index"),
            ("name_pattern", name_pattern.as_str()),
            ("json", "1"),
            ("orderby", "count"),
            ("order", "DESC"),
            ("limit", limit_s.as_str()),
            ("api_key", api_key),
            ("user_id", user_id),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Gelbooru request failed");
            GelbooruError::Unavailable
        })?;

    let status = resp.status();
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(GelbooruError::RateLimit);
    }
    if !status.is_success() {
        tracing::warn!(status = %status, "Gelbooru returned non-success");
        return Err(GelbooruError::Upstream);
    }

    let data: Value = resp.json().await.map_err(|e| {
        tracing::warn!(error = %e, "Gelbooru response parse error");
        GelbooruError::Upstream
    })?;

    // The API may return a bare array, {"tag": [...]}, {"tag": {...}}, or empty.
    let raw: Vec<Value> = if let Some(arr) = data.as_array() {
        arr.clone()
    } else if let Some(tag) = data.get("tag") {
        match tag {
            Value::Array(a) => a.clone(),
            obj @ Value::Object(_) => vec![obj.clone()],
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let tags = raw
        .iter()
        .map(|t| GelbooruTag {
            id: as_i64_loose(t.get("id")),
            name: t
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            count: as_i64_loose(t.get("count")),
            r#type: as_i64_loose(t.get("type")),
            ambiguous: as_bool_loose(t.get("ambiguous")),
        })
        .collect();
    Ok(tags)
}
