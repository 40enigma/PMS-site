//! In-memory rate-limiter backed by `governor`.
//!
//! Provides per-IP rate limiting for sensitive endpoints (login, registration,
//! contact, newsletter). Configuration is via environment variables:
//!
//! - `RATE_LIMIT_LOGIN_RPM`  — requests per minute for `/api/auth/login` (default 10)
//! - `RATE_LIMIT_SUBMIT_RPM` — requests per minute for submit endpoints (default 30)
//! - `TRUST_PROXY` — when set to "true" or "1", trust rightmost `X-Forwarded-For`
//!   or `X-Real-IP` set by a reverse proxy. Defaults to false.

use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    str::FromStr,
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    extract::{connect_info::ConnectInfo, FromRequestParts},
    http::request::Parts,
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
    pub register: Arc<KeyedLimiter>,
    pub contact: Arc<KeyedLimiter>,
    pub newsletter: Arc<KeyedLimiter>,
    pub membership: Arc<KeyedLimiter>,
    pub certificate_verify: Arc<KeyedLimiter>,
}

fn env_rpm(var: &str, default: u32) -> NonZeroU32 {
    let val = std::env::var(var)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(default);
    NonZeroU32::new(val.max(1)).unwrap()
}

/// Canonicalize an IP address for rate-limiting purposes.
/// For IPv4: keeps the 32-bit address.
/// For IPv6: masks to a /64 prefix to prevent /128 address rotation attacks and state exhaustion.
pub fn canonicalize_rate_limit_ip(ip: IpAddr) -> IpAddr {
    match ip.to_canonical() {
        IpAddr::V4(v4) => IpAddr::V4(v4),
        IpAddr::V6(v6) => {
            let seg = v6.segments();
            IpAddr::V6(std::net::Ipv6Addr::new(
                seg[0], seg[1], seg[2], seg[3], 0, 0, 0, 0,
            ))
        }
    }
}

impl RateLimitState {
    pub fn new() -> Self {
        let login_rpm = env_rpm("RATE_LIMIT_LOGIN_RPM", 10);
        let submit_rpm = env_rpm("RATE_LIMIT_SUBMIT_RPM", 30);
        let register_rpm = env_rpm("RATE_LIMIT_REGISTER_RPM", 60);
        let contact_rpm = env_rpm("RATE_LIMIT_CONTACT_RPM", 10);
        let newsletter_rpm = env_rpm("RATE_LIMIT_NEWSLETTER_RPM", 10);
        let membership_rpm = env_rpm("RATE_LIMIT_MEMBERSHIP_RPM", 15);
        let cert_verify_rpm = env_rpm("RATE_LIMIT_CERT_VERIFY_RPM", 60);

        tracing::info!(
            login_rpm = login_rpm.get(),
            submit_rpm = submit_rpm.get(),
            register_rpm = register_rpm.get(),
            contact_rpm = contact_rpm.get(),
            newsletter_rpm = newsletter_rpm.get(),
            membership_rpm = membership_rpm.get(),
            cert_verify_rpm = cert_verify_rpm.get(),
            "Rate limiter initialised"
        );

        let login_quota = Quota::per_minute(login_rpm);
        let submit_quota = Quota::per_minute(submit_rpm);
        let register_quota = Quota::per_minute(register_rpm);
        let contact_quota = Quota::per_minute(contact_rpm);
        let newsletter_quota = Quota::per_minute(newsletter_rpm);
        let membership_quota = Quota::per_minute(membership_rpm);
        let cert_verify_quota = Quota::per_minute(cert_verify_rpm);

        let state = Self {
            login: Arc::new(RateLimiter::keyed(login_quota)),
            submit: Arc::new(RateLimiter::keyed(submit_quota)),
            register: Arc::new(RateLimiter::keyed(register_quota)),
            contact: Arc::new(RateLimiter::keyed(contact_quota)),
            newsletter: Arc::new(RateLimiter::keyed(newsletter_quota)),
            membership: Arc::new(RateLimiter::keyed(membership_quota)),
            certificate_verify: Arc::new(RateLimiter::keyed(cert_verify_quota)),
        };

        // Periodic background task to prune stale entries and shrink DashMap,
        // preventing unbounded memory growth under unique IP load.
        let login_limiter = Arc::clone(&state.login);
        let submit_limiter = Arc::clone(&state.submit);
        let register_limiter = Arc::clone(&state.register);
        let contact_limiter = Arc::clone(&state.contact);
        let newsletter_limiter = Arc::clone(&state.newsletter);
        let membership_limiter = Arc::clone(&state.membership);
        let cert_limiter = Arc::clone(&state.certificate_verify);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                login_limiter.retain_recent();
                login_limiter.shrink_to_fit();
                submit_limiter.retain_recent();
                submit_limiter.shrink_to_fit();
                register_limiter.retain_recent();
                register_limiter.shrink_to_fit();
                contact_limiter.retain_recent();
                contact_limiter.shrink_to_fit();
                newsletter_limiter.retain_recent();
                newsletter_limiter.shrink_to_fit();
                membership_limiter.retain_recent();
                membership_limiter.shrink_to_fit();
                cert_limiter.retain_recent();
                cert_limiter.shrink_to_fit();
            }
        });

        state
    }

    /// Access the global shared instance.
    pub fn global() -> &'static Self {
        GLOBAL_RATE_LIMIT.get_or_init(Self::new)
    }

    /// Check login rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_login(&self, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        self.login
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check submit rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_submit(&self, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        self.submit
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check event registration rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_register(&self, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        self.register
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check contact message rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_contact(&self, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        self.contact
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check newsletter subscription rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_newsletter(&self, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        self.newsletter
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check membership application rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_membership(&self, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        self.membership
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check certificate verification rate limit for the given IP. Returns `Err(AppError::TooManyRequests)` if exceeded.
    pub fn check_certificate_verify(&self, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        self.certificate_verify
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if TRUST_PROXY is enabled.
pub fn is_trust_proxy() -> bool {
    std::env::var("TRUST_PROXY")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Check if TRUST_CLOUDFLARE is enabled.
pub fn is_trust_cloudflare() -> bool {
    std::env::var("TRUST_CLOUDFLARE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Extract client IP from HTTP headers ONLY if `TRUST_PROXY=true` is set in the environment.
///
/// When `TRUST_PROXY` is enabled:
/// 1. Reads `CF-Connecting-IP` or `True-Client-IP` ONLY if `TRUST_CLOUDFLARE=true`.
/// 2. Reads `X-Real-IP` (set by Nginx/Caddy) first.
/// 3. Reads the rightmost hop in `X-Forwarded-For`.
///
/// When `TRUST_PROXY` is NOT enabled, header values are strictly ignored to prevent IP spoofing.
pub fn client_ip_from_headers(headers: &axum::http::HeaderMap) -> IpAddr {
    if is_trust_proxy() {
        if is_trust_cloudflare() {
            if let Some(cf_ip) = headers
                .get("cf-connecting-ip")
                .or_else(|| headers.get("CF-Connecting-IP"))
                .or_else(|| headers.get("true-client-ip"))
                .or_else(|| headers.get("True-Client-IP"))
                .and_then(|v| v.to_str().ok())
            {
                if let Ok(ip) = IpAddr::from_str(cf_ip.trim()) {
                    return ip;
                }
            }
        }
        if let Some(xri) = headers.get("X-Real-IP").and_then(|v| v.to_str().ok()) {
            if let Ok(ip) = IpAddr::from_str(xri.trim()) {
                return ip;
            }
        }
        if let Some(xff) = headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()) {
            if let Some(rightmost) = xff.split(',').next_back() {
                if let Ok(ip) = IpAddr::from_str(rightmost.trim()) {
                    return ip;
                }
            }
        }
    }
    // Fall back to loopback address when headers are untrusted or absent
    "127.0.0.1".parse().unwrap()
}

/// Extract the client IP from request parts.
///
/// Priority:
/// 1. If `TRUST_PROXY=true`: parse rightmost IP from `X-Forwarded-For` or `X-Real-IP`.
/// 2. Extract TCP socket address from `ConnectInfo<SocketAddr>` extension.
/// 3. Fallback to `127.0.0.1` (e.g. for in-memory oneshot tests without ConnectInfo).
pub fn client_ip_from_request_parts(parts: &Parts) -> IpAddr {
    if is_trust_proxy() {
        let header_ip = client_ip_from_headers(&parts.headers);
        if header_ip != IpAddr::from([127, 0, 0, 1]) {
            return header_ip;
        }
    }

    if let Some(ConnectInfo(addr)) = parts.extensions.get::<ConnectInfo<SocketAddr>>() {
        return addr.ip();
    }

    "127.0.0.1".parse().unwrap()
}

/// Type-safe Axum extractor for resolving the client's verified IP address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientIp(pub IpAddr);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(ClientIp(client_ip_from_request_parts(parts)))
    }
}
