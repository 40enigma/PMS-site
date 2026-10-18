use std::sync::OnceLock;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::error::AppError;
use crate::models::AdminClaims;

/// Default cookie name used to store the admin JWT token in the browser.
pub const JWT_COOKIE_NAME: &str = "pms_admin_token";

static RESOLVED_JWT_SECRET: OnceLock<String> = OnceLock::new();
static RESOLVED_COOKIE_NAME: OnceLock<String> = OnceLock::new();

/// Retrieve the cached JWT secret from the environment (read once at startup).
///
/// Behaviour:
/// - If `RUST_ENV=production` and `JWT_SECRET` is unset → panic immediately (fail-fast).
/// - In any other environment, fall back to an insecure placeholder and emit a single warning.
pub fn jwt_secret() -> &'static str {
    RESOLVED_JWT_SECRET.get_or_init(|| {
        let is_production =
            std::env::var("RUST_ENV").unwrap_or_default().to_lowercase() == "production";

        match std::env::var("JWT_SECRET") {
            Ok(secret) if !secret.is_empty() => secret,
            _ => {
                if is_production {
                    eprintln!(
                        "\n\
                        ╔══════════════════════════════════════════════════════════╗\n\
                        ║  FATAL: JWT_SECRET is not set in production mode.        ║\n\
                        ║  Set JWT_SECRET to a cryptographically random value:     ║\n\
                        ║      openssl rand -hex 32                                ║\n\
                        ╚══════════════════════════════════════════════════════════╝\n"
                    );
                    std::process::exit(1);
                } else {
                    tracing::warn!(
                        "⚠️  JWT_SECRET is not set. Using insecure placeholder for development. \
                         DO NOT use in production!"
                    );
                    "pms-dev-insecure-placeholder-do-not-use-in-production".to_string()
                }
            }
        }
    })
}

/// Retrieve the cached cookie name from the environment (read once at startup).
pub fn jwt_cookie_name() -> &'static str {
    RESOLVED_COOKIE_NAME.get_or_init(|| {
        std::env::var("JWT_COOKIE_NAME").unwrap_or_else(|_| JWT_COOKIE_NAME.to_string())
    })
}

/// Create a short-lived (24 h) JWT access token for an authenticated admin user.
pub fn create_token(user_id: i64, email: &str) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = AdminClaims {
        sub: user_id,
        email: email.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token creation failed: {}", e)))
}

/// Verify a JWT token and return the claims, explicitly pinning Algorithm::HS256.
pub fn verify_token(token: &str) -> Result<AdminClaims, AppError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    let token_data = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized)?;

    Ok(token_data.claims)
}

/// Axum extractor for admin authentication.
///
/// Accepts the JWT from (in priority order):
/// 1. `Authorization: Bearer <token>` header — for programmatic API clients.
/// 2. The `pms_admin_token` httpOnly cookie — for browser-based flows.
///
/// Also verifies that the user still exists in `admin_users` table when the db pool is present.
impl<S> FromRequestParts<S> for AdminClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = if let Some(auth_header) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                verify_token(token)?
            } else {
                return Err(AppError::Unauthorized);
            }
        } else {
            let jar = CookieJar::from_headers(&parts.headers);
            if let Some(cookie) = jar.get(jwt_cookie_name()) {
                verify_token(cookie.value())?
            } else {
                return Err(AppError::Unauthorized);
            }
        };

        // If database pool is available in request extensions, verify user account still exists
        if let Some(pool) = parts.extensions.get::<sqlx::SqlitePool>() {
            let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM admin_users WHERE id = ?1")
                .bind(claims.sub)
                .fetch_optional(pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            if exists.is_none() {
                return Err(AppError::Unauthorized);
            }
        }

        Ok(claims)
    }
}
