//! Metadata parsers (mirror app/parsers/). Given a PNG/JPEG text-chunk map,
//! select the matching parser (ComfyUI → A1111/Forge → NovelAI) and extract a
//! normalized `ParsedMetadata`. Parsing is best-effort: unrecognized input
//! yields `source_tool = unknown, has_metadata = false`, and field-level
//! failures are absorbed rather than propagated.

mod a1111;
mod comfyui;
mod novelai;

use std::collections::HashMap;

use serde_json::{json, Value};

/// Generating tool, serialized to the `source_tool` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTool {
    Comfyui,
    A1111,
    Forge,
    Novelai,
    Unknown,
}

impl SourceTool {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceTool::Comfyui => "comfyui",
            SourceTool::A1111 => "a1111",
            SourceTool::Forge => "forge",
            SourceTool::Novelai => "novelai",
            SourceTool::Unknown => "unknown",
        }
    }
}

/// Coarse model family, serialized to the `model_type` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Sd15,
    Sdxl,
    Pony,
    Illustrious,
    Flux,
    Qwen,
    Other,
}

impl ModelType {
    pub fn as_str(self) -> &'static str {
        match self {
            ModelType::Sd15 => "sd15",
            ModelType::Sdxl => "sdxl",
            ModelType::Pony => "pony",
            ModelType::Illustrious => "illustrious",
            ModelType::Flux => "flux",
            ModelType::Qwen => "qwen",
            ModelType::Other => "other",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoraInfo {
    pub name: String,
    pub weight: f64,
    pub weight_clip: Option<f64>,
    pub hash: Option<String>,
}

impl LoraInfo {
    fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("name".into(), json!(self.name));
        m.insert("weight".into(), json!(self.weight));
        if let Some(wc) = self.weight_clip {
            m.insert("weight_clip".into(), json!(wc));
        }
        if let Some(h) = &self.hash {
            m.insert("hash".into(), json!(h));
        }
        Value::Object(m)
    }
}

#[derive(Debug, Clone)]
pub struct ControlNetInfo {
    pub model: String,
    pub weight: f64,
    pub guidance_start: f64,
    pub guidance_end: f64,
    pub preprocessor: Option<String>,
}

impl ControlNetInfo {
    fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("model".into(), json!(self.model));
        m.insert("weight".into(), json!(self.weight));
        m.insert("guidance_start".into(), json!(self.guidance_start));
        m.insert("guidance_end".into(), json!(self.guidance_end));
        if let Some(p) = &self.preprocessor {
            m.insert("preprocessor".into(), json!(p));
        }
        Value::Object(m)
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingInfo {
    pub name: String,
    pub hash: Option<String>,
}

impl EmbeddingInfo {
    fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("name".into(), json!(self.name));
        if let Some(h) = &self.hash {
            m.insert("hash".into(), json!(h));
        }
        Value::Object(m)
    }
}

/// Normalized metadata extracted from an image (mirror base.ParsedMetadata).
#[derive(Debug, Clone)]
pub struct ParsedMetadata {
    pub source_tool: SourceTool,
    pub model_type: Option<ModelType>,
    pub has_metadata: bool,
    pub positive_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub model_name: Option<String>,
    pub sampler_name: Option<String>,
    pub scheduler: Option<String>,
    pub steps: Option<i32>,
    pub cfg_scale: Option<f64>,
    pub seed: Option<i64>,
    pub loras: Vec<LoraInfo>,
    pub controlnets: Vec<ControlNetInfo>,
    pub embeddings: Vec<EmbeddingInfo>,
    pub model_params: serde_json::Map<String, Value>,
    pub workflow_extras: serde_json::Map<String, Value>,
    pub raw_metadata: Option<Value>,
}

impl ParsedMetadata {
    fn new(source_tool: SourceTool) -> Self {
        Self {
            source_tool,
            model_type: None,
            has_metadata: true,
            positive_prompt: None,
            negative_prompt: None,
            model_name: None,
            sampler_name: None,
            scheduler: None,
            steps: None,
            cfg_scale: None,
            seed: None,
            loras: Vec::new(),
            controlnets: Vec::new(),
            embeddings: Vec::new(),
            model_params: serde_json::Map::new(),
            workflow_extras: serde_json::Map::new(),
            raw_metadata: None,
        }
    }

    /// The fallback for images with no recognizable metadata.
    pub fn unknown() -> Self {
        let mut m = Self::new(SourceTool::Unknown);
        m.has_metadata = false;
        m
    }

    pub fn loras_json(&self) -> Value {
        Value::Array(self.loras.iter().map(LoraInfo::to_json).collect())
    }

    pub fn controlnets_json(&self) -> Value {
        Value::Array(self.controlnets.iter().map(ControlNetInfo::to_json).collect())
    }

    pub fn embeddings_json(&self) -> Value {
        Value::Array(self.embeddings.iter().map(EmbeddingInfo::to_json).collect())
    }

    pub fn model_params_json(&self) -> Value {
        Value::Object(self.model_params.clone())
    }

    pub fn workflow_extras_json(&self) -> Value {
        Value::Object(self.workflow_extras.clone())
    }
}

/// Detect the coarse model family from a model name (mirror model_detector.py).
pub fn detect_model_type(model_name: Option<&str>) -> ModelType {
    let Some(name) = model_name.filter(|s| !s.is_empty()) else {
        return ModelType::Other;
    };
    let n = name.to_lowercase();
    // Priority order matches the Python rules.
    if n.contains("qwen") {
        ModelType::Qwen
    } else if n.contains("flux") {
        ModelType::Flux
    } else if n.contains("pony") {
        ModelType::Pony
    } else if n.contains("illustrious") || n.contains("noob") {
        ModelType::Illustrious
    } else if n.contains("xl") || n.contains("sdxl") {
        ModelType::Sdxl
    } else if n.contains("sd15") || n.contains("v1-5") || n.contains("1.5") || n.contains("sd_1") {
        ModelType::Sd15
    } else {
        ModelType::Other
    }
}

/// Select the matching parser and parse, mirroring MetadataParserFactory.
pub fn parse(png_info: &HashMap<String, String>) -> ParsedMetadata {
    if comfyui::can_parse(png_info) {
        return comfyui::parse(png_info);
    }
    if a1111::can_parse(png_info) {
        return a1111::parse(png_info);
    }
    if novelai::can_parse(png_info) {
        return novelai::parse(png_info);
    }
    ParsedMetadata::unknown()
}
