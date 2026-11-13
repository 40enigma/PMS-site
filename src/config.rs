//! Centralised environment and configuration helpers.
//!
//! Every "is this production?" decision goes through [`current`] so a typo in
//! `RUST_ENV` (for example `"prod"` or `"Production "`) cannot silently select the
//! development defaults. Values are read on each call because the integration tests
//! mutate the process environment at runtime; each lookup is a cheap `std::env` read
//! rather than a hidden global that cannot be reset.

use std::time::Duration;

/// Deployment profile derived from `RUST_ENV`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Production,
    Development,
}

impl Env {
    pub fn is_production(self) -> bool {
        matches!(self, Env::Production)
    }
}

/// Resolve `RUST_ENV`, tolerating case and surrounding whitespace.
///
/// Anything that is not explicitly `production`/`prod` is treated as development;
/// [`validate_startup`] rejects values that are neither, so a typo cannot pass
/// silently in a deployed environment.
pub fn current() -> Env {
    match std::env::var("RUST_ENV")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "production" | "prod" => Env::Production,
        _ => Env::Development,
    }
}

pub fn is_production() -> bool {
    current().is_production()
}

/// Parse a boolean environment flag: `1/true/yes/on` and `0/false/no/off`.
pub fn flag(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

/// Public base URL used for canonical links, Open Graph tags and the sitemap.
pub fn app_url() -> String {
    std::env::var("APP_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
            format!("http://localhost:{port}")
        })
}

/// CORS allow-list, trimmed with empty entries removed.
pub fn allowed_origins() -> Vec<String> {
    std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|origin| origin.trim().to_string())
        .filter(|origin| !origin.is_empty())
        .collect()
}

/// A CORS origin must be an explicit scheme plus host, never a wildcard.
pub fn is_valid_origin(origin: &str) -> bool {
    !origin.contains('*')
        && !origin.contains(' ')
        && (origin.starts_with("http://") || origin.starts_with("https://"))
        && origin.len() > "https://".len()
}

/// `Secure` flag for the admin cookie; defaults to on in production.
pub fn cookie_secure() -> bool {
    flag("JWT_COOKIE_SECURE", is_production())
}

/// Admin JWT lifetime in seconds, also used for the cookie `Max-Age`.
pub fn jwt_ttl_seconds() -> i64 {
    std::env::var("JWT_TTL_SECONDS")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .filter(|seconds| (300..=604_800).contains(seconds))
        .unwrap_or(86_400)
}

/// Upload cap in bytes.
pub fn max_upload_bytes() -> usize {
    let megabytes = std::env::var("MAX_UPLOAD_MB")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|megabytes| (1..=200).contains(megabytes))
        .unwrap_or(20);
    megabytes * 1024 * 1024
}

/// SQLite pool size.
pub fn db_max_connections() -> u32 {
    std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|connections| (1..=64).contains(connections))
        .unwrap_or(5)
}

/// How long SQLite waits for a write lock.
pub fn db_busy_timeout() -> Duration {
    let millis = std::env::var("DB_BUSY_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|millis| (100..=60_000).contains(millis))
        .unwrap_or(5_000);
    Duration::from_millis(millis)
}

/// Fail-fast validation executed by `main` before any I/O happens.
///
/// Returns a message describing the first problem found; `main` prints it and
/// exits non-zero so a misconfigured deployment never reaches the listener.
pub fn validate_startup() -> Result<(), String> {
    let raw = std::env::var("RUST_ENV").unwrap_or_default();
    let normalised = raw.trim().to_ascii_lowercase();
    if !normalised.is_empty()
        && !matches!(
            normalised.as_str(),
            "production" | "prod" | "development" | "dev"
        )
    {
        return Err(format!(
            "RUST_ENV must be one of production, development (got {raw:?})"
        ));
    }

    if !is_production() {
        return Ok(());
    }

    let app_url_missing = std::env::var("APP_URL")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    if app_url_missing {
        return Err(
            "APP_URL must be set in production (for example https://pms.pieas.edu.pk)".to_string(),
        );
    }

    let origins = allowed_origins();
    if origins.is_empty() {
        return Err("ALLOWED_ORIGINS must be set in production".to_string());
    }
    if let Some(invalid) = origins.iter().find(|origin| !is_valid_origin(origin)) {
        return Err(format!(
            "ALLOWED_ORIGINS contains an invalid origin: {invalid:?}"
        ));
    }

    Ok(())
}
