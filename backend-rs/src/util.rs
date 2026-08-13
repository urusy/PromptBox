//! Small shared helpers.

use std::sync::OnceLock;

/// Round to 2 decimal places, matching Python's `round(x, 2)` closely enough for
/// the rating/weight averages surfaced in the API.
pub fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Shared HTTP client for outbound API calls (CivitAI, Gelbooru). reqwest
/// clients pool connections internally and are cheap to reuse; per-request
/// timeouts are set at the call site.
pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("PromptBox/1.0")
            .build()
            .expect("build shared reqwest client")
    })
}

/// Escape LIKE/ILIKE special characters (`%`, `_`, `\`) so a user-supplied
/// substring is matched literally. Pair with `ESCAPE '\'` in the SQL. Mirrors
/// escape_like_pattern in the Python backend.
pub fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
