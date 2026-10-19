pub mod auth;
pub mod db;
pub mod error;
pub mod models;
pub mod rate_limit;
pub mod routes;

use axum::{
    extract::{DefaultBodyLimit, Request},
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use sqlx::SqlitePool;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

/// Helper to read the upload directory from UPLOAD_DIR env var (defaulting to "uploads").
pub fn upload_dir() -> String {
    std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string())
}


/// Build the CORS layer from ALLOWED_ORIGINS env var.
///
/// - In production (`RUST_ENV=production`): ALLOWED_ORIGINS is required.
///   Server exits if it is missing or empty.
/// - In development: falls back to allowing localhost on common ports.
pub fn build_cors() -> CorsLayer {
    let is_production =
        std::env::var("RUST_ENV").unwrap_or_default().to_lowercase() == "production";

    let origins_raw = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
    let origins_raw = origins_raw.trim().to_string();

    if origins_raw.is_empty() {
        if is_production {
            eprintln!(
                "\n\
                ╔══════════════════════════════════════════════════════════╗\n\
                ║  FATAL: ALLOWED_ORIGINS must be set in production mode.   ║\n\
                ║  Example: ALLOWED_ORIGINS=https://pms.pieas.edu.pk        ║\n\
                ╚══════════════════════════════════════════════════════════╝\n"
            );
            std::process::exit(1);
        }

        // Development fallback: allow common localhost ports.
        tracing::warn!("⚠️  ALLOWED_ORIGINS not set. Allowing localhost in development mode.");
        return CorsLayer::new()
            .allow_origin([
                "http://localhost:3000".parse::<HeaderValue>().unwrap(),
                "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
            ])
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
                axum::http::header::ACCEPT,
                axum::http::header::COOKIE,
                axum::http::HeaderName::from_static("x-correlation-id"),
            ])
            .allow_credentials(true);
    }

    let allowed: Vec<HeaderValue> = origins_raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    if allowed.is_empty() {
        eprintln!("FATAL: ALLOWED_ORIGINS contains no valid origins.");
        std::process::exit(1);
    }

    tracing::info!("CORS allowed origins: {:?}", allowed);

    CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
            axum::http::HeaderName::from_static("x-correlation-id"),
            axum::http::HeaderName::from_static("x-requested-with"),
        ])
        .allow_credentials(true)
}

/// Middleware that adds modernized security headers to every response.
pub async fn security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let headers = res.headers_mut();
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("SAMEORIGIN"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com data:; img-src 'self' data: https:; connect-src 'self'; frame-ancestors 'self';"
        ),
    );
    res
}

/// Fallback handler for unmatched routes: returns HTTP 404 with frontend/static/404.html.
async fn not_found_fallback() -> Response {
    let not_found_html = tokio::fs::read_to_string("frontend/static/404.html")
        .await
        .unwrap_or_else(|_| {
            "<!DOCTYPE html><html><head><title>404 Not Found</title></head><body><h1>404 Not Found</h1></body></html>".to_string()
        });

    (
        StatusCode::NOT_FOUND,
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        not_found_html,
    )
        .into_response()
}

/// Construct the complete Axum application router.
pub fn create_app(pool: SqlitePool) -> Router {
    let cors = build_cors();

    Router::new()
        .merge(routes::public::public_routes())
        .merge(routes::admin::admin_routes())
        // uploads served as raw files (no execution)
        .nest_service("/uploads", ServeDir::new(upload_dir()))
        // Static assets served under /assets
        .nest_service("/assets", ServeDir::new("frontend/static/assets"))
        // Explicit static page routes for pretty URLs
        .route_service("/", ServeFile::new("frontend/static/index.html"))
        .route_service("/about", ServeFile::new("frontend/static/about.html"))
        .route_service("/team", ServeFile::new("frontend/static/team.html"))
        .route_service("/events", ServeFile::new("frontend/static/events.html"))
        .route_service(
            "/resources",
            ServeFile::new("frontend/static/resources.html"),
        )
        .route_service("/blog", ServeFile::new("frontend/static/blog.html"))
        .route_service(
            "/opportunities",
            ServeFile::new("frontend/static/opportunities.html"),
        )
        .route_service("/alumni", ServeFile::new("frontend/static/alumni.html"))
        .route_service("/projects", ServeFile::new("frontend/static/projects.html"))
        .route_service("/gallery", ServeFile::new("frontend/static/gallery.html"))
        .route_service("/sponsors", ServeFile::new("frontend/static/sponsors.html"))
        .route_service("/join", ServeFile::new("frontend/static/join.html"))
        .route_service("/admin", ServeFile::new("frontend/static/admin.html"))
        .route_service("/404", ServeFile::new("frontend/static/404.html"))
        .route_service("/favicon.svg", ServeFile::new("frontend/static/favicon.svg"))
        .route_service("/favicon.ico", ServeFile::new("frontend/static/favicon.svg"))
        .route_service("/pms-logo.jpeg", ServeFile::new("frontend/static/pms-logo.jpeg"))
        // Any unknown route serves 404.html with StatusCode::NOT_FOUND (404)
        .fallback(not_found_fallback)
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(CompressionLayer::new())
        .layer(axum::Extension(pool.clone()))
        .layer(middleware::from_fn(security_headers))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
