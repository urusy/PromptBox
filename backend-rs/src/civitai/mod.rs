//! CivitAI client (mirror services/civitai_service.py).
//!
//! Looks up model/LoRA metadata by SHA256 hash or by name (with normalization
//! and fuzzy scoring). Every failure path resolves to `None` — like the Python
//! service, the endpoints always answer 200 with `found: false` on miss/error.
//! NOTE: the Python service caches results (24h TTL); omitted here.

use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;

use regex::Regex;
use serde_json::Value;

use crate::dto::civitai::{
    CivitaiImage, CivitaiModelInfo, CivitaiRecommendedSettings, CivitaiVersionInfo,
};
use crate::util::http_client;

const CIVITAI_API_BASE: &str = "https://civitai.com/api/v1";
const TIMEOUT: Duration = Duration::from_secs(30);

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

/// Normalize a model name for search matching: drop version suffix, split
/// camelCase, collapse separators, lowercase. Mirrors normalize_model_name.
pub fn normalize_model_name(name: &str) -> String {
    static VER: OnceLock<Regex> = OnceLock::new();
    static CAMEL: OnceLock<Regex> = OnceLock::new();
    static SEP: OnceLock<Regex> = OnceLock::new();
    let cleaned = re(&VER, r"(?i)[_-]?v\d+(\.\d+)?[a-zA-Z]*$").replace(name, "");
    let cleaned = re(&CAMEL, r"([a-z])([A-Z])").replace_all(&cleaned, "${1} ${2}");
    let cleaned = re(&SEP, r"[_-]+").replace_all(&cleaned, " ");
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Extract significant words (len > 2, not version-like) for word matching.
fn extract_words(text: &str) -> HashSet<String> {
    static CAMEL: OnceLock<Regex> = OnceLock::new();
    static LD: OnceLock<Regex> = OnceLock::new();
    static DL: OnceLock<Regex> = OnceLock::new();
    static SPLIT: OnceLock<Regex> = OnceLock::new();
    static VERWORD: OnceLock<Regex> = OnceLock::new();
    let text = re(&CAMEL, r"([a-z])([A-Z])").replace_all(text, "${1} ${2}");
    let text = re(&LD, r"(\D)(\d)").replace_all(&text, "${1} ${2}");
    let text = re(&DL, r"(\d)(\D)").replace_all(&text, "${1} ${2}");
    let lower = text.to_lowercase();
    let verword = re(&VERWORD, r"^v?\d+");
    re(&SPLIT, r"[\s_-]+")
        .split(&lower)
        .filter(|w| w.chars().count() > 2 && !verword.is_match(w))
        .map(str::to_string)
        .collect()
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn parse_version_data(v: &Value) -> CivitaiVersionInfo {
    let images = v
        .get("images")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .take(10)
                .map(|img| CivitaiImage {
                    url: str_field(img, "url").unwrap_or_default(),
                    width: img.get("width").and_then(Value::as_i64),
                    height: img.get("height").and_then(Value::as_i64),
                    nsfw: img.get("nsfw").and_then(Value::as_bool).unwrap_or(false)
                        || img.get("nsfwLevel").and_then(Value::as_i64).unwrap_or(0) > 1,
                })
                .collect()
        })
        .unwrap_or_default();

    let files = v.get("files").and_then(Value::as_array);
    let first_file = files.and_then(|f| f.first());
    let file_size_kb = first_file.and_then(|f| f.get("sizeKB")).and_then(Value::as_f64);
    let download_url = first_file.and_then(|f| str_field(f, "downloadUrl"));
    let recommended_settings = first_file
        .and_then(|f| f.get("metadata"))
        .filter(|m| m.is_object() && !m.as_object().unwrap().is_empty())
        .map(|m| CivitaiRecommendedSettings {
            clip_skip: m.get("clipSkip").and_then(Value::as_i64),
            steps: m.get("steps").and_then(Value::as_i64),
            cfg_scale: m.get("cfgScale").and_then(Value::as_f64),
            sampler: str_field(m, "sampler"),
            vae: str_field(m, "vae"),
            strength: m.get("strength").and_then(Value::as_f64),
        });

    let trigger_words = v
        .get("trainedWords")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();

    CivitaiVersionInfo {
        version_id: v.get("id").and_then(Value::as_i64).unwrap_or(0),
        name: str_field(v, "name").unwrap_or_default(),
        description: str_field(v, "description"),
        base_model: str_field(v, "baseModel"),
        images,
        recommended_settings,
        trigger_words,
        download_url,
        file_size_kb,
        published_at: str_field(v, "publishedAt"),
    }
}

fn parse_model_response(data: &Value, is_exact_match: bool) -> CivitaiModelInfo {
    let versions = data
        .get("modelVersions")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(parse_version_data).collect())
        .unwrap_or_default();
    let id = data.get("id").and_then(Value::as_i64).unwrap_or(0);
    CivitaiModelInfo {
        civitai_id: id,
        name: str_field(data, "name").unwrap_or_default(),
        description: str_field(data, "description"),
        r#type: str_field(data, "type").unwrap_or_default(),
        nsfw: data.get("nsfw").and_then(Value::as_bool).unwrap_or(false),
        creator: data
            .get("creator")
            .and_then(|c| str_field(c, "username")),
        civitai_url: Some(format!("https://civitai.com/models/{id}")),
        is_exact_match,
        versions,
    }
}

/// Parse the model-versions/by-hash response (a single version with a nested
/// `model` object).
fn parse_version_response(data: &Value, is_exact_match: bool) -> CivitaiModelInfo {
    let model = data.get("model");
    let model_id = model
        .and_then(|m| m.get("id"))
        .and_then(Value::as_i64)
        .or_else(|| data.get("modelId").and_then(Value::as_i64))
        .unwrap_or(0);
    let version = parse_version_data(data);
    CivitaiModelInfo {
        civitai_id: model_id,
        name: model
            .and_then(|m| str_field(m, "name"))
            .or_else(|| str_field(data, "name"))
            .unwrap_or_default(),
        description: str_field(data, "description")
            .or_else(|| model.and_then(|m| str_field(m, "description"))),
        r#type: model.and_then(|m| str_field(m, "type")).unwrap_or_default(),
        nsfw: model
            .and_then(|m| m.get("nsfw"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        creator: None,
        civitai_url: Some(format!("https://civitai.com/models/{model_id}")),
        is_exact_match,
        versions: vec![version],
    }
}

/// Look up a model by SHA256 hash — the most accurate match.
pub async fn get_model_by_hash(hash_value: &str) -> Option<CivitaiModelInfo> {
    let url = format!("{CIVITAI_API_BASE}/model-versions/by-hash/{hash_value}");
    let resp = http_client().get(&url).timeout(TIMEOUT).send().await.ok()?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return None;
    }
    if !resp.status().is_success() {
        tracing::warn!(status = %resp.status(), hash = hash_value, "CivitAI by-hash error");
        return None;
    }
    let data: Value = resp.json().await.ok()?;
    Some(parse_version_response(&data, true))
}

/// Look up a model by name: exact search first, then fuzzy (marked non-exact).
pub async fn get_model_info(name: &str, model_type: &str) -> Option<CivitaiModelInfo> {
    if let Some(info) = search_models(name, model_type, true).await {
        return Some(info);
    }
    if let Some(mut info) = search_models(name, model_type, false).await {
        info.is_exact_match = false;
        return Some(info);
    }
    None
}

/// Search CivitAI and pick the best fuzzy match above the mode's threshold.
async fn search_models(query: &str, model_type: &str, exact: bool) -> Option<CivitaiModelInfo> {
    let normalized_query = normalize_model_name(query);
    let resp = http_client()
        .get(format!("{CIVITAI_API_BASE}/models"))
        .timeout(TIMEOUT)
        .query(&[
            ("query", normalized_query.as_str()),
            ("types", model_type),
            ("limit", "10"),
            ("nsfw", "true"),
        ])
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        tracing::warn!(status = %resp.status(), query, "CivitAI search error");
        return None;
    }
    let data: Value = resp.json().await.ok()?;
    let items = data.get("items").and_then(Value::as_array)?;
    if items.is_empty() {
        return None;
    }

    let query_words = extract_words(query);
    let query_normalized = normalize_model_name(query);

    let mut best: Option<&Value> = None;
    let mut best_score = 0i32;
    for item in items {
        let item_name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let item_normalized = normalize_model_name(item_name);
        let item_words = extract_words(item_name);

        let mut score = 0i32;
        if item_normalized == query_normalized {
            score = 100;
        } else if item_normalized.contains(&query_normalized) {
            score = 90;
        } else if query_normalized.contains(&item_normalized) {
            score = 85;
        } else {
            let common: HashSet<&String> = query_words.intersection(&item_words).collect();
            if !common.is_empty() {
                let match_ratio = common.len() as f64 / query_words.len().max(1) as f64;
                let item_match_ratio = common.len() as f64 / item_words.len().max(1) as f64;
                score = (match_ratio * 50.0 + item_match_ratio * 30.0) as i32;
                for word in &common {
                    if word.chars().count() >= 5 {
                        score += 10;
                    }
                }
            } else {
                let item_concat = item_normalized.replace(' ', "");
                for qword in &query_words {
                    if qword.chars().count() >= 5 && item_concat.contains(qword) {
                        score = score.max(60);
                        break;
                    }
                }
                let query_concat = query_normalized.replace(' ', "");
                for iword in &item_words {
                    if iword.chars().count() >= 5 && query_concat.contains(iword) {
                        score = score.max(55);
                        break;
                    }
                }
            }
        }

        if score > best_score {
            best_score = score;
            best = Some(item);
        }
    }

    let min_score = if exact { 80 } else { 30 };
    let best = best?;
    if best_score >= min_score {
        Some(parse_model_response(best, best_score >= 85))
    } else {
        None
    }
}
