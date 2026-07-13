//! Image file helpers for the import pipeline (mirror utils/{hash,image,file}_utils.py):
//! SHA-256 hashing, dimensions, WebP thumbnailing, PNG/JPEG metadata extraction,
//! and the content-hash storage layout. These are blocking (CPU/IO) operations,
//! intended to be called from `spawn_blocking`.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// Streamed SHA-256 hex digest of a file (mirror calculate_file_hash).
pub fn file_hash(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(File::open(path).with_context(|| format!("open {path:?}"))?);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// (width, height) without decoding the full image.
pub fn image_dimensions(path: &Path) -> Result<(u32, u32)> {
    image::image_dimensions(path).with_context(|| format!("read dimensions of {path:?}"))
}

/// Encode a WebP thumbnail (lossy, given quality) from in-memory image bytes,
/// shrinking to fit within `size`×`size` while preserving aspect ratio and
/// never upscaling (mirror create_thumbnail with Pillow's thumbnail()).
/// Decoding doubles as content validation before anything is uploaded.
pub fn create_thumbnail_bytes(data: &[u8], size: u32, quality: u8) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data).context("decode image for thumbnail")?;
    let (w, h) = (img.width(), img.height());
    let longest = w.max(h);
    let resized = if longest <= size || longest == 0 {
        img
    } else {
        let scale = size as f64 / longest as f64;
        let nw = ((w as f64 * scale).round() as u32).max(1);
        let nh = ((h as f64 * scale).round() as u32).max(1);
        img.resize(nw, nh, image::imageops::FilterType::Lanczos3)
    };
    let rgb = resized.to_rgb8();
    let encoder = webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height());
    Ok(encoder.encode(quality as f32).to_vec())
}

/// Read text metadata into a flat map (mirror read_image_info). PNG text chunks
/// for `.png`; the A1111 EXIF `parameters` string for `.jpg`/`.jpeg`; empty
/// otherwise. Best-effort: failures yield an empty map.
pub fn read_image_info(path: &Path) -> HashMap<String, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);
    match ext.as_deref() {
        Some("png") => read_png_text(path).unwrap_or_default(),
        Some("jpg") | Some("jpeg") => read_jpeg_params(path).unwrap_or_default(),
        _ => HashMap::new(),
    }
}

fn read_png_text(path: &Path) -> Option<HashMap<String, String>> {
    let file = File::open(path).ok()?;
    let reader = png::Decoder::new(file).read_info().ok()?;
    let info = reader.info();
    let mut map = HashMap::new();
    // Uncompressed tEXt (ComfyUI prompt/workflow, A1111 parameters, NovelAI Comment).
    for chunk in &info.uncompressed_latin1_text {
        map.insert(chunk.keyword.clone(), chunk.text.clone());
    }
    // iTXt (UTF-8) and zTXt (compressed) as fallbacks; don't overwrite tEXt.
    for chunk in &info.utf8_text {
        if let Ok(text) = chunk.get_text() {
            map.entry(chunk.keyword.clone()).or_insert(text);
        }
    }
    for chunk in &info.compressed_latin1_text {
        if let Ok(text) = chunk.get_text() {
            map.entry(chunk.keyword.clone()).or_insert(text);
        }
    }
    Some(map)
}

fn read_jpeg_params(path: &Path) -> Option<HashMap<String, String>> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let field = exif.get_field(exif::Tag::UserComment, exif::In::PRIMARY)?;
    let text = decode_user_comment(&field.value)?;
    if !text.is_empty() && (text.contains("Steps:") || text.contains("Sampler:")) {
        let mut map = HashMap::new();
        map.insert("parameters".to_string(), text.trim().to_string());
        Some(map)
    } else {
        None
    }
}

/// Decode an EXIF UserComment, honoring its 8-byte charset marker (mirror the
/// Python JPEG reader): UNICODE → UTF-16BE, ASCII → ASCII, else UTF-8.
fn decode_user_comment(value: &exif::Value) -> Option<String> {
    let exif::Value::Undefined(bytes, _) = value else {
        return None;
    };
    if bytes.len() < 8 {
        return None;
    }
    let (marker, payload) = bytes.split_at(8);
    let text = if marker == b"UNICODE\0" {
        let units: Vec<u16> = payload
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else if marker == b"ASCII\0\0\0" {
        String::from_utf8_lossy(payload).into_owned()
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    };
    Some(text)
}

/// Relative storage path partitioned by content hash: `ab/cd/<hash><ext>`
/// (mirror get_storage_path). `file_hash` must be a 64-char hex digest.
pub fn storage_path(file_hash: &str, original_filename: &str) -> String {
    let ext = Path::new(original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default();
    format!("{}/{}/{}{}", &file_hash[..2], &file_hash[2..4], file_hash, ext)
}

/// Relative thumbnail path: `thumbnails/ab/cd/<hash>.webp` (mirror get_thumbnail_path).
pub fn thumbnail_path(file_hash: &str) -> String {
    format!(
        "thumbnails/{}/{}/{}.webp",
        &file_hash[..2],
        &file_hash[2..4],
        file_hash
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_paths_partition_by_hash() {
        let hash = "a1b2c3d4e5f6000000000000000000000000000000000000000000000000abcd";
        assert_eq!(
            storage_path(hash, "PIC.PNG"),
            format!("a1/b2/{hash}.png")
        );
        assert_eq!(
            thumbnail_path(hash),
            format!("thumbnails/a1/b2/{hash}.webp")
        );
    }
}
