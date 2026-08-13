//! Contract tests for the route manifest (docs/13 A2a).
//!
//! The manifest is only useful if it is true, so this drives the real router:
//! every declared path must exist, and every path declared as authenticated
//! must actually reject an anonymous caller. A unit test in `http::manifest`
//! covers the other direction (routes that exist but are not declared).

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use common::{get_json, session_cookie, test_router};
use promptbox::http::manifest::ROUTES;
use sqlx::PgPool;
use tower::ServiceExt;

/// Replace `{param}` placeholders with values that parse.
fn concrete_path(path: &str) -> String {
    path.replace("{id}", "0198f0e2-0000-7000-8000-000000000000")
        .replace("{filename}", "example.png")
        .replace("{model_name}", "SomeModel")
        .replace("{lora_name}", "SomeLora")
        .replace("{*path}", "ab/cd/example.png")
}

async fn request(
    router: Router,
    method: &str,
    uri: &str,
    cookie: Option<String>,
) -> StatusCode {
    let mut builder = Request::builder()
        .uri(uri)
        .method(method)
        // /api/auth/login is rate-limited by client address, which comes from
        // this header or from ConnectInfo (set up in main.rs). `oneshot`
        // provides no connection info, so the header stands in for it.
        .header("x-forwarded-for", "198.51.100.50");
    if let Some(c) = cookie {
        builder = builder.header("cookie", c);
    }
    router
        .oneshot(builder.body(Body::empty()).expect("build request"))
        .await
        .expect("router response")
        .status()
}

#[sqlx::test(migrations = "./migrations")]
async fn manifest_is_public_and_lists_routes(pool: PgPool) {
    let (status, body) = get_json(test_router(pool), "/api/_manifest", None).await;

    assert_eq!(status, StatusCode::OK, "no session required");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));

    let routes = body["routes"].as_array().expect("routes array");
    assert_eq!(routes.len(), ROUTES.len());
    // Spot-check the shape Falcon reads.
    let images = routes
        .iter()
        .find(|r| r["path"] == "/api/images")
        .expect("/api/images is listed");
    assert_eq!(images["methods"][0], "GET");
    assert_eq!(images["auth"], true);
    assert!(
        images["query"]
            .as_array()
            .is_some_and(|q| q.iter().any(|p| p == "sampler_name")),
        "the documented query parameters are published"
    );
}

/// Every declared path must be registered in the router. TRACE is never
/// registered, so axum answers 405 when the *path* matches and 404 when it does
/// not — which is exactly the distinction being tested.
#[sqlx::test(migrations = "./migrations")]
async fn every_declared_path_exists(pool: PgPool) {
    for route in ROUTES {
        let uri = concrete_path(route.path);
        let status = request(test_router(pool.clone()), "TRACE", &uri, None).await;
        assert_eq!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{} is declared in the manifest but the router does not serve it",
            route.path
        );
    }
}

/// Routes declared `auth: true` must reject an anonymous caller — for every
/// method they declare, not just GET.
#[sqlx::test(migrations = "./migrations")]
async fn authenticated_routes_reject_anonymous_callers(pool: PgPool) {
    for route in ROUTES.iter().filter(|r| r.auth) {
        let uri = concrete_path(route.path);
        for method in route.methods {
            let status = request(test_router(pool.clone()), method, &uri, None).await;
            assert_eq!(
                status,
                StatusCode::UNAUTHORIZED,
                "{method} {} must require a session",
                route.path
            );
        }
    }
}

/// The converse: routes declared `auth: false` must be reachable without a
/// session (any status other than 401).
#[sqlx::test(migrations = "./migrations")]
async fn public_routes_do_not_require_a_session(pool: PgPool) {
    for route in ROUTES.iter().filter(|r| !r.auth) {
        let uri = concrete_path(route.path);
        for method in route.methods {
            let status = request(test_router(pool.clone()), method, &uri, None).await;
            assert_ne!(
                status,
                StatusCode::UNAUTHORIZED,
                "{method} {} is declared public but demanded a session",
                route.path
            );
        }
    }
}

/// A GET on an authenticated route with a valid session must get past the auth
/// layer (i.e. never 401), so the manifest's `auth` flag is not just a label.
#[sqlx::test(migrations = "./migrations")]
async fn session_unlocks_authenticated_get_routes(pool: PgPool) {
    for route in ROUTES
        .iter()
        .filter(|r| r.auth && r.methods.contains(&"GET"))
    {
        let uri = concrete_path(route.path);
        let status = request(
            test_router(pool.clone()),
            "GET",
            &uri,
            Some(session_cookie()),
        )
        .await;
        assert_ne!(
            status,
            StatusCode::UNAUTHORIZED,
            "GET {} rejected a valid session",
            route.path
        );
    }
}
