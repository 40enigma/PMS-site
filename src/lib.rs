pub mod auth;
pub mod db;
pub mod error;
pub mod models;
pub mod rate_limit;
pub mod routes;
pub mod util;

use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
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
                axum::http::HeaderName::from_static("x-requested-with"),
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

/// Security headers specifically applied to static uploads.
pub async fn uploads_security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let headers = res.headers_mut();
    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'; sandbox"),
    );
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=2592000"),
    );
    res
}

/// Middleware that adds modernized security headers and cache policies to responses.
pub async fn security_headers(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let is_production =
        std::env::var("RUST_ENV").unwrap_or_default().to_lowercase() == "production";

    let mut res = next.run(req).await;
    let headers = res.headers_mut();

    if is_production {
        headers.insert(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    if path.starts_with("/api/admin/") || path.starts_with("/api/auth/") {
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        );
    } else if method == Method::GET
        && (path == "/api/announcements" || path == "/api/stats" || path == "/api/sections")
    {
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=60"),
        );
    } else if path.starts_with("/assets") {
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        );
    }

    if path.starts_with("/uploads") {
        return res;
    }

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
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com data:; img-src 'self' data: https:; connect-src 'self'; frame-ancestors 'self';"
        ),
    );
    res
}

/// Convert JSON/body-limit rejections (413/422/400) into standard JSON error shape.
pub async fn standardize_api_errors(req: Request, next: Next) -> Response {
    let is_api = req.uri().path().starts_with("/api/");
    let res = next.run(req).await;
    if is_api {
        let status = res.status();
        let content_type = res
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if status == StatusCode::PAYLOAD_TOO_LARGE
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || (status == StatusCode::BAD_REQUEST && !content_type.contains("application/json"))
        {
            let message = if status == StatusCode::PAYLOAD_TOO_LARGE {
                "Payload too large".to_string()
            } else if status == StatusCode::UNPROCESSABLE_ENTITY {
                "Failed to deserialize JSON request body".to_string()
            } else {
                "Invalid request payload".to_string()
            };
            let body = serde_json::json!({
                "error": message,
                "status": status.as_u16(),
            });
            return (
                status,
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                axum::Json(body),
            )
                .into_response();
        }
    }
    res
}

static NOT_FOUND_HTML: &str = include_str!("../frontend/static/404.html");

/// Fallback handler for unmatched routes:
/// Returns JSON for `/api/*` routes and HTML 404 for frontend routes.
async fn not_found_fallback(req: Request) -> Response {
    if req.uri().path().starts_with("/api/") {
        (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            axum::Json(serde_json::json!({
                "error": "Resource not found",
                "status": 404
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            NOT_FOUND_HTML,
        )
            .into_response()
    }
}

/// Dedicated 404 route handler returning StatusCode::NOT_FOUND with 404.html.
async fn not_found_page() -> Response {
    (
        StatusCode::NOT_FOUND,
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        NOT_FOUND_HTML,
    )
        .into_response()
}

/// Health check endpoint returning 200 {"status":"ok"} on successful DB ping, 503 on failure.
#[axum::debug_handler]
pub async fn health_check(State(pool): State<SqlitePool>) -> Response {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            axum::Json(serde_json::json!({"status": "ok"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Health check DB ping failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                axum::Json(serde_json::json!({
                    "status": "error",
                    "message": "Database unavailable"
                })),
            )
                .into_response()
        }
    }
}

/// Dynamic sitemap.xml generator listing canonical static pages, published articles, and events.
#[axum::debug_handler]
pub async fn sitemap_xml(State(pool): State<SqlitePool>) -> Response {
    let base_url =
        std::env::var("APP_URL").unwrap_or_else(|_| "https://pms.pieas.edu.pk".to_string());
    let base_url = base_url.trim_end_matches('/');

    let static_routes = [
        "",
        "/about",
        "/team",
        "/events",
        "/resources",
        "/blog",
        "/opportunities",
        "/alumni",
        "/projects",
        "/gallery",
        "/sponsors",
        "/join",
    ];

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for route in static_routes {
        xml.push_str(&format!(
            "  <url>\n    <loc>{}{}</loc>\n    <changefreq>weekly</changefreq>\n    <priority>{}</priority>\n  </url>\n",
            base_url,
            route,
            if route.is_empty() { "1.0" } else { "0.8" }
        ));
    }

    if let Ok(posts) = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT slug, published_at FROM posts WHERE is_published = 1",
    )
    .fetch_all(&pool)
    .await
    {
        for (slug, pub_date) in posts {
            let lastmod = pub_date
                .map(|d| format!("    <lastmod>{}</lastmod>\n", d))
                .unwrap_or_default();
            xml.push_str(&format!(
                "  <url>\n    <loc>{}/blog#{}</loc>\n{}    <changefreq>monthly</changefreq>\n    <priority>0.7</priority>\n  </url>\n",
                base_url, slug, lastmod
            ));
        }
    }

    if let Ok(events) = sqlx::query_as::<_, (String, String)>("SELECT slug, date FROM events")
        .fetch_all(&pool)
        .await
    {
        for (slug, date) in events {
            xml.push_str(&format!(
                "  <url>\n    <loc>{}/events#{}</loc>\n    <lastmod>{}</lastmod>\n    <changefreq>monthly</changefreq>\n    <priority>0.6</priority>\n  </url>\n",
                base_url, slug, date
            ));
        }
    }

    xml.push_str("</urlset>");

    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "application/xml; charset=utf-8",
        )],
        xml,
    )
        .into_response()
}

/// Construct the complete Axum application router.
pub fn create_app(pool: SqlitePool) -> Router {
    let cors = build_cors();

    let uploads_router = Router::new()
        .fallback_service(ServeDir::new(upload_dir()))
        .layer(middleware::from_fn(uploads_security_headers));

    let mut router = Router::new()
        .route("/healthz", get(health_check))
        .route("/sitemap.xml", get(sitemap_xml))
        .route_service("/robots.txt", ServeFile::new("frontend/static/robots.txt"))
        .merge(routes::public::public_routes())
        .merge(routes::admin::admin_routes())
        // uploads served as raw files wrapped in custom response headers
        .nest("/uploads", uploads_router)
        // Static assets served under /assets
        .nest_service("/assets", ServeDir::new("frontend/static/assets"))
        // Pretty URLs and static files (with tolerance for trailing slashes)
        .route_service("/", ServeFile::new("frontend/static/index.html"));

    let pretty_pages = [
        ("/about", "frontend/static/about.html"),
        ("/team", "frontend/static/team.html"),
        ("/events", "frontend/static/events.html"),
        ("/resources", "frontend/static/resources.html"),
        ("/blog", "frontend/static/blog.html"),
        ("/opportunities", "frontend/static/opportunities.html"),
        ("/alumni", "frontend/static/alumni.html"),
        ("/projects", "frontend/static/projects.html"),
        ("/gallery", "frontend/static/gallery.html"),
        ("/sponsors", "frontend/static/sponsors.html"),
        ("/join", "frontend/static/join.html"),
        ("/admin", "frontend/static/admin.html"),
    ];

    for (route, file_path) in pretty_pages {
        router = router
            .route_service(route, ServeFile::new(file_path))
            .route_service(&format!("{}/", route), ServeFile::new(file_path));
    }

    router
        .route("/404", get(not_found_page))
        .route("/404/", get(not_found_page))
        .route_service(
            "/favicon.svg",
            ServeFile::new("frontend/static/favicon.svg"),
        )
        .route_service(
            "/favicon.ico",
            ServeFile::new("frontend/static/favicon.svg"),
        )
        .route_service(
            "/pms-logo.jpeg",
            ServeFile::new("frontend/static/pms-logo.jpeg"),
        )
        // Any unknown route serves fallback (JSON on /api/*, 404.html elsewhere)
        .fallback(not_found_fallback)
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(CompressionLayer::new())
        .layer(axum::Extension(pool.clone()))
        .layer(middleware::from_fn(standardize_api_errors))
        .layer(middleware::from_fn(security_headers))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
