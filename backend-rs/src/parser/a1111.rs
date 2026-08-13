//! A1111 / Forge parameters-string parser (mirror parsers/a1111.py).

use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;
use serde_json::json;

use super::{detect_model_type, LoraInfo, ParsedMetadata, SourceTool};

/// `<lora:name:weight>` or `<lora:name:weight:clip_weight>`.
fn lora_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<lora:([^:>]+):([0-9.]+)(?::([0-9.]+))?>").unwrap())
}

pub fn can_parse(png_info: &HashMap<String, String>) -> bool {
    match png_info.get("parameters") {
        Some(p) => p.contains("Steps:") && p.contains("Sampler:"),
        None => false,
    }
}

pub fn parse(png_info: &HashMap<String, String>) -> ParsedMetadata {
    let params_str = png_info.get("parameters").cloned().unwrap_or_default();
    let source_tool = if params_str.contains("Forge") {
        SourceTool::Forge
    } else {
        SourceTool::A1111
    };
    let mut m = ParsedMetadata::new(source_tool);
    m.raw_metadata = Some(json!({ "parameters": params_str }));

    parse_parameters(&params_str, &mut m);
    m.model_type = Some(detect_model_type(m.model_name.as_deref()));
    m
}

fn parse_parameters(params_str: &str, m: &mut ParsedMetadata) {
    let lines: Vec<&str> = params_str.trim().split('\n').collect();

    // The "Steps:" line separates the prompt block from the parameters block.
    let Some(idx) = lines.iter().position(|l| l.starts_with("Steps:")) else {
        return;
    };

    parse_prompts(&lines[..idx], m);
    parse_params_line(lines[idx], m);

    if let Some(prompt) = m.positive_prompt.clone() {
        extract_loras(&prompt, m);
    }
}

fn parse_prompts(lines: &[&str], m: &mut ParsedMetadata) {
    let mut positive_parts: Vec<String> = Vec::new();
    let mut negative_parts: Vec<String> = Vec::new();
    let mut in_negative = false;

    for &line in lines {
        if let Some(rest) = line.strip_prefix("Negative prompt:") {
            in_negative = true;
            let neg = rest.trim();
            if !neg.is_empty() {
                negative_parts.push(neg.to_string());
            }
        } else if in_negative {
            negative_parts.push(line.to_string());
        } else {
            positive_parts.push(line.to_string());
        }
    }

    let positive = positive_parts.join(" ").trim().to_string();
    let negative = negative_parts.join(" ").trim().to_string();
    if !positive.is_empty() {
        m.positive_prompt = Some(positive);
    }
    if !negative.is_empty() {
        m.negative_prompt = Some(negative);
    }
}

fn parse_params_line(params_line: &str, m: &mut ParsedMetadata) {
    for part in split_params(params_line) {
        let Some((key, value)) = part.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        let mp = &mut m.model_params;
        match key {
            "Steps" => {
                if let Ok(v) = value.parse::<i32>() {
                    m.steps = Some(v);
                }
            }
            "Sampler" => m.sampler_name = Some(value.to_string()),
            "CFG scale" => {
                if let Ok(v) = value.parse::<f64>() {
                    m.cfg_scale = Some(v);
                }
            }
            "Seed" => {
                if let Ok(v) = value.parse::<i64>() {
                    m.seed = Some(v);
                }
            }
            "Model" => m.model_name = Some(value.to_string()),
            "Scheduler" => m.scheduler = Some(value.to_string()),
            "Clip skip" => {
                if let Ok(v) = value.parse::<i64>() {
                    mp.insert("clip_skip".into(), json!(v));
                }
            }
            "VAE" => {
                mp.insert("vae".into(), json!(value));
            }
            "Model hash" => {
                mp.insert("model_hash".into(), json!(value));
            }
            "Size" => {
                mp.insert("size".into(), json!(value));
            }
            "Hires upscale" => {
                if let Ok(v) = value.parse::<f64>() {
                    mp.insert("hires_upscale".into(), json!(v));
                }
            }
            "Hires upscaler" => {
                mp.insert("hires_upscaler".into(), json!(value));
            }
            "Hires steps" => {
                if let Ok(v) = value.parse::<i64>() {
                    mp.insert("hires_steps".into(), json!(v));
                }
            }
            "Denoising strength" => {
                if let Ok(v) = value.parse::<f64>() {
                    mp.insert("denoising_strength".into(), json!(v));
                }
            }
            "Script" if value == "X/Y/Z plot" => {
                mp.insert("is_xyz_grid".into(), json!(true));
            }
            "X Type" => {
                mp.insert("xyz_x_type".into(), json!(value));
            }
            "X Values" => {
                mp.insert("xyz_x_values".into(), json!(value.trim_matches('"')));
            }
            "Y Type" => {
                mp.insert("xyz_y_type".into(), json!(value));
            }
            "Y Values" => {
                mp.insert("xyz_y_values".into(), json!(value.trim_matches('"')));
            }
            "Z Type" => {
                mp.insert("xyz_z_type".into(), json!(value));
            }
            "Z Values" => {
                mp.insert("xyz_z_values".into(), json!(value.trim_matches('"')));
            }
            _ => {}
        }
    }
}

/// Split a parameters line on commas, ignoring commas inside double quotes.
fn split_params(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            ',' if !in_quotes => {
                let t = current.trim();
                if !t.is_empty() {
                    parts.push(t.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let t = current.trim();
    if !t.is_empty() {
        parts.push(t.to_string());
    }
    parts
}

fn extract_loras(prompt: &str, m: &mut ParsedMetadata) {
    for caps in lora_re().captures_iter(prompt) {
        let name = caps[1].to_string();
        let weight = caps.get(2).and_then(|w| w.as_str().parse().ok()).unwrap_or(1.0);
        let weight_clip = caps.get(3).and_then(|w| w.as_str().parse().ok());
        m.loras.push(LoraInfo {
            name,
            weight,
            weight_clip,
            hash: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_a1111() {
        let params = "masterpiece, 1girl <lora:detail:0.8>\n\
            Negative prompt: bad hands\n\
            Steps: 28, Sampler: DPM++ 2M, CFG scale: 7.0, Seed: 12345, Model: animagineXL";
        let mut info = HashMap::new();
        info.insert("parameters".to_string(), params.to_string());
        assert!(can_parse(&info));
        let m = parse(&info);
        assert_eq!(m.source_tool, SourceTool::A1111);
        assert_eq!(m.steps, Some(28));
        assert_eq!(m.cfg_scale, Some(7.0));
        assert_eq!(m.seed, Some(12345));
        assert_eq!(m.sampler_name.as_deref(), Some("DPM++ 2M"));
        assert_eq!(m.model_name.as_deref(), Some("animagineXL"));
        assert_eq!(m.negative_prompt.as_deref(), Some("bad hands"));
        assert_eq!(m.loras.len(), 1);
        assert_eq!(m.loras[0].name, "detail");
        assert_eq!(m.loras[0].weight, 0.8);
        assert_eq!(m.model_type.unwrap().as_str(), "sdxl");
    }
}
