//! Non-breaking request warnings (docs/13 A3).
//!
//! The API silently drops what it does not understand: serde discards unknown
//! query parameters, `per_page` is clamped, and an unrecognised `sort_by` falls
//! back to `created_at`. That combination already caused a real incident —
//! Falcon sent `sampler` instead of `sampler_name` and the filter simply never
//! applied (see the note in Falcon's client.go).
//!
//! `deny_unknown_fields` would fix it by breaking every existing client, so
//! instead every request that was not taken literally reports why:
//!
//!   * `warnings[]` in the response body (omitted when empty, so existing
//!     clients see no change), and
//!   * an `X-PromptBox-Warnings` header for clients that only look at headers.
//!
//! `?strict=true` turns the same warnings into a 400, which is what CI and
//! staging should use.

use axum::http::{HeaderMap, HeaderValue};
use serde::Serialize;

/// Response header carrying a compact form of the warnings.
pub const HEADER: &str = "x-promptbox-warnings";

#[derive(Debug, Clone, Serialize)]
pub struct Warning {
    /// Machine-readable kind: `unknown_param` | `clamped` | `fallback`.
    pub code: &'static str,
    /// The query parameter this is about.
    pub param: String,
    /// Human-readable explanation of what the server did instead.
    pub message: String,
    /// Suggested correction, when one can be guessed (typo in a parameter name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Warnings collected while interpreting one request.
#[derive(Debug, Default)]
pub struct Warnings(Vec<Warning>);

impl Warnings {
    /// A query parameter the server does not know, with a "did you mean" hint
    /// when a known name is one or two edits away.
    pub fn unknown_param(&mut self, param: &str, known: &[&str]) {
        let hint = closest(param, known).map(|c| format!("did you mean {c}?"));
        self.0.push(Warning {
            code: "unknown_param",
            param: param.to_string(),
            message: format!("unknown query parameter {param:?}; it was ignored"),
            hint,
        });
    }

    /// A value outside the accepted range that was silently pulled inside it.
    pub fn clamped(&mut self, param: &str, requested: i64, applied: i64) {
        self.0.push(Warning {
            code: "clamped",
            param: param.to_string(),
            message: format!("{param}={requested} is out of range; used {applied}"),
            hint: None,
        });
    }

    /// A value that is not part of an allowed set, replaced by a default.
    pub fn fallback(&mut self, param: &str, requested: &str, applied: &str, allowed: &[&str]) {
        self.0.push(Warning {
            code: "fallback",
            param: param.to_string(),
            message: format!("{param}={requested:?} is not supported; used {applied:?}"),
            hint: Some(format!("allowed values: {}", allowed.join(", "))),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_vec(self) -> Vec<Warning> {
        self.0
    }

    /// One-line summary, used for the header and for the `strict` error body.
    pub fn summary(&self) -> String {
        self.0
            .iter()
            .map(|w| format!("{}: {}", w.code, w.message))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Header map to merge into the response. Empty when there is nothing to
    /// report (never emits a blank header).
    pub fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if self.0.is_empty() {
            return headers;
        }
        // Header values must be visible ASCII; parameter names are, but a
        // rejected *value* could be anything, so drop the header rather than
        // fail the request if it is not encodable.
        if let Ok(value) = HeaderValue::from_str(&self.summary()) {
            headers.insert(HEADER, value);
        }
        headers
    }
}

/// Query parameter names present in `raw_query` that are not in `known`.
/// Duplicates are reported once, in first-seen order.
pub fn unknown_params(raw_query: Option<&str>, known: &[&str]) -> Vec<String> {
    let Some(raw) = raw_query.filter(|q| !q.is_empty()) else {
        return Vec::new();
    };
    let mut seen: Vec<String> = Vec::new();
    for pair in raw.split('&').filter(|p| !p.is_empty()) {
        let key = pair.split('=').next().unwrap_or_default();
        let key = percent_decode(key);
        if key.is_empty() || known.contains(&key.as_str()) || seen.contains(&key) {
            continue;
        }
        seen.push(key);
    }
    seen
}

/// Minimal percent-decoding for parameter *names* (values are handled by serde).
/// `+` is a space in form encoding.
fn percent_decode(s: &str) -> String {
    let bytes = s.replace('+', " ").into_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The known parameter closest to `candidate`: within two edits, or a prefix
/// match either way. The prefix rule is what catches the incident this exists
/// for — `sampler` is seven edits from `sampler_name`, but it is its prefix.
fn closest(candidate: &str, known: &[&str]) -> Option<String> {
    let lower = candidate.to_lowercase();
    known
        .iter()
        .filter_map(|k| {
            let distance = levenshtein(&lower, k);
            let prefix = k.starts_with(&lower) || lower.starts_with(k);
            (distance <= 2 || prefix).then_some((distance, *k))
        })
        .min_by_key(|(distance, k)| (*distance, k.len()))
        .map(|(_, k)| k.to_string())
}

/// Plain iterative Levenshtein distance (parameter names are short).
fn levenshtein(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut current = vec![0usize; b_chars.len() + 1];

    for (i, ca) in a.chars().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b_chars.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            current[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(current[j] + 1);
        }
        std::mem::swap(&mut prev, &mut current);
    }
    prev[b_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN: &[&str] = &["page", "per_page", "sampler_name", "sort_by", "q"];

    #[test]
    fn unknown_params_reports_only_unrecognised_keys() {
        let found = unknown_params(Some("page=1&sampler=euler&q=cat"), KNOWN);
        assert_eq!(found, vec!["sampler"]);
    }

    #[test]
    fn unknown_params_handles_empty_and_valueless_keys() {
        assert!(unknown_params(None, KNOWN).is_empty());
        assert!(unknown_params(Some(""), KNOWN).is_empty());
        assert_eq!(unknown_params(Some("flag"), KNOWN), vec!["flag"]);
        assert_eq!(
            unknown_params(Some("dup=1&dup=2"), KNOWN),
            vec!["dup"],
            "a repeated unknown key is reported once"
        );
    }

    #[test]
    fn unknown_params_decodes_encoded_names() {
        assert_eq!(unknown_params(Some("sort%5Fby=rating"), KNOWN).len(), 0);
    }

    /// The incident this feature exists for: `sampler` silently ignored while
    /// the real parameter is `sampler_name`.
    #[test]
    fn hint_points_at_the_intended_parameter() {
        let mut w = Warnings::default();
        w.unknown_param("sampler", KNOWN);
        let warning = &w.into_vec()[0];
        assert_eq!(warning.code, "unknown_param");
        assert_eq!(warning.hint.as_deref(), Some("did you mean sampler_name?"));
    }

    #[test]
    fn hint_is_absent_for_unrelated_names() {
        let mut w = Warnings::default();
        w.unknown_param("completely_different", KNOWN);
        assert_eq!(w.into_vec()[0].hint, None);
    }

    #[test]
    fn header_is_omitted_when_there_is_nothing_to_report() {
        assert!(Warnings::default().headers().is_empty());
    }

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("sort_by", "sort_by"), 0);
    }
}
