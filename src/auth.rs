use std::sync::OnceLock;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::models::AdminClaims;
use crate::util::is_production;

/// Default cookie name used to store the admin JWT token in the browser.
pub const JWT_COOKIE_NAME: &str = "pms_admin_token";

/// Compute SHA-256 hash of a JWT string for revocation tracking.
#[must_use]
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

static RESOLVED_JWT_SECRET: OnceLock<String> = OnceLock::new();
static RESOLVED_COOKIE_NAME: OnceLock<String> = OnceLock::new();

/// Validate JWT secret string against production security standards.
///
/// # Errors
/// Returns an error message if the secret is shorter than 32 characters or contains placeholder text.
pub fn validate_jwt_secret_candidate(secret: &str) -> Result<(), &'static str> {
    if secret.len() < 32 {
        return Err("JWT_SECRET must be at least 32 characters long");
    }
    if secret.contains("replace-with") || secret.contains("placeholder") {
        return Err("JWT_SECRET must not contain 'replace-with' or 'placeholder'");
    }
    Ok(())
}

fn generate_ephemeral_dev_secret() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let mut hex = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// Retrieve the cached JWT secret from the environment (read once at startup).
///
/// Behaviour:
/// - If `RUST_ENV=production` and `JWT_SECRET` is unset, <32 chars, or contains
///   "replace-with"/"placeholder" → exit immediately with FATAL message (fail-fast).
/// - In any other environment, generate an ephemeral random 256-bit secret and emit a warning.
pub fn jwt_secret() -> &'static str {
    RESOLVED_JWT_SECRET.get_or_init(|| {
        let prod = is_production();

        match std::env::var("JWT_SECRET") {
            Ok(secret) if !secret.is_empty() => {
                if prod {
                    if let Err(err) = validate_jwt_secret_candidate(&secret) {
                        eprintln!(
                            "\n\
                            ╔══════════════════════════════════════════════════════════╗\n\
                            ║  FATAL: Insecure JWT_SECRET in production mode:          ║\n\
                            ║  {err:<56}║\n\
                            ║  Set JWT_SECRET to a cryptographically random value:     ║\n\
                            ║      openssl rand -hex 32                                ║\n\
                            ╚══════════════════════════════════════════════════════════╝\n"
                        );
                        std::process::exit(1);
                    }
                }
                secret
            }
            _ => {
                if prod {
                    eprintln!(
                        "\n\
                        ╔══════════════════════════════════════════════════════════╗\n\
                        ║  FATAL: JWT_SECRET is not set in production mode.        ║\n\
                        ║  Set JWT_SECRET to a cryptographically random value:     ║\n\
                        ║      openssl rand -hex 32                                ║\n\
                        ╚══════════════════════════════════════════════════════════╝\n"
                    );
                    std::process::exit(1);
                }
                tracing::warn!(
                    "⚠️  JWT_SECRET is not set. Generating an ephemeral random secret for this \
                     development process. DO NOT omit JWT_SECRET in production!"
                );
                generate_ephemeral_dev_secret()
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

/// Configured JWT lifetime in hours (`JWT_EXPIRY_HOURS`, default 24, clamped 1..=168).
#[must_use]
pub fn jwt_expiry_hours() -> i64 {
    std::env::var("JWT_EXPIRY_HOURS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .unwrap_or(24)
        .clamp(1, 168)
}

/// Create a short-lived JWT access token for an authenticated admin user.
///
/// # Errors
/// Returns `AppError::Internal` if JWT encoding fails.
pub fn create_token(user_id: i64, email: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let issued_at = usize::try_from(now.timestamp().max(0)).unwrap_or(0);
    let expiration_ts = now
        .checked_add_signed(chrono::Duration::hours(jwt_expiry_hours()))
        .ok_or_else(|| AppError::Internal("Timestamp overflow".into()))?
        .timestamp();
    let expiration = usize::try_from(expiration_ts.max(0))
        .map_err(|_| AppError::Internal("Invalid expiration timestamp".into()))?;

    let claims = AdminClaims {
        sub: user_id,
        email: email.to_string(),
        iat: issued_at,
        exp: expiration,
        jti: uuid::Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))
}

/// Verify a JWT token and return the claims, explicitly pinning `Algorithm::HS256`.
///
/// # Errors
/// Returns `AppError::Unauthorized` if signature, algorithm, or expiration validation fails.
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
/// Also verifies that the token is not revoked and the user still exists in `admin_users`.
impl<S> FromRequestParts<S> for AdminClaims
where
    S: Send + Sync,
    sqlx::SqlitePool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let (token_str, claims, is_cookie_auth) = if let Some(auth_header) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                (token.to_string(), verify_token(token)?, false)
            } else {
                return Err(AppError::Unauthorized);
            }
        } else {
            let jar = CookieJar::from_headers(&parts.headers);
            if let Some(cookie) = jar.get(jwt_cookie_name()) {
                (
                    cookie.value().to_string(),
                    verify_token(cookie.value())?,
                    true,
                )
            } else {
                return Err(AppError::Unauthorized);
            }
        };

        // CSRF protection for cookie-authenticated mutating requests:
        // Require custom header `X-Requested-With: PMS-Client` for non-GET/HEAD/OPTIONS methods
        if is_cookie_auth
            && !matches!(
                parts.method,
                axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
            )
        {
            let req_with = parts
                .headers
                .get("x-requested-with")
                .and_then(|v| v.to_str().ok());
            if req_with != Some("PMS-Client") {
                return Err(AppError::Forbidden(
                    "CSRF protection: missing or invalid X-Requested-With header".into(),
                ));
            }
        }

        let pool = sqlx::SqlitePool::from_ref(state);

        let token_h = hash_token(&token_str);
        let is_revoked: Option<(String,)> =
            sqlx::query_as("SELECT token_hash FROM revoked_tokens WHERE token_hash = ?1")
                .bind(&token_h)
                .fetch_optional(&pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

        if is_revoked.is_some() {
            return Err(AppError::Unauthorized);
        }

        let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM admin_users WHERE id = ?1")
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if exists.is_none() {
            return Err(AppError::Unauthorized);
        }

        Ok(claims)
    }
}
