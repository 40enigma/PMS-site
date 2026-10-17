//! In-memory rate-limiter backed by `governor`.
//!
//! Provides per-IP rate limiting for sensitive endpoints (login, registration,
//! contact, newsletter).  Configuration is via environment variables:
//!
//! - `RATE_LIMIT_LOGIN_RPM`   — requests per minute for `/api/auth/login` (default 10)
//! - `RATE_LIMIT_SUBMIT_RPM`  — requests per minute for submit endpoints (default 30)

use std::{
    net::IpAddr,
    num::NonZeroU32,
    str::FromStr,
    sync::{Arc, OnceLock},
    time::Duration,
};

use governor::{
    clock::DefaultClock, middleware::NoOpMiddleware, state::keyed::DashMapStateStore, Quota,
    RateLimiter,
};

use crate::error::AppError;

type KeyedLimiter = RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;

static GLOBAL_RATE_LIMIT: OnceLock<RateLimitState> = OnceLock::new();

/// Shared rate-limit state passed through Axum state or accessed globally.
pub struct RateLimitState {
    pub login: Arc<KeyedLimiter>,
    pub submit: Arc<KeyedLimiter>,
}

fn env_rpm(var: &str, default: u32) -> NonZeroU32 {
    let val = std::env::var(var)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(default);
    NonZeroU32::new(val.max(1)).unwrap()
}

impl RateLimitState {
    pub fn new() -> Self {
        let login_rpm = env_rpm("RATE_LIMIT_LOGIN_RPM", 10);
        let submit_rpm = env_rpm("RATE_LIMIT_SUBMIT_RPM", 30);

        tracing::info!(
            login_rpm = login_rpm.get(),
            submit_rpm = submit_rpm.get(),
            "Rate limiter initialised"
        );

        let login_quota = Quota::with_period(Duration::from_secs(60 / login_rpm.get() as u64))
            .unwrap()
            .allow_burst(login_rpm);

        let submit_quota = Quota::with_period(Duration::from_secs(60 / submit_rpm.get() as u64))
            .unwrap()
            .allow_burst(submit_rpm);

        Self {
            login: Arc::new(RateLimiter::keyed(login_quota)),
            submit: Arc::new(RateLimiter::keyed(submit_quota)),
        }
    }

    /// Access the global shared instance.
    pub fn global() -> &'static Self {
        GLOBAL_RATE_LIMIT.get_or_init(Self::new)
    }

    /// Check login rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_login(&self, ip: IpAddr) -> Result<(), AppError> {
        self.login
            .check_key(&ip)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check submit rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_submit(&self, ip: IpAddr) -> Result<(), AppError> {
        self.submit
            .check_key(&ip)
            .map_err(|_| AppError::TooManyRequests)
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the real client IP from HTTP headers, checking `X-Forwarded-For` and `X-Real-IP`.
pub fn client_ip_from_headers(headers: &axum::http::HeaderMap) -> IpAddr {
    if let Some(xff) = headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            if let Ok(ip) = IpAddr::from_str(first.trim()) {
                return ip;
            }
        }
    }
    if let Some(xri) = headers.get("X-Real-IP").and_then(|v| v.to_str().ok()) {
        if let Ok(ip) = IpAddr::from_str(xri.trim()) {
            return ip;
        }
    }
    // Fall back to loopback address for local testing / direct connections
    "127.0.0.1".parse().unwrap()
}

/// Extract the real client IP from a request Parts.
#[allow(dead_code)]
pub fn client_ip(parts: &axum::http::request::Parts) -> IpAddr {
    client_ip_from_headers(&parts.headers)
}
