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

/// Import failures that are about the image being too big, kept as their own
/// error type so the worker can classify them as permanent without matching on
/// error strings.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("{width}x{height} ({pixels} pixels) exceeds the IMPORT_MAX_PIXELS limit of {limit}")]
    TooManyPixels {
        width: u32,
        height: u32,
        pixels: u64,
        limit: u64,
    },
    #[error("image is too large to decode in memory and cannot be streamed: {reason}")]
    StreamingUnsupported { reason: String },
}

/// Thumbnail dimensions for a `w`×`h` source: fit inside `size`×`size`, keep the
/// aspect ratio, never upscale (mirror Pillow's thumbnail()).
fn thumb_dimensions(w: u32, h: u32, size: u32) -> (u32, u32) {
    let longest = w.max(h);
    if longest <= size || longest == 0 {
        return (w.max(1), h.max(1));
    }
    let scale = size as f64 / longest as f64;
    (
        ((w as f64 * scale).round() as u32).max(1),
        ((h as f64 * scale).round() as u32).max(1),
    )
}

/// Encode a WebP thumbnail (lossy, given quality) from in-memory image bytes,
/// shrinking to fit within `size`×`size` while preserving aspect ratio and
/// never upscaling (mirror create_thumbnail with Pillow's thumbnail()).
/// Decoding doubles as content validation before anything is uploaded.
pub fn create_thumbnail_bytes(data: &[u8], size: u32, quality: u8) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data).context("decode image for thumbnail")?;
    let (w, h) = (img.width(), img.height());
    let (nw, nh) = thumb_dimensions(w, h, size);
    let resized = if (nw, nh) == (w, h) {
        img
    } else {
        img.resize(nw, nh, image::imageops::FilterType::Lanczos3)
    };
    let rgb = resized.to_rgb8();
    let encoder = webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height());
    Ok(encoder.encode(quality as f32).to_vec())
}

/// Encode a WebP thumbnail from a PNG **without ever holding the whole image in
/// memory**: scanlines are decoded one at a time and box-filtered (area average)
/// straight into the thumbnail grid. Peak memory is one scanline plus the small
/// accumulator, no matter how large the source is — which is what lets grids of
/// hundreds of megapixels import inside a 1GB container.
///
/// Only non-interlaced PNGs qualify: Adam7 delivers pixels out of order, which a
/// single-pass row accumulator cannot reassemble.
pub fn create_thumbnail_streaming_png(path: &Path, size: u32, quality: u8) -> Result<Vec<u8>> {
    let is_png = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("png"));
    if !is_png {
        return Err(MediaError::StreamingUnsupported {
            reason: format!("only PNG can be thumbnailed by streaming, got {path:?}"),
        }
        .into());
    }

    let file = File::open(path).with_context(|| format!("open {path:?}"))?;
    let mut decoder = png::Decoder::new(BufReader::new(file));
    // Expand palettes and sub-byte depths, and drop 16-bit precision, so every
    // row arrives as 8 bits per sample.
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder
        .read_info()
        .with_context(|| format!("read PNG header of {path:?}"))?;

    let (w, h, interlaced) = {
        let info = reader.info();
        (info.width, info.height, info.interlaced)
    };
    if interlaced {
        return Err(MediaError::StreamingUnsupported {
            reason: format!("{path:?} is an interlaced PNG"),
        }
        .into());
    }
    if w == 0 || h == 0 {
        return Err(MediaError::StreamingUnsupported {
            reason: format!("{path:?} has zero width or height"),
        }
        .into());
    }
    let (color, depth) = reader.output_color_type();
    if depth != png::BitDepth::Eight {
        return Err(MediaError::StreamingUnsupported {
            reason: format!("{path:?} did not normalize to 8 bits per sample ({depth:?})"),
        }
        .into());
    }
    let channels = color.samples();

    let (tw, th) = thumb_dimensions(w, h, size);
    let cells = tw as usize * th as usize;
    // u64 sums: the largest possible bin holds IMPORT_MAX_PIXELS samples of 255,
    // orders of magnitude below saturation.
    let mut sums = vec![0u64; cells * 3];
    let mut counts = vec![0u64; cells];

    let mut y: u32 = 0;
    while let Some(row) = reader
        .next_row()
        .with_context(|| format!("decode PNG row of {path:?}"))?
    {
        if y >= h {
            break;
        }
        let ty = ((y as u64 * th as u64) / h as u64).min(th as u64 - 1) as usize;
        let row_base = ty * tw as usize;
        for (x, px) in row
            .data()
            .chunks_exact(channels)
            .enumerate()
            .take(w as usize)
        {
            // Grayscale (1) and grayscale+alpha (2) put luma in the first byte;
            // RGB (3) and RGBA (4) put the colour in the first three.
            let (r, g, b) = match channels {
                1 | 2 => (px[0], px[0], px[0]),
                _ => (px[0], px[1], px[2]),
            };
            let tx = ((x as u64 * tw as u64) / w as u64).min(tw as u64 - 1) as usize;
            let cell = row_base + tx;
            sums[cell * 3] += r as u64;
            sums[cell * 3 + 1] += g as u64;
            sums[cell * 3 + 2] += b as u64;
            counts[cell] += 1;
        }
        y += 1;
    }

    let mut rgb = vec![0u8; cells * 3];
    for ((out, sum), count) in rgb
        .chunks_exact_mut(3)
        .zip(sums.chunks_exact(3))
        .zip(counts.iter())
    {
        // Shrinking guarantees every cell got at least one sample; max(1) just
        // keeps a degenerate case from dividing by zero.
        let n = (*count).max(1);
        for (channel, total) in out.iter_mut().zip(sum.iter()) {
            *channel = (total / n) as u8;
        }
    }

    let encoder = webp::Encoder::from_rgb(&rgb, tw, th);
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

    #[test]
    fn thumb_dimensions_fit_the_box_without_upscaling() {
        assert_eq!(thumb_dimensions(1000, 500, 300), (300, 150));
        assert_eq!(thumb_dimensions(500, 1000, 300), (150, 300));
        // Already smaller than the box — left alone.
        assert_eq!(thumb_dimensions(200, 100, 300), (200, 100));
        assert_eq!(thumb_dimensions(300, 300, 300), (300, 300));
        // Extreme aspect ratios still keep at least one pixel per side.
        assert_eq!(thumb_dimensions(10_000, 1, 300), (300, 1));
    }

    /// Write a deterministic gradient PNG to a uniquely named temp file.
    fn write_gradient_png(name: &str, w: u32, h: u32) -> std::path::PathBuf {
        let mut img = image::RgbImage::new(w, h);
        for (x, y, px) in img.enumerate_pixels_mut() {
            *px = image::Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8]);
        }
        let path = std::env::temp_dir().join(format!("promptbox_{name}.png"));
        img.save(&path).expect("write test png");
        path
    }

    #[test]
    fn streaming_thumbnail_agrees_with_full_decode() {
        let path = write_gradient_png("stream_vs_decode", 240, 160);
        let streamed = create_thumbnail_streaming_png(&path, 60, 90).unwrap();
        let decoded = create_thumbnail_bytes(&std::fs::read(&path).unwrap(), 60, 90).unwrap();

        let a = image::load_from_memory(&streamed).unwrap().to_rgb8();
        let b = image::load_from_memory(&decoded).unwrap().to_rgb8();
        assert_eq!(a.dimensions(), (60, 40));
        assert_eq!(a.dimensions(), b.dimensions(), "same thumbnail geometry");

        // A box filter and Lanczos3 disagree per pixel by design; what must hold
        // is that the streamed thumbnail shows the same image overall.
        let mean = |img: &image::RgbImage| -> f64 {
            let total: u64 = img
                .pixels()
                .map(|p| p.0.iter().map(|c| *c as u64).sum::<u64>())
                .sum();
            total as f64 / (img.width() * img.height() * 3) as f64
        };
        let (ma, mb) = (mean(&a), mean(&b));
        assert!((ma - mb).abs() < 6.0, "streamed mean {ma} vs decoded mean {mb}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn streaming_thumbnail_handles_grayscale() {
        let mut img = image::GrayImage::new(120, 90);
        for (x, y, px) in img.enumerate_pixels_mut() {
            *px = image::Luma([((x + y) % 256) as u8]);
        }
        let path = std::env::temp_dir().join("promptbox_gray_stream.png");
        img.save(&path).expect("write test png");

        let thumb = create_thumbnail_streaming_png(&path, 30, 90).unwrap();
        let out = image::load_from_memory(&thumb).unwrap().to_rgb8();
        assert_eq!(out.dimensions(), (30, 23));
        // Luma is copied into all three channels; lossy WebP may spread it a
        // little, but the result must still read as neutral gray.
        for p in out.pixels() {
            let [r, g, b] = p.0;
            let spread = r.max(g).max(b) - r.min(g).min(b);
            assert!(spread <= 12, "expected near-neutral gray, got {:?}", p.0);
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn streaming_thumbnail_rejects_what_it_cannot_stream() {
        let path = std::env::temp_dir().join("promptbox_not_a_png.jpg");
        std::fs::write(&path, b"not really a jpeg").expect("write test file");
        let err = create_thumbnail_streaming_png(&path, 60, 90).unwrap_err();
        assert!(
            matches!(
                err.downcast_ref::<MediaError>(),
                Some(MediaError::StreamingUnsupported { .. })
            ),
            "expected a typed MediaError, got: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }
}
