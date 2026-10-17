pub mod auth;
pub mod db;
pub mod error;
pub mod models;
pub mod rate_limit;
pub mod routes;

use axum::{
    extract::Request,
    http::{HeaderValue, Method},
    middleware::{self, Next},
    response::Response,
    Router,
};
use sqlx::SqlitePool;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(info(
    title = "PIEAS Materials Society (PMS) API",
    version = "1.0.0",
    description = "Official REST API for PIEAS Materials Society (DMME, PIEAS Islamabad)."
))]
pub struct ApiDoc;

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
                axum::http::HeaderName::from_static("x-forwarded-for"),
                axum::http::HeaderName::from_static("x-real-ip"),
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
            axum::http::HeaderName::from_static("x-forwarded-for"),
            axum::http::HeaderName::from_static("x-real-ip"),
            axum::http::HeaderName::from_static("x-correlation-id"),
        ])
        .allow_credentials(true)
}

/// Middleware that adds security headers to every response.
pub async fn security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let headers = res.headers_mut();
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("SAMEORIGIN"));
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    res
}

/// Construct the complete Axum application router.
pub fn create_app(pool: SqlitePool) -> Router {
    let cors = build_cors();

    // Static file service — serves frontend/static for non-API routes.
    // Falls back to index.html for unknown paths (pretty-URL support).
    let static_service =
        ServeDir::new("frontend/static").fallback(ServeFile::new("frontend/static/index.html"));

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(routes::public::public_routes())
        .merge(routes::admin::admin_routes())
        // uploads served as raw files (no execution)
        .nest_service("/uploads", ServeDir::new("uploads"))
        // Explicit static page routes for pretty URLs (e.g. /about → about.html)
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
        // Everything else → static directory (serves assets, or index.html fallback)
        .fallback_service(static_service)
        .layer(middleware::from_fn(security_headers))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
