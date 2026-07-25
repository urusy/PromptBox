//! Machine-readable route manifest (docs/13 A2a).
//!
//! Falcon proxies this backend through a hand-maintained table of regexes
//! (`gateway/internal/infrastructure/promptbox/proxy_routes.go`), whose own
//! comment admits the failure mode:
//!
//! > When PromptBox's router changes, update this table — the pass-through does
//! > not validate payloads, so a silent drift would only surface as a 404 in
//! > the UI.
//!
//! Publishing the route table turns that into a test Falcon can run in CI.
//! Unauthenticated on purpose: a client has to be able to check compatibility
//! before it has a session.

use axum::Json;
use serde::Serialize;

use super::images::ListQuery;

/// One route, as the API contract sees it.
#[derive(Debug, Serialize)]
pub struct RouteSpec {
    /// Full path including the `/api` prefix, with axum-style `{param}`
    /// placeholders.
    pub path: &'static str,
    pub methods: &'static [&'static str],
    /// Whether a valid `session` cookie is required.
    pub auth: bool,
    /// Query parameters the endpoint understands, where a documented set
    /// exists. Sending anything else is reported via `warnings[]` (A3).
    #[serde(skip_serializing_if = "is_empty")]
    pub query: &'static [&'static str],
}

fn is_empty(s: &&'static [&'static str]) -> bool {
    s.is_empty()
}

const GET: &[&str] = &["GET"];
const POST: &[&str] = &["POST"];
const NONE: &[&str] = &[];

/// The API surface. **Keep in sync with `router()` in this module** — the test
/// at the bottom of this file fails if the two drift apart.
pub const ROUTES: &[RouteSpec] = &[
    // Unauthenticated: liveness, identity, and the object stream nginx proxies.
    RouteSpec { path: "/", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/health", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/health/db", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/storage/{*path}", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/api/health", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/api/health/db", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/api/version", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/api/_manifest", methods: GET, auth: false, query: NONE },
    RouteSpec { path: "/api/auth/login", methods: POST, auth: false, query: NONE },
    // Everything below requires a session.
    RouteSpec { path: "/api/config", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/auth/logout", methods: POST, auth: true, query: NONE },
    RouteSpec { path: "/api/auth/me", methods: GET, auth: true, query: NONE },
    RouteSpec {
        path: "/api/images",
        methods: GET,
        auth: true,
        query: ListQuery::KNOWN_PARAMS,
    },
    RouteSpec {
        path: "/api/images/{id}",
        methods: &["GET", "PATCH", "DELETE"],
        auth: true,
        // GET accepts the listing parameters too (they define prev/next
        // context); DELETE accepts ?permanent=true.
        query: ListQuery::KNOWN_PARAMS,
    },
    RouteSpec { path: "/api/images/{id}/restore", methods: POST, auth: true, query: NONE },
    RouteSpec {
        path: "/api/search-presets",
        methods: &["GET", "POST"],
        auth: true,
        query: NONE,
    },
    RouteSpec {
        path: "/api/search-presets/{id}",
        methods: &["PUT", "DELETE"],
        auth: true,
        query: NONE,
    },
    RouteSpec {
        path: "/api/smart-folders",
        methods: &["GET", "POST"],
        auth: true,
        query: NONE,
    },
    RouteSpec {
        path: "/api/smart-folders/{id}",
        methods: &["GET", "PUT", "DELETE"],
        auth: true,
        query: NONE,
    },
    RouteSpec { path: "/api/tags", methods: GET, auth: true, query: &["q", "limit"] },
    RouteSpec {
        path: "/api/changes",
        methods: GET,
        auth: true,
        query: &["since", "limit", "compact"],
    },
    RouteSpec {
        path: "/api/jobs",
        methods: &["GET", "POST"],
        auth: true,
        query: &["status", "limit"],
    },
    RouteSpec { path: "/api/jobs/{id}", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/jobs/{id}/cancel", methods: POST, auth: true, query: NONE },
    RouteSpec { path: "/api/bulk/update", methods: POST, auth: true, query: NONE },
    RouteSpec { path: "/api/bulk/delete", methods: POST, auth: true, query: NONE },
    RouteSpec { path: "/api/bulk/restore", methods: POST, auth: true, query: NONE },
    RouteSpec {
        path: "/api/duplicates",
        methods: &["GET", "DELETE"],
        auth: true,
        query: NONE,
    },
    RouteSpec { path: "/api/duplicates/{filename}", methods: &["DELETE"], auth: true, query: NONE },
    RouteSpec { path: "/api/export/metadata", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/export/prompts", methods: GET, auth: true, query: NONE },
    RouteSpec {
        path: "/api/showcases",
        methods: &["GET", "POST"],
        auth: true,
        query: NONE,
    },
    RouteSpec { path: "/api/showcases/check-images", methods: POST, auth: true, query: NONE },
    RouteSpec {
        path: "/api/showcases/{id}",
        methods: &["GET", "PUT", "DELETE"],
        auth: true,
        query: NONE,
    },
    RouteSpec {
        path: "/api/showcases/{id}/images",
        methods: &["POST", "DELETE"],
        auth: true,
        query: NONE,
    },
    RouteSpec { path: "/api/showcases/{id}/images/reorder", methods: &["PUT"], auth: true, query: NONE },
    RouteSpec { path: "/api/stats", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/stats/models-for-analysis", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/stats/loras-for-filter", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/stats/samplers-for-filter", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/stats/rating-analysis", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/stats/model-rating-distribution", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/models", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/models/{model_name}/detail", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/models/{model_name}/civitai", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/loras", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/loras/{lora_name}/detail", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/loras/{lora_name}/civitai", methods: GET, auth: true, query: NONE },
    RouteSpec { path: "/api/gelbooru/tags", methods: GET, auth: true, query: NONE },
];

#[derive(Debug, Serialize)]
pub struct ManifestResponse {
    /// Crate version, so a client can tell which build produced this table.
    pub version: &'static str,
    pub routes: &'static [RouteSpec],
}

/// GET /api/_manifest
pub async fn manifest() -> Json<ManifestResponse> {
    Json(ManifestResponse {
        version: env!("CARGO_PKG_VERSION"),
        routes: ROUTES,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    /// Paths as written in the `.route("…")` calls of `http::mod`.
    fn paths_in_router_source() -> Vec<String> {
        let source = include_str!("mod.rs");
        let re = Regex::new(r#"\.route\(\s*"([^"]+)""#).expect("valid regex");
        let mut paths: Vec<String> = re
            .captures_iter(source)
            .map(|c| c[1].to_string())
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }

    /// Manifest paths reduced to the same form (the `/api` prefix comes from
    /// `.nest("/api", …)`, not from the individual route calls).
    fn paths_in_manifest() -> Vec<String> {
        let mut paths: Vec<String> = ROUTES
            .iter()
            .map(|r| {
                r.path
                    .strip_prefix("/api")
                    .filter(|rest| !rest.is_empty())
                    .unwrap_or(r.path)
                    .to_string()
            })
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }

    /// axum cannot enumerate a built Router, so the router source is the thing
    /// to compare against. Crude, but it catches exactly the drift that Falcon
    /// can only see as a 404: a route added (or removed) without updating the
    /// manifest.
    #[test]
    fn manifest_covers_exactly_the_routes_the_router_defines() {
        assert_eq!(
            paths_in_router_source(),
            paths_in_manifest(),
            "the route table in manifest.rs is out of sync with router() in mod.rs"
        );
    }

    #[test]
    fn every_route_declares_at_least_one_method() {
        for route in ROUTES {
            assert!(
                !route.methods.is_empty(),
                "{} declares no methods",
                route.path
            );
        }
    }

    #[test]
    fn manifest_has_no_duplicate_paths() {
        let mut seen: Vec<&str> = ROUTES.iter().map(|r| r.path).collect();
        let before = seen.len();
        seen.sort();
        seen.dedup();
        assert_eq!(before, seen.len(), "duplicate path in the manifest");
    }
}
