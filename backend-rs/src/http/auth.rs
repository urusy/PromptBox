//! Authentication HTTP handlers and the `CurrentUser` extractor.

use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::{json, Value};

use super::AppState;
use crate::auth;
use crate::config::Config;
use crate::dto::auth::{LoginRequest, LoginResponse, MessageResponse};
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

    let jar = jar.add(build_session_cookie(token, &state.config));
    Ok((
        jar,
        Json(LoginResponse {
            message: "Login successful".to_string(),
            username: req.username,
        }),
    ))
}

/// POST /api/auth/logout (requires authentication, mirroring the Python API)
pub async fn logout(_user: CurrentUser, jar: CookieJar) -> (CookieJar, Json<MessageResponse>) {
    let jar = jar.remove(Cookie::build(("session", "")).path("/").build());
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

/// Build the session cookie with attributes matching the Python backend:
/// HttpOnly, Secure unless debug, SameSite Strict (Lax in debug), 1-week max-age.
fn build_session_cookie(token: String, cfg: &Config) -> Cookie<'static> {
    Cookie::build(("session", token))
        .http_only(true)
        .secure(!cfg.debug)
        .same_site(if cfg.debug {
            SameSite::Lax
        } else {
            SameSite::Strict
        })
        .path("/")
        .max_age(time::Duration::days(7))
        .build()
}
