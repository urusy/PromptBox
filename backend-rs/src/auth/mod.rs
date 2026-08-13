//! Authentication logic: password verification and JWT session tokens.
//!
//! Mirrors backend/app/services/auth_service.py. The JWT uses HS256 with the
//! same SECRET_KEY, so tokens issued by the Python backend remain valid here —
//! no re-login is needed at the strangler-fig cutover.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

/// Verify username and password against the configured admin credentials.
pub fn verify_password(
    username: &str,
    password: &str,
    admin_username: &str,
    admin_hash: &str,
) -> bool {
    if username != admin_username || admin_hash.is_empty() {
        return false;
    }
    bcrypt::verify(password, admin_hash).unwrap_or(false)
}

/// Create a JWT session token (HS256), matching create_session.
pub fn create_session(username: &str, secret: &str, expire_hours: i64) -> anyhow::Result<String> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: username.to_string(),
        exp: (now + chrono::Duration::hours(expire_hours)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

/// Verify a session token and return the username if valid.
pub fn verify_session(token: &str, secret: &str) -> Option<String> {
    let mut validation = Validation::new(Algorithm::HS256);
    // python-jose verifies exp but does not require aud/iss. jsonwebtoken
    // validates aud by default and would reject tokens without an aud claim, so
    // disable it to stay compatible with Python-issued tokens.
    validation.validate_aud = false;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn session_roundtrip_returns_username() {
        let token = create_session("admin", SECRET, 24).unwrap();
        assert_eq!(verify_session(&token, SECRET).as_deref(), Some("admin"));
    }

    #[test]
    fn session_with_wrong_secret_fails() {
        let token = create_session("admin", SECRET, 24).unwrap();
        assert!(verify_session(&token, "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ").is_none());
    }

    #[test]
    fn session_garbage_token_fails() {
        assert!(verify_session("not.a.valid.jwt", SECRET).is_none());
    }

    #[test]
    fn expired_session_fails() {
        // expire_hours = -1 puts exp in the past; jsonwebtoken's default leeway
        // (60s) is far smaller than an hour, so verification must fail.
        let token = create_session("admin", SECRET, -1).unwrap();
        assert!(verify_session(&token, SECRET).is_none());
    }

    #[test]
    fn password_verification() {
        let hash = bcrypt::hash("secret", 4).unwrap();
        assert!(verify_password("admin", "secret", "admin", &hash));
        assert!(!verify_password("admin", "wrong", "admin", &hash));
        assert!(!verify_password("bob", "secret", "admin", &hash)); // wrong username
        assert!(!verify_password("admin", "secret", "admin", "")); // no configured hash
    }
}
