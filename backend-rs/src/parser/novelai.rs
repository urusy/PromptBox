//! NovelAI parser (mirror parsers/novelai.py). Metadata lives in the PNG
//! `Comment` text chunk as a JSON object.

use std::collections::HashMap;

use serde_json::{json, Value};

use super::{ModelType, ParsedMetadata, SourceTool};

pub fn can_parse(png_info: &HashMap<String, String>) -> bool {
    let Some(comment) = png_info.get("Comment") else {
        return false;
    };
    match serde_json::from_str::<Value>(comment) {
        Ok(data) => data.is_object() && (data.get("uc").is_some() || data.get("prompt").is_some()),
        Err(_) => false,
    }
}

pub fn parse(png_info: &HashMap<String, String>) -> ParsedMetadata {
    let comment = png_info.get("Comment").map(String::as_str).unwrap_or("{}");
    let data: Value = serde_json::from_str(comment).unwrap_or_else(|_| json!({}));

    let mut m = ParsedMetadata::new(SourceTool::Novelai);
    m.model_type = Some(ModelType::Other);
    m.raw_metadata = Some(data.clone());

    if let Some(p) = data.get("prompt") {
        m.positive_prompt = Some(value_to_string(p));
    }
    if let Some(uc) = data.get("uc") {
        m.negative_prompt = Some(value_to_string(uc));
    }
    if let Some(v) = data.get("steps").and_then(as_loose_i64) {
        m.steps = Some(v as i32);
    }
    if let Some(v) = data.get("scale").and_then(as_loose_f64) {
        m.cfg_scale = Some(v);
    }
    if let Some(v) = data.get("seed").and_then(as_loose_i64) {
        m.seed = Some(v);
    }
    if let Some(sampler) = data.get("sampler") {
        // NovelAI prefixes sampler names with "k_".
        let s = value_to_string(sampler);
        m.sampler_name = Some(s.strip_prefix("k_").unwrap_or(&s).to_string());
    }

    for key in ["width", "height", "n_samples", "ucPreset", "qualityToggle"] {
        if let Some(v) = data.get(key) {
            m.model_params.insert(key.to_string(), v.clone());
        }
    }

    m
}

/// Mirror Python `str(value)` for prompt/sampler fields: strings pass through,
/// everything else uses its JSON representation.
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn as_loose_i64(v: &Value) -> Option<i64> {
    v.as_i64()
        .or_else(|| v.as_f64().map(|f| f as i64))
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn as_loose_f64(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}
