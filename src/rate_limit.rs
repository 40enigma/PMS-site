//! In-memory rate-limiter backed by `governor`.
//!
//! Provides per-IP rate limiting for sensitive endpoints (login, logout,
//! registration, contact, newsletter, membership, certificate verification).
//! Configuration is via environment variables:
//!
//! - `RATE_LIMIT_LOGIN_RPM`       — requests per minute for `/api/auth/login` (default 10)
//! - `RATE_LIMIT_SUBMIT_RPM`      — requests per minute for `/api/auth/logout` (default 30)
//! - `RATE_LIMIT_REGISTER_RPM`    — requests per minute for event registration (default 60)
//! - `RATE_LIMIT_CONTACT_RPM`     — requests per minute for `/api/contact` (default 10)
//! - `RATE_LIMIT_NEWSLETTER_RPM`  — requests per minute for `/api/newsletter` (default 10)
//! - `RATE_LIMIT_MEMBERSHIP_RPM`  — requests per minute for `/api/membership` (default 15)
//! - `RATE_LIMIT_CERT_VERIFY_RPM` — requests per minute for `/api/certificates/verify` (default 60)
//! - `TRUST_PROXY`                — trust proxy headers when peer is a trusted proxy (default false)
//! - `TRUST_CLOUDFLARE`           — trust `CF-Connecting-IP` / `True-Client-IP` when `TRUST_PROXY=true` (default false)
//! - `TRUST_PROXY_HEADER`         — `auto` (default), `x-forwarded-for`, `x-real-ip`, or `cf-connecting-ip`

use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    str::FromStr,
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    extract::{connect_info::ConnectInfo, FromRequestParts},
    http::{request::Parts, HeaderMap},
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
#[must_use]
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
    #[must_use]
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

        let state = Self {
            login: Arc::new(RateLimiter::keyed(Quota::per_minute(login_rpm))),
            submit: Arc::new(RateLimiter::keyed(Quota::per_minute(submit_rpm))),
            register: Arc::new(RateLimiter::keyed(Quota::per_minute(register_rpm))),
            contact: Arc::new(RateLimiter::keyed(Quota::per_minute(contact_rpm))),
            newsletter: Arc::new(RateLimiter::keyed(Quota::per_minute(newsletter_rpm))),
            membership: Arc::new(RateLimiter::keyed(Quota::per_minute(membership_rpm))),
            certificate_verify: Arc::new(RateLimiter::keyed(Quota::per_minute(cert_verify_rpm))),
        };

        // Periodic background task to prune stale entries and shrink DashMap,
        // preventing unbounded memory growth under unique IP load.
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let login_limiter = Arc::clone(&state.login);
            let submit_limiter = Arc::clone(&state.submit);
            let register_limiter = Arc::clone(&state.register);
            let contact_limiter = Arc::clone(&state.contact);
            let newsletter_limiter = Arc::clone(&state.newsletter);
            let membership_limiter = Arc::clone(&state.membership);
            let cert_limiter = Arc::clone(&state.certificate_verify);
            handle.spawn(async move {
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
        }

        state
    }

    /// Access the global shared instance.
    pub fn global() -> &'static Self {
        GLOBAL_RATE_LIMIT.get_or_init(Self::new)
    }

    fn check_limiter(limiter: &KeyedLimiter, ip: IpAddr) -> Result<(), AppError> {
        let key = canonicalize_rate_limit_ip(ip);
        limiter
            .check_key(&key)
            .map_err(|_| AppError::TooManyRequests)
    }

    /// Check login rate limit for the given IP.
    ///
    /// # Errors
    /// Returns `AppError::TooManyRequests` if the per-minute quota is exceeded.
    pub fn check_login(&self, ip: IpAddr) -> Result<(), AppError> {
        Self::check_limiter(&self.login, ip)
    }

    /// Check logout/submit rate limit for the given IP.
    ///
    /// # Errors
    /// Returns `AppError::TooManyRequests` if the per-minute quota is exceeded.
    pub fn check_submit(&self, ip: IpAddr) -> Result<(), AppError> {
        Self::check_limiter(&self.submit, ip)
    }

    /// Check event registration rate limit for the given IP.
    ///
    /// # Errors
    /// Returns `AppError::TooManyRequests` if the per-minute quota is exceeded.
    pub fn check_register(&self, ip: IpAddr) -> Result<(), AppError> {
        Self::check_limiter(&self.register, ip)
    }

    /// Check contact message rate limit for the given IP.
    ///
    /// # Errors
    /// Returns `AppError::TooManyRequests` if the per-minute quota is exceeded.
    pub fn check_contact(&self, ip: IpAddr) -> Result<(), AppError> {
        Self::check_limiter(&self.contact, ip)
    }

    /// Check newsletter subscription rate limit for the given IP.
    ///
    /// # Errors
    /// Returns `AppError::TooManyRequests` if the per-minute quota is exceeded.
    pub fn check_newsletter(&self, ip: IpAddr) -> Result<(), AppError> {
        Self::check_limiter(&self.newsletter, ip)
    }

    /// Check membership application rate limit for the given IP.
    ///
    /// # Errors
    /// Returns `AppError::TooManyRequests` if the per-minute quota is exceeded.
    pub fn check_membership(&self, ip: IpAddr) -> Result<(), AppError> {
        Self::check_limiter(&self.membership, ip)
    }

    /// Check certificate verification rate limit for the given IP.
    ///
    /// # Errors
    /// Returns `AppError::TooManyRequests` if the per-minute quota is exceeded.
    pub fn check_certificate_verify(&self, ip: IpAddr) -> Result<(), AppError> {
        Self::check_limiter(&self.certificate_verify, ip)
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if `TRUST_PROXY` is enabled.
#[must_use]
pub fn is_trust_proxy() -> bool {
    std::env::var("TRUST_PROXY")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Check if `TRUST_CLOUDFLARE` is enabled.
#[must_use]
pub fn is_trust_cloudflare() -> bool {
    std::env::var("TRUST_CLOUDFLARE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Check whether a TCP socket peer IP is a trusted local/private reverse-proxy hop.
#[must_use]
pub fn is_trusted_proxy_peer(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 ULA
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
        }
    }
}

fn parse_cf_header(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("true-client-ip"))
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| IpAddr::from_str(raw.trim()).ok())
}

fn parse_x_real_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| IpAddr::from_str(raw.trim()).ok())
}

fn parse_rightmost_xff(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|xff| xff.split(',').next_back())
        .and_then(|rightmost| IpAddr::from_str(rightmost.trim()).ok())
}

fn extract_proxy_ip_from_headers(headers: &HeaderMap) -> Option<IpAddr> {
    let mode = std::env::var("TRUST_PROXY_HEADER")
        .unwrap_or_else(|_| "auto".to_string())
        .trim()
        .to_ascii_lowercase();

    match mode.as_str() {
        "x-forwarded-for" => parse_rightmost_xff(headers),
        "x-real-ip" => parse_x_real_ip(headers),
        "cf-connecting-ip" => {
            if is_trust_cloudflare() {
                parse_cf_header(headers)
            } else {
                None
            }
        }
        _ => {
            if is_trust_cloudflare() {
                if let Some(ip) = parse_cf_header(headers) {
                    return Some(ip);
                }
            }
            if let Some(ip) = parse_x_real_ip(headers) {
                return Some(ip);
            }
            parse_rightmost_xff(headers)
        }
    }
}

/// Extract client IP from HTTP headers ONLY if `TRUST_PROXY=true` is set in the environment.
///
/// Falls back to `127.0.0.1` when headers are untrusted or absent.
#[must_use]
pub fn client_ip_from_headers(headers: &HeaderMap) -> IpAddr {
    if is_trust_proxy() {
        if let Some(ip) = extract_proxy_ip_from_headers(headers) {
            return ip;
        }
    }
    IpAddr::from([127, 0, 0, 1])
}

/// Extract the client IP from request parts.
///
/// Priority:
/// 1. If `TRUST_PROXY=true` AND (`ConnectInfo` is absent OR TCP peer is a trusted private/loopback
///    proxy or `TRUST_CLOUDFLARE=true`): parse configured proxy header.
/// 2. Extract TCP socket address from `ConnectInfo<SocketAddr>` extension.
/// 3. Fallback to `127.0.0.1` (e.g. for in-memory oneshot tests without `ConnectInfo`).
#[must_use]
pub fn client_ip_from_request_parts(parts: &Parts) -> IpAddr {
    let peer_ip = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip());

    if is_trust_proxy() {
        let peer_trusted =
            peer_ip.is_none_or(|ip| is_trust_cloudflare() || is_trusted_proxy_peer(ip));
        if peer_trusted {
            if let Some(header_ip) = extract_proxy_ip_from_headers(&parts.headers) {
                return header_ip;
            }
        }
    }

    if let Some(ip) = peer_ip {
        return ip;
    }

    IpAddr::from([127, 0, 0, 1])
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
