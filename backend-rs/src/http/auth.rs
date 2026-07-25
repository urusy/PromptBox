//! Authentication HTTP handlers and the `CurrentUser` extractor.

use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::{json, Value};

use super::AppState;
use crate::auth;
use crate::config::Config;
use crate::dto::auth::{LoginRequest, LoginResponse};
use crate::dto::common::MessageResponse;
use crate::error::AppError;

/// Authenticated user, extracted from the `session` cookie. Handlers that take
/// this argument are protected: a missing/invalid cookie yields 401.
pub struct CurrentUser(pub String);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("CookieJar extraction is infallible");
        let token = jar
            .get("session")
            .map(|c| c.value().to_string())
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))?;
        let username = auth::verify_session(&token, &state.config.secret_key)
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".to_string()))?;
        Ok(CurrentUser(username))
    }
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
    if !auth::verify_password(
        &req.username,
        &req.password,
        &state.config.admin_username,
        &state.config.admin_password_hash,
    ) {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let token = auth::create_session(
        &req.username,
        &state.config.secret_key,
        state.config.session_expire_hours,
    )
    .map_err(AppError::Internal)?;

    let secure = cookie_secure(&state.config, &headers);
    let jar = jar.add(build_session_cookie(token, &state.config, secure));
    Ok((
        jar,
        Json(LoginResponse {
            message: "Login successful".to_string(),
            username: req.username,
        }),
    ))
}

/// POST /api/auth/logout (requires authentication, mirroring the Python API)
pub async fn logout(
    State(state): State<AppState>,
    _user: CurrentUser,
    jar: CookieJar,
    headers: HeaderMap,
) -> (CookieJar, Json<MessageResponse>) {
    // The removal cookie must carry the same attributes as the one that was
    // set: a browser on a plain-HTTP origin ignores a `Secure` Set-Cookie, so
    // a mismatched attribute would leave the session cookie in place.
    let secure = cookie_secure(&state.config, &headers);
    let jar = jar.remove(build_session_cookie(String::new(), &state.config, secure));
    (
        jar,
        Json(MessageResponse {
            message: "Logout successful".to_string(),
        }),
    )
}

/// GET /api/auth/me
pub async fn me(CurrentUser(username): CurrentUser) -> Json<Value> {
    Json(json!({ "username": username }))
}

/// Whether this request reached the edge over HTTPS.
///
/// The backend only ever speaks plain HTTP inside the Docker network, so the
/// client's original scheme is visible solely through proxy headers: nginx
/// forwards `X-Forwarded-Proto`, and Cloudflare sets it (plus `CF-Visitor`)
/// for tunnelled traffic.
///
/// Trusting a client-supplied header is safe here because the only thing it
/// can influence is whether the caller's *own* cookie gets `Secure`; it grants
/// no access and cannot affect another session.
fn request_is_https(headers: &HeaderMap) -> bool {
    if let Some(proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
    {
        // Multiple proxies append to the list; the left-most hop is the client.
        let first = proto.split(',').next().unwrap_or_default().trim();
        if !first.is_empty() {
            return first.eq_ignore_ascii_case("https");
        }
    }
    // Fallback for setups where X-Forwarded-Proto is stripped before us.
    headers
        .get("cf-visitor")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.replace(' ', "").contains("\"scheme\":\"https\""))
}

/// Decide the `Secure` attribute for the session cookie.
///
/// This deployment is reachable two ways at once — `http://<nas-ip>:<port>`
/// from the LAN and `https://<domain>` through the Cloudflare tunnel. Browsers
/// silently discard a `Secure` cookie on a plain-HTTP origin, so a fixed value
/// would break one of the two. Deciding per request keeps HTTPS traffic
/// protected while letting LAN clients log in.
fn cookie_secure(cfg: &Config, headers: &HeaderMap) -> bool {
    match cfg.session_cookie_secure {
        Some(forced) => forced,
        None => !cfg.debug && request_is_https(headers),
    }
}

/// Build the session cookie: HttpOnly, `Secure` per [`cookie_secure`],
/// SameSite Strict (Lax in debug), 1-week max-age.
fn build_session_cookie(token: String, cfg: &Config, secure: bool) -> Cookie<'static> {
    Cookie::build(("session", token))
        .http_only(true)
        .secure(secure)
        .same_site(if cfg.debug {
            SameSite::Lax
        } else {
            SameSite::Strict
        })
        .path("/")
        .max_age(time::Duration::days(7))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            );
        }
        h
    }

    /// A request as it arrives behind nginx, which forwards the client scheme.
    fn forwarded(proto: &str) -> HeaderMap {
        headers(&[("x-forwarded-proto", proto)])
    }

    fn cf_visitor(scheme: &str) -> HeaderMap {
        headers(&[("cf-visitor", &format!(r#"{{"scheme":"{scheme}"}}"#))])
    }

    fn prod_config() -> Config {
        Config {
            debug: false,
            ..Config::for_test()
        }
    }

    #[test]
    fn forwarded_proto_decides_scheme() {
        assert!(request_is_https(&forwarded("https")));
        assert!(request_is_https(&forwarded("HTTPS")));
        assert!(!request_is_https(&forwarded("http")));
        // A proxy chain lists the client hop first.
        assert!(request_is_https(&forwarded("https, http")));
        assert!(!request_is_https(&forwarded("http, https")));
    }

    #[test]
    fn cf_visitor_is_the_fallback() {
        assert!(request_is_https(&cf_visitor("https")));
        assert!(!request_is_https(&cf_visitor("http")));
        assert!(request_is_https(&headers(&[(
            "cf-visitor",
            r#"{ "scheme": "https" }"#
        )])));
        // X-Forwarded-Proto wins when both are present.
        assert!(!request_is_https(&headers(&[
            ("x-forwarded-proto", "http"),
            ("cf-visitor", r#"{"scheme":"https"}"#),
        ])));
    }

    #[test]
    fn no_proxy_headers_means_plain_http() {
        assert!(!request_is_https(&HeaderMap::new()));
    }

    #[test]
    fn auto_mode_follows_the_request_scheme() {
        let cfg = prod_config();
        // The regression this fixes: a LAN client on http:// used to receive a
        // Secure cookie, which the browser dropped, so every request stayed 401.
        assert!(!cookie_secure(&cfg, &forwarded("http")));
        assert!(!cookie_secure(&cfg, &HeaderMap::new()));
        assert!(cookie_secure(&cfg, &forwarded("https")));
    }

    #[test]
    fn explicit_override_wins_over_the_request() {
        let forced_on = Config {
            session_cookie_secure: Some(true),
            ..prod_config()
        };
        assert!(cookie_secure(&forced_on, &HeaderMap::new()));

        let forced_off = Config {
            session_cookie_secure: Some(false),
            ..prod_config()
        };
        assert!(!cookie_secure(&forced_off, &forwarded("https")));
    }

    #[test]
    fn debug_never_sets_secure() {
        let cfg = Config::for_test(); // debug = true
        assert!(!cookie_secure(&cfg, &forwarded("https")));
    }

    #[test]
    fn cookie_attributes_match_the_decision() {
        let cfg = prod_config();
        let secure = build_session_cookie("tok".into(), &cfg, true);
        assert!(secure.http_only().unwrap());
        assert!(secure.secure().unwrap());
        assert_eq!(secure.same_site(), Some(SameSite::Strict));
        assert_eq!(secure.path(), Some("/"));

        let insecure = build_session_cookie("tok".into(), &cfg, false);
        assert!(!insecure.secure().unwrap());
    }

    /// POST /api/auth/login through the real router and return its Set-Cookie.
    /// Password checking never touches the database, so a lazy pool suffices.
    async fn login_set_cookie(forwarded_proto: Option<&str>) -> String {
        let mut config = prod_config();
        config.admin_password_hash = bcrypt::hash("secret", 4).unwrap();
        let storage = crate::storage::build(&config).expect("fs store");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://user:pass@localhost/db")
            .expect("lazy pool");
        let jobs = crate::job::Jobs::new(pool.clone());
        let state = AppState {
            config: Arc::new(config),
            pool,
            storage,
            jobs,
        };

        let mut req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            // The login rate limiter keys on the client address. In production
            // that comes from nginx's header or, failing that, from ConnectInfo
            // (main.rs); `oneshot` provides neither, so state it explicitly.
            // A distinct address per test keeps the buckets independent.
            .header(
                "x-forwarded-for",
                match forwarded_proto {
                    Some(_) => "198.51.100.1",
                    None => "198.51.100.2",
                },
            );
        if let Some(proto) = forwarded_proto {
            req = req.header("x-forwarded-proto", proto);
        }
        let body = Body::from(r#"{"username":"admin","password":"secret"}"#);
        let res = super::super::router(state)
            .oneshot(req.body(body).unwrap())
            .await
            .unwrap();

        assert_eq!(res.status(), axum::http::StatusCode::OK);
        res.headers()
            .get("set-cookie")
            .expect("login must set a session cookie")
            .to_str()
            .unwrap()
            .to_string()
    }

    #[tokio::test]
    async fn login_over_https_sets_a_secure_cookie() {
        let cookie = login_set_cookie(Some("https")).await;
        assert!(cookie.contains("Secure"), "{cookie}");
        assert!(cookie.contains("HttpOnly"), "{cookie}");
    }

    #[tokio::test]
    async fn login_over_plain_http_omits_secure() {
        // Regression: a LAN client on http://<nas-ip>:<port> used to get a
        // Secure cookie, which the browser discarded — every subsequent request
        // came back 401 and the SPA bounced to /login forever.
        let cookie = login_set_cookie(None).await;
        assert!(!cookie.contains("Secure"), "{cookie}");
        assert!(cookie.contains("HttpOnly"), "{cookie}");
    }
}
