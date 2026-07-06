//! ComfyUI workflow-graph parser (mirror parsers/comfyui.py).
//!
//! The PNG `prompt` chunk is a JSON object mapping node-id → {class_type,
//! inputs}. Values reference other nodes as `[node_id, output_index]`. We walk
//! the graph to pull out sampler settings, checkpoint, prompts, loras,
//! controlnets and upscale info. Node iteration order follows insertion order
//! (serde_json `preserve_order`), matching the Python dict behaviour.

use std::collections::HashMap;

use serde_json::{Map, Value};

use super::{detect_model_type, ControlNetInfo, LoraInfo, ParsedMetadata, SourceTool};

const SAMPLER_NODES: &[&str] = &["KSampler", "KSamplerAdvanced", "SamplerCustom"];
const CHECKPOINT_NODES: &[&str] = &["CheckpointLoaderSimple", "CheckpointLoader", "UNETLoader"];
const PROMPT_NODES: &[&str] = &["CLIPTextEncode", "CLIPTextEncodeSDXL"];
const LORA_NODES: &[&str] = &["LoraLoader", "LoraLoaderModelOnly"];
const UPSCALE_MODEL_NODES: &[&str] = &["UpscaleModelLoader"];

/// Guard against cyclic graphs when following prompt references.
const MAX_REF_DEPTH: u32 = 30;

pub fn can_parse(png_info: &HashMap<String, String>) -> bool {
    let Some(prompt) = png_info.get("prompt") else {
        return false;
    };
    let Ok(data) = serde_json::from_str::<Value>(prompt) else {
        return false;
    };
    let Some(obj) = data.as_object() else {
        return false;
    };
    obj.values()
        .any(|n| n.is_object() && n.get("class_type").is_some())
}

pub fn parse(png_info: &HashMap<String, String>) -> ParsedMetadata {
    let prompt_data: Value = png_info
        .get("prompt")
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| Value::Object(Map::new()));
    let workflow_data: Value = png_info
        .get("workflow")
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| Value::Object(Map::new()));

    let mut m = ParsedMetadata::new(SourceTool::Comfyui);
    m.raw_metadata = Some(serde_json::json!({
        "prompt": prompt_data.clone(),
        "workflow": workflow_data.clone(),
    }));

    let empty = Map::new();
    let prompt = prompt_data.as_object().unwrap_or(&empty);

    extract_sampler_data(prompt, &mut m);
    extract_checkpoint_data(prompt, &mut m);
    extract_prompts(prompt, &mut m);
    extract_loras(prompt, &mut m);
    extract_controlnets(prompt, &mut m);
    extract_upscale_info(prompt, &mut m);
    extract_workflow_extras(&workflow_data, &mut m);

    m.model_type = Some(detect_model_type(m.model_name.as_deref()));
    m
}

fn class_type(node: &Value) -> &str {
    node.get("class_type").and_then(Value::as_str).unwrap_or("")
}

fn inputs(node: &Value) -> Option<&Map<String, Value>> {
    node.get("inputs").and_then(Value::as_object)
}

/// JSON number → i64 (accepts integral floats), excluding bools.
fn num_i64(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))
}

fn num_f64(v: &Value) -> Option<f64> {
    if v.is_boolean() {
        return None;
    }
    v.as_f64()
}

/// Strip a single trailing file extension (mirror `name.rsplit(".", 1)[0]`).
fn strip_ext(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, _)) => stem.to_string(),
        None => name.to_string(),
    }
}

fn extract_sampler_data(prompt: &Map<String, Value>, m: &mut ParsedMetadata) {
    for node in prompt.values() {
        if !node.is_object() || !SAMPLER_NODES.contains(&class_type(node)) {
            continue;
        }
        if let Some(inp) = inputs(node) {
            if let Some(v) = inp.get("seed").and_then(num_i64) {
                m.seed = Some(v);
            }
            if let Some(v) = inp.get("steps").and_then(num_i64) {
                m.steps = Some(v as i32);
            }
            if let Some(v) = inp.get("cfg").and_then(num_f64) {
                m.cfg_scale = Some(v);
            }
            if let Some(v) = inp.get("sampler_name").and_then(Value::as_str) {
                m.sampler_name = Some(v.to_string());
            }
            if let Some(v) = inp.get("scheduler").and_then(Value::as_str) {
                m.scheduler = Some(v.to_string());
            }
        }
        break;
    }
}

fn extract_checkpoint_data(prompt: &Map<String, Value>, m: &mut ParsedMetadata) {
    for node in prompt.values() {
        if !node.is_object() || !CHECKPOINT_NODES.contains(&class_type(node)) {
            continue;
        }
        if let Some(inp) = inputs(node) {
            let ckpt = inp
                .get("ckpt_name")
                .and_then(Value::as_str)
                .or_else(|| inp.get("unet_name").and_then(Value::as_str));
            if let Some(name) = ckpt {
                m.model_name = Some(strip_ext(name));
                break;
            }
        }
    }
}

fn extract_prompts(prompt: &Map<String, Value>, m: &mut ParsedMetadata) {
    let Some(sampler) = prompt
        .values()
        .find(|n| n.is_object() && SAMPLER_NODES.contains(&class_type(n)))
    else {
        return;
    };
    let Some(inp) = inputs(sampler) else {
        return;
    };

    if let Some(pos) = inp.get("positive")
        && let Some(text) = resolve_prompt_reference(prompt, pos, MAX_REF_DEPTH)
    {
        m.positive_prompt = Some(text);
    }
    if let Some(neg) = inp.get("negative")
        && let Some(text) = resolve_prompt_reference(prompt, neg, MAX_REF_DEPTH)
    {
        m.negative_prompt = Some(text);
    }
}

fn ref_node_id(r: &Value) -> Option<String> {
    match r.as_array()?.first()? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn resolve_prompt_reference(prompt: &Map<String, Value>, r: &Value, depth: u32) -> Option<String> {
    if depth == 0 {
        return None;
    }
    let node = prompt.get(&ref_node_id(r)?)?;
    if !node.is_object() {
        return None;
    }
    let inp = inputs(node);

    if PROMPT_NODES.contains(&class_type(node))
        && let Some(inp) = inp
    {
        if let Some(t) = inp.get("text").and_then(Value::as_str) {
            return Some(t.to_string());
        }
        if let Some(tg) = inp.get("text_g").and_then(Value::as_str) {
            return Some(tg.to_string());
        }
    }

    // Follow conditioning/clip links toward the text-encode node.
    if let Some(inp) = inp {
        for key in ["conditioning", "clip"] {
            if let Some(next) = inp.get(key)
                && next.is_array()
                && let Some(result) = resolve_prompt_reference(prompt, next, depth - 1)
            {
                return Some(result);
            }
        }
    }
    None
}

fn extract_loras(prompt: &Map<String, Value>, m: &mut ParsedMetadata) {
    for node in prompt.values() {
        if !node.is_object() || !LORA_NODES.contains(&class_type(node)) {
            continue;
        }
        let Some(inp) = inputs(node) else { continue };
        let Some(name) = inp.get("lora_name").and_then(Value::as_str) else {
            continue;
        };
        let weight = inp.get("strength_model").and_then(num_f64).unwrap_or(1.0);
        let weight_clip = inp.get("strength_clip").and_then(num_f64);
        m.loras.push(LoraInfo {
            name: strip_ext(name),
            weight,
            weight_clip,
            hash: None,
        });
    }
}

fn extract_controlnets(prompt: &Map<String, Value>, m: &mut ParsedMetadata) {
    // Pass 1: ControlNetLoader node-id → model name.
    let mut models: HashMap<String, String> = HashMap::new();
    for (node_id, node) in prompt {
        if node.is_object() && class_type(node) == "ControlNetLoader"
            && let Some(name) = inputs(node)
                .and_then(|i| i.get("control_net_name"))
                .and_then(Value::as_str)
        {
            models.insert(node_id.clone(), name.to_string());
        }
    }

    // Pass 2: Apply nodes carry the strength and reference a loader.
    for node in prompt.values() {
        if !node.is_object()
            || !matches!(class_type(node), "ControlNetApply" | "ControlNetApplyAdvanced")
        {
            continue;
        }
        let Some(inp) = inputs(node) else { continue };
        let strength = inp.get("strength").and_then(num_f64).unwrap_or(1.0);
        let model = inp
            .get("control_net")
            .and_then(ref_node_id)
            .and_then(|id| models.get(&id).cloned())
            .unwrap_or_else(|| "unknown".to_string());
        m.controlnets.push(ControlNetInfo {
            model,
            weight: strength,
            guidance_start: inp.get("start_percent").and_then(num_f64).unwrap_or(0.0),
            guidance_end: inp.get("end_percent").and_then(num_f64).unwrap_or(1.0),
            preprocessor: None,
        });
    }
}

fn extract_upscale_info(prompt: &Map<String, Value>, m: &mut ParsedMetadata) {
    let mut models: HashMap<String, String> = HashMap::new();
    for (node_id, node) in prompt {
        if node.is_object() && UPSCALE_MODEL_NODES.contains(&class_type(node))
            && let Some(name) = inputs(node)
                .and_then(|i| i.get("model_name"))
                .and_then(Value::as_str)
        {
            models.insert(node_id.clone(), name.to_string());
        }
    }

    for node in prompt.values() {
        if !node.is_object() {
            continue;
        }
        let Some(inp) = inputs(node) else { continue };
        let mp = &mut m.model_params;
        match class_type(node) {
            "ImageUpscaleWithModel" => {
                let model = inp
                    .get("upscale_model")
                    .and_then(ref_node_id)
                    .and_then(|id| models.get(&id).cloned())
                    .unwrap_or_else(|| "unknown".to_string());
                mp.insert("hires_upscaler".into(), Value::String(model));
                mp.insert("upscale_method".into(), Value::String("model".into()));
            }
            "LatentUpscale" => {
                let method = inp
                    .get("upscale_method")
                    .and_then(Value::as_str)
                    .unwrap_or("nearest-exact");
                mp.insert("hires_upscaler".into(), Value::String(format!("Latent ({method})")));
                mp.insert("upscale_method".into(), Value::String("latent".into()));
                let width = inp.get("width").and_then(num_i64);
                let height = inp.get("height").and_then(num_i64);
                if let (Some(w), Some(h)) = (width, height) {
                    mp.insert("upscale_size".into(), Value::String(format!("{w}x{h}")));
                }
            }
            "LatentUpscaleBy" => {
                let method = inp
                    .get("upscale_method")
                    .and_then(Value::as_str)
                    .unwrap_or("nearest-exact");
                let scale = inp.get("scale_by").and_then(num_f64).unwrap_or(1.0);
                mp.insert("hires_upscaler".into(), Value::String(format!("Latent ({method})")));
                mp.insert("hires_upscale".into(), serde_json::json!(scale));
                mp.insert("upscale_method".into(), Value::String("latent".into()));
            }
            "ImageScaleBy" => {
                let method = inp
                    .get("upscale_method")
                    .and_then(Value::as_str)
                    .unwrap_or("nearest-exact");
                let scale = inp.get("scale_by").and_then(num_f64).unwrap_or(1.0);
                mp.insert("hires_upscaler".into(), Value::String(format!("Image ({method})")));
                mp.insert("hires_upscale".into(), serde_json::json!(scale));
                mp.insert("upscale_method".into(), Value::String("image".into()));
            }
            _ => {}
        }
    }
}

fn extract_workflow_extras(workflow: &Value, m: &mut ParsedMetadata) {
    let Some(obj) = workflow.as_object() else {
        return;
    };
    if let Some(nodes) = obj.get("nodes").and_then(Value::as_array) {
        m.workflow_extras
            .insert("node_count".into(), serde_json::json!(nodes.len()));
    }
    if let Some(version) = obj.get("version")
        && !version.is_null()
    {
        m.workflow_extras
            .insert("workflow_version".into(), version.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_comfyui_graph() {
        let prompt = r#"{
            "3": {"class_type": "KSampler", "inputs": {
                "seed": 42, "steps": 20, "cfg": 7.5,
                "sampler_name": "euler", "scheduler": "normal",
                "positive": ["6", 0], "negative": ["7", 0]
            }},
            "4": {"class_type": "CheckpointLoaderSimple", "inputs": {"ckpt_name": "animagine.safetensors"}},
            "6": {"class_type": "CLIPTextEncode", "inputs": {"text": "a cat"}},
            "7": {"class_type": "CLIPTextEncode", "inputs": {"text": "bad"}}
        }"#;
        let mut info = HashMap::new();
        info.insert("prompt".to_string(), prompt.to_string());
        assert!(can_parse(&info));
        let m = parse(&info);
        assert_eq!(m.source_tool, SourceTool::Comfyui);
        assert_eq!(m.seed, Some(42));
        assert_eq!(m.steps, Some(20));
        assert_eq!(m.cfg_scale, Some(7.5));
        assert_eq!(m.sampler_name.as_deref(), Some("euler"));
        assert_eq!(m.scheduler.as_deref(), Some("normal"));
        assert_eq!(m.model_name.as_deref(), Some("animagine"));
        assert_eq!(m.positive_prompt.as_deref(), Some("a cat"));
        assert_eq!(m.negative_prompt.as_deref(), Some("bad"));
    }
}
