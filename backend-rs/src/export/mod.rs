//! Export formatting (mirror services/export_service.py): image metadata as
//! JSON or CSV, and prompts as plain text. Pure formatting over rows fetched by
//! `image::list_for_export`.

use std::fmt::Display;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::Value;

use crate::dto::export::ExportRow;
use crate::image::model::ImageRow;

/// CSV header / column order (matches the ExportRow field order).
const CSV_HEADER: &str = "id,original_filename,source_tool,model_type,model_name,\
positive_prompt,negative_prompt,sampler_name,scheduler,steps,cfg_scale,seed,\
width,height,rating,is_favorite,user_tags,user_memo,created_at";

/// Map a DB row to its export representation.
pub fn to_export_row(row: ImageRow) -> ExportRow {
    let user_tags: Vec<String> = serde_json::from_value(row.user_tags).unwrap_or_default();
    ExportRow {
        id: row.id.to_string(),
        original_filename: row.original_filename,
        source_tool: row.source_tool,
        model_type: row.model_type,
        model_name: row.model_name,
        positive_prompt: row.positive_prompt,
        negative_prompt: row.negative_prompt,
        sampler_name: row.sampler_name,
        scheduler: row.scheduler,
        steps: row.steps,
        // Python: `float(cfg) if cfg else None` — Decimal(0) is treated as unset.
        cfg_scale: row
            .cfg_scale
            .filter(|d| *d != Decimal::ZERO)
            .and_then(|d| d.to_f64()),
        seed: row.seed,
        width: row.width,
        height: row.height,
        rating: row.rating as i32,
        is_favorite: row.is_favorite,
        user_tags: user_tags.join(","),
        user_memo: row.user_memo,
        created_at: row.created_at.to_rfc3339(),
    }
}

/// Pretty-printed JSON array (2-space indent), matching json.dumps(indent=2).
pub fn to_json(rows: &[ExportRow]) -> String {
    serde_json::to_string_pretty(rows).unwrap_or_else(|_| "[]".to_string())
}

/// CSV with a header row and CRLF terminators (Python csv.writer defaults).
/// An empty input yields an empty string (Python writes nothing when no data).
pub fn to_csv(rows: &[ExportRow]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(CSV_HEADER);
    out.push_str("\r\n");
    for r in rows {
        let fields = [
            r.id.clone(),
            r.original_filename.clone(),
            r.source_tool.clone(),
            opt_cell(&r.model_type),
            opt_cell(&r.model_name),
            opt_cell(&r.positive_prompt),
            opt_cell(&r.negative_prompt),
            opt_cell(&r.sampler_name),
            opt_cell(&r.scheduler),
            opt_cell(&r.steps),
            opt_cell(&r.cfg_scale),
            opt_cell(&r.seed),
            r.width.to_string(),
            r.height.to_string(),
            r.rating.to_string(),
            // Match Python str(bool): capitalized.
            if r.is_favorite { "True" } else { "False" }.to_string(),
            r.user_tags.clone(),
            opt_cell(&r.user_memo),
            r.created_at.clone(),
        ];
        let escaped: Vec<String> = fields.iter().map(|f| csv_escape(f)).collect();
        out.push_str(&escaped.join(","));
        out.push_str("\r\n");
    }
    out
}

/// Prompts export as plain text, mirroring get_prompts_export line for line.
pub fn prompts_text(rows: &[ImageRow]) -> String {
    let sep = format!("\n{}\n", "-".repeat(50));
    let mut lines: Vec<String> = Vec::new();
    for img in rows {
        lines.push(format!("=== {} ===", img.original_filename));
        let model = img
            .model_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("Unknown");
        lines.push(format!("Model: {model}"));

        if let Some(p) = img.positive_prompt.as_deref().filter(|s| !s.is_empty()) {
            lines.push(format!("\nPositive Prompt:\n{p}"));
        }
        if let Some(n) = img.negative_prompt.as_deref().filter(|s| !s.is_empty()) {
            lines.push(format!("\nNegative Prompt:\n{n}"));
        }

        if let Some(arr) = img.loras.as_array()
            && !arr.is_empty()
        {
            let lora_strs: Vec<String> = arr
                .iter()
                .map(|lora| {
                    let name = lora.get("name").and_then(Value::as_str).unwrap_or("Unknown");
                    let weight = lora
                        .get("weight")
                        .map(|w| w.to_string())
                        .unwrap_or_else(|| "1.0".to_string());
                    format!("{name}:{weight}")
                })
                .collect();
            lines.push(format!("\nLoRAs: {}", lora_strs.join(", ")));
        }

        lines.push(format!(
            "\nSettings: Steps={}, CFG={}, Sampler={}, Seed={}",
            opt_none(&img.steps),
            opt_none(&img.cfg_scale),
            opt_none(&img.sampler_name),
            opt_none(&img.seed),
        ));
        lines.push(sep.clone());
    }
    lines.join("\n")
}

/// CSV cell for an optional value: the value's string form, or "" when absent.
fn opt_cell<T: Display>(o: &Option<T>) -> String {
    o.as_ref().map(ToString::to_string).unwrap_or_default()
}

/// Render an optional like Python f-strings do: the value, or the literal "None".
fn opt_none<T: Display>(o: &Option<T>) -> String {
    o.as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "None".to_string())
}

/// Quote a CSV field if it contains a delimiter, quote, or newline (RFC 4180).
fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
