//! Build-time provenance for `GET /api/version`.
//!
//! Falcon proxies this backend by hand-mirroring its routes, so "which build am
//! I actually talking to" has to be answerable over HTTP (docs/13 B14).
//!
//! The Docker build context does not include `.git`, so the commit is passed in
//! as the `GIT_SHA` build arg there and only falls back to invoking git for
//! local builds.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("cargo:rerun-if-env-changed=GIT_SHA");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    let sha = std::env::var("GIT_SHA")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(git_sha)
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=PROMPTBOX_GIT_SHA={sha}");

    // Seconds since the epoch; formatted as RFC3339 at runtime so the build
    // script needs no date dependency. SOURCE_DATE_EPOCH keeps reproducible
    // builds reproducible.
    let built_at = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        });
    println!("cargo:rustc-env=PROMPTBOX_BUILT_AT_EPOCH={built_at}");
}

fn git_sha() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!sha.is_empty()).then_some(sha)
}
