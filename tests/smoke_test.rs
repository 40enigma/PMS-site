use argon2::PasswordHasher;
use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use pms_platform::{auth::create_token, create_app, db};
use serial_test::serial;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn setup_test_app_and_pool() -> (axum::Router, sqlx::SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory test db");

    db::run_migrations(&pool).await;

    // Insert dummy stats row (id=1)
    sqlx::query(
        "INSERT INTO site_stats (id, active_members, events_held, years_active, industrial_partners) VALUES (1, 150, 30, 8, 12) ON CONFLICT(id) DO UPDATE SET active_members = 150, events_held = 30, years_active = 8, industrial_partners = 12"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert site stats");

    // Insert dummy admin (id=1) so AdminClaims verification against admin_users succeeds
    sqlx::query(
        "INSERT OR IGNORE INTO admin_users (id, email, password_hash, name) VALUES (1, 'admin@pieas.edu.pk', 'dummyhash', 'Admin')"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert dummy admin");

    (create_app(pool.clone()), pool)
}

async fn setup_test_app() -> axum::Router {
    setup_test_app_and_pool().await.0
}

#[tokio::test]
async fn test_get_stats_returns_200_and_json() {
    let app = setup_test_app().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stats")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["active_members"], 150);
    assert_eq!(json["events_held"], 30);
}

#[tokio::test]
async fn test_static_pages_served() {
    let app = setup_test_app().await;

    // Test GET / returns index.html
    let req = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("<!DOCTYPE html>") || html.contains("<html"));

    // Test GET /about returns about.html
    let app2 = setup_test_app().await;
    let req2 = Request::builder()
        .method(Method::GET)
        .uri("/about")
        .body(Body::empty())
        .unwrap();

    let res2 = app2.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_sitemap_and_robots_routes() {
    let app = setup_test_app().await;

    // Test GET /robots.txt
    let req = Request::builder()
        .method(Method::GET)
        .uri("/robots.txt")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("Disallow: /admin"));

    // Test GET /sitemap.xml
    let req2 = Request::builder()
        .method(Method::GET)
        .uri("/sitemap.xml")
        .body(Body::empty())
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    assert_eq!(
        res2.headers()
            .get(axum::http::header::CONTENT_TYPE)
            .unwrap(),
        "application/xml; charset=utf-8"
    );
    let body2 = res2.into_body().collect().await.unwrap().to_bytes();
    let xml = String::from_utf8_lossy(&body2);
    assert!(xml.contains("<urlset"));
    assert!(xml.contains("/about</loc>"));
}

#[tokio::test]
#[serial]
async fn test_login_invalid_credentials_returns_401_and_rate_limits() {
    let app = setup_test_app().await;

    let socket: std::net::SocketAddr = "198.51.100.42:12345".parse().unwrap();

    let mut got_429 = false;
    // Attempt multiple invalid logins until rate-limited (max 15 attempts, default quota is 10)
    for _ in 0..15 {
        let app_clone = app.clone();
        let payload = serde_json::json!({
            "email": "invalid_user@example.com",
            "password": "wrong_password_123"
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .extension(axum::extract::ConnectInfo(socket))
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let res = app_clone.oneshot(req).await.unwrap();
        if res.status() == StatusCode::TOO_MANY_REQUESTS {
            got_429 = true;
            assert!(
                res.headers().contains_key("retry-after"),
                "429 response must contain Retry-After header"
            );
            break;
        } else {
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }
    }

    assert!(
        got_429,
        "Expected 429 Too Many Requests after exceeding login rate limit"
    );
}

#[tokio::test]
async fn test_upload_file_allowed_and_forbidden() {
    let app = setup_test_app().await;

    // Generate valid admin token
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. Forbidden file: shell script disguised as .sh
    let boundary = "------------------------boundary123";
    let body_sh = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"malicious.sh\"\r\nContent-Type: application/x-sh\r\n\r\n#!/bin/bash\necho 'pwned'\r\n--{boundary}--\r\n",
        boundary = boundary
    );

    let req_forbidden = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body_sh))
        .unwrap();

    let res_forbidden = app.clone().oneshot(req_forbidden).await.unwrap();
    assert_eq!(res_forbidden.status(), StatusCode::BAD_REQUEST);

    // 2. Allowed file: genuine PNG image with PNG magic bytes (\x89PNG\r\n\x1a\n...)
    let png_header = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4\x00\x00\x00\nIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\r\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82";

    let mut multipart_data = Vec::new();
    multipart_data.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"valid.png\"\r\nContent-Type: image/png\r\n\r\n",
            boundary = boundary
        )
        .as_bytes(),
    );
    multipart_data.extend_from_slice(png_header);
    multipart_data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req_allowed = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(multipart_data))
        .unwrap();

    let res_allowed = app.oneshot(req_allowed).await.unwrap();
    assert_eq!(res_allowed.status(), StatusCode::OK);

    let body = res_allowed.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let url_str = json["url"].as_str().unwrap();
    assert!(url_str.starts_with("/uploads/"));

    // Clean up temporary test file from uploads directory
    let file_path = url_str.trim_start_matches('/');
    let _ = tokio::fs::remove_file(file_path).await;
}

#[tokio::test]
#[serial]
async fn test_seed_admin_not_created_without_flag() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    db::run_migrations(&pool).await;

    // Run seed without SEED_ADMIN
    std::env::remove_var("SEED_ADMIN");
    std::env::remove_var("ADMIN_INITIAL_EMAIL");
    std::env::remove_var("ADMIN_INITIAL_PASSWORD");

    db::seed_data(&pool).await;

    let admin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admin_users")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(
        admin_count.0, 0,
        "No admin user should be seeded without explicit SEED_ADMIN=true"
    );
}

#[tokio::test]
async fn test_input_validation_rejections() {
    let app = setup_test_app().await;

    // 1. Empty name in event registration
    let bad_reg = serde_json::json!({
        "student_name": "",
        "email": "student@example.com",
        "roll_number": "123",
        "batch": "2024",
        "department": "DMME"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/events/1/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&bad_reg).unwrap()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 2. Invalid email in contact
    let bad_contact = serde_json::json!({
        "name": "Test User",
        "email": "not-an-email",
        "message": "Hello"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/contact")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&bad_contact).unwrap()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 3. Empty email in newsletter
    let bad_sub = serde_json::json!({
        "email": "   "
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/newsletter")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&bad_sub).unwrap()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_404_fallback_returns_not_found_status_and_page() {
    let app = setup_test_app().await;

    // Test nonexistent route returns 404 status code (not 200) and contains 404 page content
    let req = Request::builder()
        .method(Method::GET)
        .uri("/nonexistent-page-path-12345")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("404") || html.contains("Page Not Found"),
        "Expected 404 error page content in fallback response"
    );
}

#[tokio::test]
async fn test_admin_put_and_delete_nonexistent_returns_404() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. PUT nonexistent announcement id 999999 -> 404
    let update_payload = serde_json::json!({
        "message": "Nonexistent announcement",
        "priority": "normal",
        "is_active": false
    });

    let req_put = Request::builder()
        .method(Method::PUT)
        .uri("/api/admin/announcements/999999")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
        .unwrap();

    let res_put = app.clone().oneshot(req_put).await.unwrap();
    assert_eq!(res_put.status(), StatusCode::NOT_FOUND);

    // 2. DELETE nonexistent announcement id 999999 -> 404
    let req_del = Request::builder()
        .method(Method::DELETE)
        .uri("/api/admin/announcements/999999")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res_del = app.oneshot(req_del).await.unwrap();
    assert_eq!(res_del.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_crud_workflow() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. Create FAQ via POST /api/admin/faqs
    let create_payload = serde_json::json!({
        "question": "What is PIEAS Materials Society?",
        "answer": "PMS is the official student chapter of Materials Engineering at PIEAS.",
        "display_order": 1
    });

    let req_post = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/faqs")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let res_post = app.clone().oneshot(req_post).await.unwrap();
    assert_eq!(res_post.status(), StatusCode::OK);

    // 2. Read back via GET /api/faqs
    let req_get = Request::builder()
        .method(Method::GET)
        .uri("/api/faqs")
        .body(Body::empty())
        .unwrap();

    let res_get = app.clone().oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);
    let body = res_get.into_body().collect().await.unwrap().to_bytes();
    let faqs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let created = faqs
        .iter()
        .find(|f| f["question"] == "What is PIEAS Materials Society?")
        .expect("Newly created FAQ should exist in list");
    let faq_id = created["id"].as_i64().unwrap();

    // 3. Update via PUT /api/admin/faqs/{id}
    let update_payload = serde_json::json!({
        "question": "What is PMS?",
        "answer": "Updated Answer for PIEAS Materials Society.",
        "display_order": 2
    });

    let req_put = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/admin/faqs/{}", faq_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
        .unwrap();

    let res_put = app.clone().oneshot(req_put).await.unwrap();
    assert_eq!(res_put.status(), StatusCode::OK);

    // 4. Delete via DELETE /api/admin/faqs/{id}
    let req_del = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/admin/faqs/{}", faq_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res_del = app.clone().oneshot(req_del).await.unwrap();
    assert_eq!(res_del.status(), StatusCode::OK);

    // 5. Deleting again returns 404
    let req_del_again = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/admin/faqs/{}", faq_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res_del_again = app.oneshot(req_del_again).await.unwrap();
    assert_eq!(res_del_again.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn test_rate_limiting_ip_spoofing_prevention() {
    use pms_platform::rate_limit::{client_ip_from_headers, client_ip_from_request_parts};

    let mut headers = axum::http::HeaderMap::new();
    headers.insert("X-Forwarded-For", "203.0.113.195".parse().unwrap());
    headers.insert("X-Real-IP", "198.51.100.17".parse().unwrap());

    // Without TRUST_PROXY, header values are ignored to prevent spoofing
    std::env::remove_var("TRUST_PROXY");
    let ip = client_ip_from_headers(&headers);
    assert_eq!(
        ip,
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        "X-Forwarded-For should be ignored when TRUST_PROXY is not enabled"
    );

    // With TRUST_PROXY=true, X-Real-IP is preferred first
    std::env::set_var("TRUST_PROXY", "true");
    let ip_trusted = client_ip_from_headers(&headers);
    assert_eq!(
        ip_trusted,
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(198, 51, 100, 17)),
        "X-Real-IP should be preferred first when TRUST_PROXY=true"
    );

    // If X-Real-IP is missing, rightmost X-Forwarded-For hop is extracted
    headers.remove("X-Real-IP");
    let ip_xff = client_ip_from_headers(&headers);
    assert_eq!(
        ip_xff,
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(203, 0, 113, 195)),
        "Rightmost X-Forwarded-For should be respected when X-Real-IP is absent and TRUST_PROXY=true"
    );
    std::env::remove_var("TRUST_PROXY");

    // ConnectInfo extraction test
    let mut parts = Request::builder().uri("/").body(()).unwrap().into_parts().0;
    let socket: std::net::SocketAddr = "192.0.2.1:8080".parse().unwrap();
    parts.extensions.insert(axum::extract::ConnectInfo(socket));

    let resolved_ip = client_ip_from_request_parts(&parts);
    assert_eq!(
        resolved_ip,
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 0, 2, 1)),
        "Should resolve IP from ConnectInfo when available"
    );
}

#[tokio::test]
async fn test_body_limits_global_and_upload() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. Regular JSON endpoint with body > 256KB should be rejected with 413 Payload Too Large
    let large_text = "a".repeat(300 * 1024);
    let large_json = serde_json::json!({
        "title": "A",
        "date": "2026-10-19",
        "short_description": large_text,
    });
    let req_large_json = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/events")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&large_json).unwrap()))
        .unwrap();

    let res_large_json = app.clone().oneshot(req_large_json).await.unwrap();
    assert_eq!(
        res_large_json.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "JSON payload > 256KB should be rejected with 413"
    );

    // 2. Upload endpoint with ~500KB PDF (> 256KB) should succeed (under 20MB limit)
    let boundary = "------------------------limitboundary123";
    let pdf_magic = b"%PDF-1.4\n";
    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(pdf_magic);
    file_bytes.resize(500 * 1024, b'0');

    let mut multipart_data = Vec::new();
    multipart_data.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"document.pdf\"\r\nContent-Type: application/pdf\r\n\r\n",
            boundary = boundary
        )
        .as_bytes(),
    );
    multipart_data.extend_from_slice(&file_bytes);
    multipart_data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req_upload = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(multipart_data))
        .unwrap();

    let res_upload = app.oneshot(req_upload).await.unwrap();
    assert_eq!(
        res_upload.status(),
        StatusCode::OK,
        "Upload of 500KB file should succeed on /api/admin/upload"
    );
}

#[tokio::test]
async fn test_csrf_cookie_protection() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();
    let cookie_val = format!("pms_admin_token={}", token);

    let payload = serde_json::json!({
        "message": "Testing CSRF",
        "priority": "normal",
        "is_active": true
    });

    // 1. Cookie auth + POST without X-Requested-With header should fail with 400 Bad Request
    let req_no_header = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/announcements")
        .header(header::COOKIE, &cookie_val)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res_no_header = app.clone().oneshot(req_no_header).await.unwrap();
    assert_eq!(
        res_no_header.status(),
        StatusCode::FORBIDDEN,
        "Cookie-authenticated mutation without X-Requested-With should be rejected"
    );

    // 2. Cookie auth + POST with X-Requested-With: PMS-Client should succeed
    let req_with_header = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/announcements")
        .header(header::COOKIE, &cookie_val)
        .header("X-Requested-With", "PMS-Client")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res_with_header = app.clone().oneshot(req_with_header).await.unwrap();
    assert_eq!(
        res_with_header.status(),
        StatusCode::OK,
        "Cookie-authenticated mutation with X-Requested-With: PMS-Client should succeed"
    );

    // 3. Cookie auth + GET without X-Requested-With header should succeed (safe method)
    let req_get = Request::builder()
        .method(Method::GET)
        .uri("/api/admin/announcements")
        .header(header::COOKIE, &cookie_val)
        .body(Body::empty())
        .unwrap();

    let res_get = app.clone().oneshot(req_get).await.unwrap();
    assert_eq!(
        res_get.status(),
        StatusCode::OK,
        "Cookie-authenticated GET without custom header should be allowed"
    );

    // 4. Bearer auth without X-Requested-With header should succeed
    let req_bearer = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/announcements")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res_bearer = app.oneshot(req_bearer).await.unwrap();
    assert_eq!(
        res_bearer.status(),
        StatusCode::OK,
        "Bearer-authenticated mutation without custom header should succeed"
    );
}

#[tokio::test]
async fn test_migrations_on_copy_of_existing_db() {
    if std::path::Path::new("pms.db").exists() {
        let temp_db_path = format!("target/test_copy_{}.db", uuid::Uuid::new_v4());
        std::fs::copy("pms.db", &temp_db_path).unwrap();

        let pool = SqlitePoolOptions::new()
            .connect(&format!("sqlite://{}", temp_db_path))
            .await
            .unwrap();

        // Verify migrations run cleanly and idempotently
        db::run_migrations(&pool).await;

        drop(pool);
        let _ = std::fs::remove_file(temp_db_path);
    }
}

#[tokio::test]
async fn test_email_lowercasing_migration_handles_mixed_case_duplicates_without_losing_rows() {
    let temp_db_path = format!("target/test_email_dups_{}.db", uuid::Uuid::new_v4());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=rwc", temp_db_path))
        .await
        .unwrap();

    // 1. Create initial schema
    let schema = include_str!("../migrations/0001_initial_schema.sql");
    sqlx::raw_sql(schema).execute(&pool).await.unwrap();

    // 2. Pre-populate tables with duplicate mixed-case rows and single mixed-case rows
    // Argon2 hash for password "Password123!"
    let salt = argon2::password_hash::SaltString::generate(&mut rand_core::OsRng);
    let argon2 = argon2::Argon2::default();
    let hashed = argon2
        .hash_password(b"Password123!", &salt)
        .unwrap()
        .to_string();

    // admin_users duplicate pair + single mixed-case
    sqlx::query(
        "INSERT INTO admin_users (id, email, password_hash, name) VALUES (1, 'duplicate@pieas.edu.pk', ?1, 'User 1')"
    )
    .bind(&hashed)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO admin_users (id, email, password_hash, name) VALUES (2, 'Duplicate@pieas.edu.pk', ?1, 'User 2')"
    )
    .bind(&hashed)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO admin_users (id, email, password_hash, name) VALUES (3, 'SingleMixed@pieas.edu.pk', ?1, 'User 3')"
    )
    .bind(&hashed)
    .execute(&pool)
    .await
    .unwrap();

    // newsletter_subscribers duplicate pair + single mixed-case
    sqlx::query("INSERT INTO newsletter_subscribers (id, email) VALUES (1, 'sub@pieas.edu.pk')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO newsletter_subscribers (id, email) VALUES (2, 'Sub@pieas.edu.pk')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO newsletter_subscribers (id, email) VALUES (3, 'OtherMixed@pieas.edu.pk')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // 3. Run migration (must succeed without crashing or failing despite duplicate pair)
    db::run_migrations(&pool).await;

    // 4. Verify no rows were lost in admin_users
    let (admin_count,): (i64,) = sqlx::query_as("SELECT count(*) FROM admin_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(admin_count, 3, "No admin_users rows should be lost");

    // 5. Verify no rows were lost in newsletter_subscribers
    let (sub_count,): (i64,) = sqlx::query_as("SELECT count(*) FROM newsletter_subscribers")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        sub_count, 3,
        "No newsletter_subscribers rows should be lost"
    );

    // 6. Verify single mixed-case row was normalized to lowercase
    let (admin3_email,): (String,) = sqlx::query_as("SELECT email FROM admin_users WHERE id = 3")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(admin3_email, "singlemixed@pieas.edu.pk");

    let (sub3_email,): (String,) =
        sqlx::query_as("SELECT email FROM newsletter_subscribers WHERE id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(sub3_email, "othermixed@pieas.edu.pk");

    // 7. Verify colliding rows were safely preserved without corruption
    let (admin1_email,): (String,) = sqlx::query_as("SELECT email FROM admin_users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (admin2_email,): (String,) = sqlx::query_as("SELECT email FROM admin_users WHERE id = 2")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(admin1_email, "duplicate@pieas.edu.pk");
    assert_eq!(admin2_email, "Duplicate@pieas.edu.pk");

    // 8. Verify migration idempotence on second run
    db::run_migrations(&pool).await;
    let (admin_count2,): (i64,) = sqlx::query_as("SELECT count(*) FROM admin_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(admin_count2, 3, "Repeated migration must remain idempotent");

    // 9. Verify login still works for an admin whose email was originally mixed-case
    // Insert site_stats row 1 so create_app dependencies are met
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO site_stats (id, active_members, events_held, years_active, industrial_partners) VALUES (1, 10, 5, 2, 1)"
    )
    .execute(&pool)
    .await;

    let app = create_app(pool.clone());
    let login_payload = serde_json::json!({
        "email": "SingleMixed@pieas.edu.pk",
        "password": "Password123!"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Admin whose stored email was originally mixed-case should log in successfully"
    );

    // 10. Clean up temp DB files
    drop(pool);
    let _ = std::fs::remove_file(&temp_db_path);
    let _ = std::fs::remove_file(format!("{}-wal", temp_db_path));
    let _ = std::fs::remove_file(format!("{}-shm", temp_db_path));
}

#[tokio::test]
async fn test_token_revocation_on_logout() {
    let app = setup_test_app().await;

    // Generate valid token for admin user 1
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. Calling GET /api/auth/me with Bearer token works
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Call POST /api/auth/logout with Bearer token
    let req_logout = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res_logout = app.clone().oneshot(req_logout).await.unwrap();
    assert_eq!(res_logout.status(), StatusCode::OK);

    // 3. Calling GET /api/auth/me with the same Bearer token is now rejected with 401
    let req_again = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res_again = app.oneshot(req_again).await.unwrap();
    assert_eq!(res_again.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_ipv6_subnet_rate_limiting() {
    use pms_platform::rate_limit::canonicalize_rate_limit_ip;
    use std::net::IpAddr;

    let ip1: IpAddr = "2001:db8:abcd:0012:0000:0000:0000:0001".parse().unwrap();
    let ip2: IpAddr = "2001:db8:abcd:0012:ffff:ffff:ffff:ffff".parse().unwrap();
    let ip3: IpAddr = "2001:db8:abcd:0013:0000:0000:0000:0001".parse().unwrap();

    let c1 = canonicalize_rate_limit_ip(ip1);
    let c2 = canonicalize_rate_limit_ip(ip2);
    let c3 = canonicalize_rate_limit_ip(ip3);

    assert_eq!(
        c1, c2,
        "IPs within the same /64 subnet must canonicalize to the same prefix"
    );
    assert_ne!(
        c1, c3,
        "IPs in different /64 subnets must have distinct canonical prefixes"
    );

    // Also verify Cloudflare CF-Connecting-IP extraction when both flags are true
    std::env::set_var("TRUST_PROXY", "true");
    std::env::set_var("TRUST_CLOUDFLARE", "true");
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("CF-Connecting-IP", "203.0.113.88".parse().unwrap());
    headers.insert(
        "X-Forwarded-For",
        "203.0.113.88, 172.68.1.1".parse().unwrap(),
    );

    let extracted = pms_platform::rate_limit::client_ip_from_headers(&headers);
    assert_eq!(extracted, "203.0.113.88".parse::<IpAddr>().unwrap());
    std::env::remove_var("TRUST_PROXY");
    std::env::remove_var("TRUST_CLOUDFLARE");
}

#[tokio::test]
async fn test_stats_negative_number_validation() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    let payload = serde_json::json!({
        "active_members": -10,
        "events_held": 5
    });

    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/admin/stats")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_text_plain_upload_validation() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. Text upload with HTML/script should be rejected
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body_bad = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"notes.txt\"\r\nContent-Type: text/plain\r\n\r\n<script>alert(1)</script>\r\n--{boundary}--\r\n"
    );

    let req_bad = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body_bad))
        .unwrap();

    let res_bad = app.clone().oneshot(req_bad).await.unwrap();
    assert_eq!(res_bad.status(), StatusCode::BAD_REQUEST);

    // 2. Pure plaintext should be accepted
    let body_good = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"report.txt\"\r\nContent-Type: text/plain\r\n\r\nASTM E8 tensile test results: Yield strength 450 MPa\r\n--{boundary}--\r\n"
    );

    let req_good = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body_good))
        .unwrap();

    let res_good = app.oneshot(req_good).await.unwrap();
    assert_eq!(res_good.status(), StatusCode::OK);

    let body = res_good.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let url_str = json["url"].as_str().unwrap();
    let file_path = url_str.trim_start_matches('/');
    let _ = tokio::fs::remove_file(file_path).await;
}

#[tokio::test]
async fn test_jwt_secret_validation_rules() {
    use pms_platform::auth::validate_jwt_secret_candidate;

    assert!(validate_jwt_secret_candidate("too-short").is_err());
    assert!(
        validate_jwt_secret_candidate("replace-with-a-secure-random-secret-key-in-production")
            .is_err()
    );
    assert!(
        validate_jwt_secret_candidate("pms-dev-insecure-placeholder-do-not-use-in-production")
            .is_err()
    );
    assert!(validate_jwt_secret_candidate("0123456789abcdef0123456789abcdef").is_ok());
}

#[tokio::test]
async fn test_migrations_only_stats_returns_200() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory test db");

    db::run_migrations(&pool).await;

    let app = create_app(pool);
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stats")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_rate_limit_proxy_and_cloudflare_headers() {
    use std::net::IpAddr;

    // 1. Plain TRUST_PROXY=true (TRUST_CLOUDFLARE unset/false): ignore CF headers, use X-Real-IP
    std::env::set_var("TRUST_PROXY", "true");
    std::env::remove_var("TRUST_CLOUDFLARE");

    let mut headers = axum::http::HeaderMap::new();
    headers.insert("CF-Connecting-IP", "198.51.100.1".parse().unwrap());
    headers.insert("True-Client-IP", "198.51.100.2".parse().unwrap());
    headers.insert("X-Real-IP", "203.0.113.10".parse().unwrap());
    headers.insert("X-Forwarded-For", "10.0.0.1, 203.0.113.20".parse().unwrap());

    let ip = pms_platform::rate_limit::client_ip_from_headers(&headers);
    assert_eq!(ip, "203.0.113.10".parse::<IpAddr>().unwrap());

    // 2. Plain TRUST_PROXY=true without X-Real-IP: use rightmost X-Forwarded-For hop
    headers.remove("X-Real-IP");
    let ip = pms_platform::rate_limit::client_ip_from_headers(&headers);
    assert_eq!(ip, "203.0.113.20".parse::<IpAddr>().unwrap());

    // 3. TRUST_PROXY=true AND TRUST_CLOUDFLARE=true: trust CF-Connecting-IP
    std::env::set_var("TRUST_CLOUDFLARE", "true");
    let ip = pms_platform::rate_limit::client_ip_from_headers(&headers);
    assert_eq!(ip, "198.51.100.1".parse::<IpAddr>().unwrap());

    std::env::remove_var("TRUST_PROXY");
    std::env::remove_var("TRUST_CLOUDFLARE");
}

#[tokio::test]
async fn test_upload_file_derives_extension_and_returns_safe_url() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    let png_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4\x00\x00\x00\nIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\r\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82";

    // 1. x.html with PNG magic bytes -> saved as .png
    let boundary = "----WebKitFormBoundarySafeUpload123";
    let mut data1 = Vec::new();
    data1.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"x.html\"\r\nContent-Type: text/html\r\n\r\n",
            boundary = boundary
        )
        .as_bytes(),
    );
    data1.extend_from_slice(png_bytes);
    data1.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req1 = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(data1))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    let body1 = res1.into_body().collect().await.unwrap().to_bytes();
    let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
    let url1 = json1["url"].as_str().unwrap();
    let filename1 = json1["filename"].as_str().unwrap();

    assert!(url1.ends_with(".png"), "URL must end with .png");
    assert!(filename1.ends_with(".png"), "Filename must end with .png");
    assert!(!url1.contains(".html"), "URL must not contain .html");

    // 2. Check response headers on GET /uploads/...
    let req_get = Request::builder()
        .method(Method::GET)
        .uri(url1)
        .body(Body::empty())
        .unwrap();

    let res_get = app.clone().oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);
    let csp = res_get
        .headers()
        .get("Content-Security-Policy")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(
        csp,
        "default-src 'none'; style-src 'unsafe-inline'; sandbox"
    );
    let nosniff = res_get
        .headers()
        .get("X-Content-Type-Options")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(nosniff, "nosniff");
    let cache_control = res_get
        .headers()
        .get("Cache-Control")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(cache_control, "public, max-age=2592000");

    let _ = tokio::fs::remove_file(url1.trim_start_matches('/')).await;

    // 3. Filename with spaces, #, ? yields valid safe URL
    let mut data2 = Vec::new();
    data2.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"my space #file ? test.png\"\r\nContent-Type: image/png\r\n\r\n",
            boundary = boundary
        )
        .as_bytes(),
    );
    data2.extend_from_slice(png_bytes);
    data2.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req2 = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(data2))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    let body2 = res2.into_body().collect().await.unwrap().to_bytes();
    let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
    let url2 = json2["url"].as_str().unwrap();
    assert!(!url2.contains(' '), "URL must not contain spaces");
    assert!(!url2.contains('#'), "URL must not contain #");
    assert!(!url2.contains('?'), "URL must not contain ?");

    let _ = tokio::fs::remove_file(url2.trim_start_matches('/')).await;
}

#[tokio::test]
async fn test_logout_unverifiable_token_not_revoked() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    db::run_migrations(&pool).await;

    // Insert dummy admin
    sqlx::query(
        "INSERT OR IGNORE INTO admin_users (id, email, password_hash, name) VALUES (1, 'admin@pieas.edu.pk', 'dummyhash', 'Admin')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = create_app(pool.clone());

    // 1. Call logout with unverifiable token string
    let req_unverifiable = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header(header::AUTHORIZATION, "Bearer not.a.real.jwt.token")
        .body(Body::empty())
        .unwrap();

    let res1 = app.clone().oneshot(req_unverifiable).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // Verify revoked_tokens count is still 0
    let (count1,): (i64,) = sqlx::query_as("SELECT count(*) FROM revoked_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count1, 0,
        "Unverifiable token must not be inserted into revoked_tokens"
    );

    // 2. Call logout with a genuinely verifiable token
    let real_token = create_token(1, "admin@pieas.edu.pk").unwrap();
    let req_real = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {}", real_token))
        .body(Body::empty())
        .unwrap();

    let res2 = app.oneshot(req_real).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    let (count2,): (i64,) = sqlx::query_as("SELECT count(*) FROM revoked_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count2, 1,
        "Verifiable token must be inserted into revoked_tokens"
    );
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let app = setup_test_app().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_trailing_slashes_and_404_routes() {
    let app = setup_test_app().await;

    // 1. /about/ with trailing slash works and returns 200
    let req1 = Request::builder()
        .method(Method::GET)
        .uri("/about/")
        .body(Body::empty())
        .unwrap();
    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // 2. /404 returns 404 status
    let req2 = Request::builder()
        .method(Method::GET)
        .uri("/404")
        .body(Body::empty())
        .unwrap();
    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::NOT_FOUND);

    // 3. /api/* unmatched route returns 404 JSON shape
    let req3 = Request::builder()
        .method(Method::GET)
        .uri("/api/unknown_route_xyz")
        .body(Body::empty())
        .unwrap();
    let res3 = app.oneshot(req3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::NOT_FOUND);
    let body3 = res3.into_body().collect().await.unwrap().to_bytes();
    let json3: serde_json::Value = serde_json::from_slice(&body3).unwrap();
    assert_eq!(json3["status"], 404);
    assert_eq!(json3["error"], "Resource not found");
}

#[tokio::test]
async fn test_attendance_toggle_and_certificate_verification() {
    let (app, pool) = setup_test_app_and_pool().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. Create future event
    let future_date = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(10))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();

    let event_res = sqlx::query(
        "INSERT INTO events (title, slug, date, category, is_past) VALUES ('Ceramics Symposium', 'ceramics-symposium', ?1, 'Symposium', 0)"
    )
    .bind(&future_date)
    .execute(&pool)
    .await
    .unwrap();
    let event_id = event_res.last_insert_rowid();

    // 2. Register for event
    let reg_payload = serde_json::json!({
        "student_name": "Hamza Ali",
        "email": "hamza@pieas.edu.pk",
        "roll_number": "22-MS-01",
        "batch": "2022-2026",
        "department": "DMME"
    });

    let req_reg = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/events/{}/register", event_id))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&reg_payload).unwrap()))
        .unwrap();

    let res_reg = app.clone().oneshot(req_reg).await.unwrap();
    assert_eq!(res_reg.status(), StatusCode::OK);
    let reg_body = res_reg.into_body().collect().await.unwrap().to_bytes();
    let reg_json: serde_json::Value = serde_json::from_slice(&reg_body).unwrap();
    let reg_id = reg_json["id"].as_i64().unwrap();
    let cert_code = reg_json["certificate_code"].as_str().unwrap().to_string();
    assert_eq!(reg_json["attended"], false);

    // 3. Verify certificate before attendance marked -> valid: false
    let req_cert1 = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/certificates/verify?code={}", cert_code))
        .body(Body::empty())
        .unwrap();
    let res_cert1 = app.clone().oneshot(req_cert1).await.unwrap();
    assert_eq!(res_cert1.status(), StatusCode::OK);
    let cert1_json: serde_json::Value =
        serde_json::from_slice(&res_cert1.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(cert1_json["valid"], false);
    assert_eq!(cert1_json["reason"], "not_attended");
    assert!(cert1_json["student_name"].is_null());
    assert!(cert1_json["event_title"].is_null());

    // 4. Admin toggles attendance to true
    let toggle_payload = serde_json::json!({
        "attended": true
    });
    let req_toggle = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/admin/registrations/{}", reg_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&toggle_payload).unwrap()))
        .unwrap();
    let res_toggle = app.clone().oneshot(req_toggle).await.unwrap();
    assert_eq!(res_toggle.status(), StatusCode::OK);
    let toggle_json: serde_json::Value =
        serde_json::from_slice(&res_toggle.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    assert_eq!(toggle_json["attended"], true);

    // 5. Verify certificate after attendance marked -> valid: true
    let req_cert2 = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/certificates/verify?code={}", cert_code))
        .body(Body::empty())
        .unwrap();
    let res_cert2 = app.oneshot(req_cert2).await.unwrap();
    assert_eq!(res_cert2.status(), StatusCode::OK);
    let cert2_json: serde_json::Value =
        serde_json::from_slice(&res_cert2.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(cert2_json["valid"], true);
    assert_eq!(cert2_json["student_name"], "Hamza Ali");
    assert!(cert2_json["reason"].is_null());
}

#[tokio::test]
async fn test_login_token_return_header() {
    let (app, pool) = setup_test_app_and_pool().await;

    // Insert admin with known Argon2 hash
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let hashed = argon2::Argon2::default()
        .hash_password(b"TestPassword123!", &salt)
        .unwrap()
        .to_string();

    sqlx::query("UPDATE admin_users SET password_hash = ?1 WHERE id = 1")
        .bind(&hashed)
        .execute(&pool)
        .await
        .unwrap();

    let login_payload = serde_json::json!({
        "email": "admin@pieas.edu.pk",
        "password": "TestPassword123!"
    });

    // 1. Browser login without X-Return-Token: 1 -> token is null in JSON body
    let req_browser = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
        .unwrap();

    let res_browser = app.clone().oneshot(req_browser).await.unwrap();
    assert_eq!(res_browser.status(), StatusCode::OK);
    assert!(res_browser.headers().contains_key("set-cookie"));
    let browser_json: serde_json::Value =
        serde_json::from_slice(&res_browser.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    assert!(
        browser_json["token"].is_null(),
        "Browser login must not return token in JSON"
    );

    // 2. Programmatic login with X-Return-Token: 1 -> token is returned in JSON body
    let req_prog = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("X-Return-Token", "1")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
        .unwrap();

    let res_prog = app.oneshot(req_prog).await.unwrap();
    assert_eq!(res_prog.status(), StatusCode::OK);
    let prog_json: serde_json::Value =
        serde_json::from_slice(&res_prog.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(
        prog_json["token"].is_string(),
        "Programmatic login must return token in JSON"
    );
}

#[tokio::test]
async fn test_post_body_markdown_omitted_in_list_and_pagination() {
    let (app, pool) = setup_test_app_and_pool().await;

    sqlx::query(
        "INSERT INTO posts (title, slug, excerpt, body_markdown, category) VALUES ('Titanium Alloys', 'titanium-alloys', 'Short excerpt', '# Full Article Body Markdown', 'Technical')"
    )
    .execute(&pool)
    .await
    .unwrap();

    // 1. List query omits body_markdown (is null)
    let req_list = Request::builder()
        .method(Method::GET)
        .uri("/api/posts")
        .body(Body::empty())
        .unwrap();

    let res_list = app.clone().oneshot(req_list).await.unwrap();
    assert_eq!(res_list.status(), StatusCode::OK);
    let list_json: serde_json::Value =
        serde_json::from_slice(&res_list.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let posts = list_json.as_array().unwrap();
    assert!(!posts.is_empty());
    assert!(
        posts[0]["body_markdown"].is_null(),
        "body_markdown must be omitted in list query"
    );

    // 2. Slug detail endpoint returns full body_markdown
    let req_slug = Request::builder()
        .method(Method::GET)
        .uri("/api/posts/titanium-alloys")
        .body(Body::empty())
        .unwrap();

    let res_slug = app.oneshot(req_slug).await.unwrap();
    assert_eq!(res_slug.status(), StatusCode::OK);
    let slug_json: serde_json::Value =
        serde_json::from_slice(&res_slug.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(slug_json["body_markdown"], "# Full Article Body Markdown");
}

#[tokio::test]
async fn test_coalesce_partial_update_stats_and_empty_string_clear() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    // 1. Partial update: only update active_members, other fields should not be nulled
    let partial_payload = serde_json::json!({
        "active_members": 200
    });
    let req1 = Request::builder()
        .method(Method::PUT)
        .uri("/api/admin/stats")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&partial_payload).unwrap()))
        .unwrap();
    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // Verify via GET /api/stats that active_members is updated while events_held is preserved
    let req_get1 = Request::builder()
        .method(Method::GET)
        .uri("/api/stats")
        .body(Body::empty())
        .unwrap();
    let res_get1 = app.clone().oneshot(req_get1).await.unwrap();
    assert_eq!(res_get1.status(), StatusCode::OK);
    let json1: serde_json::Value =
        serde_json::from_slice(&res_get1.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(json1["active_members"], 200);
    assert_eq!(
        json1["events_held"], 30,
        "events_held should be preserved via COALESCE"
    );

    // 2. Explicit empty string clears optional field
    let clear_payload = serde_json::json!({
        "constitution_pdf_url": ""
    });
    let req2 = Request::builder()
        .method(Method::PUT)
        .uri("/api/admin/stats")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&clear_payload).unwrap()))
        .unwrap();
    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    // Verify via GET /api/stats that constitution_pdf_url is now null
    let req_get2 = Request::builder()
        .method(Method::GET)
        .uri("/api/stats")
        .body(Body::empty())
        .unwrap();
    let res_get2 = app.oneshot(req_get2).await.unwrap();
    assert_eq!(res_get2.status(), StatusCode::OK);
    let json2: serde_json::Value =
        serde_json::from_slice(&res_get2.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(
        json2["constitution_pdf_url"].is_null(),
        "Empty string should clear the field to null"
    );
}

#[tokio::test]
async fn test_unique_violation_returns_conflict_409() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    let payload = serde_json::json!({
        "section_key": "unique_hero_key",
        "title": "Hero",
        "content": "Hero content"
    });

    // First creation succeeds
    let req1 = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/sections")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // Duplicate section_key creation returns 409 Conflict
    let req2 = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/sections")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_standard_json_error_shapes_and_rejections() {
    let app = setup_test_app().await;

    // Malformed JSON (syntax error) to /api/contact should return 400 JSON
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/contact")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(b"{malformed_json_without_quotes}".to_vec()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 400);
    assert!(json["error"].is_string());

    // Deserialization type mismatch (e.g. number for email string) returns 422 JSON
    let bad_type_payload = serde_json::json!({
        "email": 12345
    });
    let req2 = Request::builder()
        .method(Method::POST)
        .uri("/api/newsletter")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&bad_type_payload).unwrap()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body2 = res2.into_body().collect().await.unwrap().to_bytes();
    let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
    assert_eq!(json2["status"], 422);
    assert!(json2["error"].is_string());
}

#[tokio::test]
async fn test_certificate_verify_reasons_not_found_and_not_attended() {
    let (app, pool) = setup_test_app_and_pool().await;

    // Unknown certificate code -> valid: false, reason: "not_found"
    let req_missing = Request::builder()
        .method(Method::GET)
        .uri("/api/certificates/verify?code=PMS-NONEXISTENT-9999")
        .body(Body::empty())
        .unwrap();
    let res_missing = app.clone().oneshot(req_missing).await.unwrap();
    assert_eq!(res_missing.status(), StatusCode::OK);
    let json_missing: serde_json::Value =
        serde_json::from_slice(&res_missing.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    assert_eq!(json_missing["valid"], false);
    assert_eq!(json_missing["reason"], "not_found");
    assert!(json_missing["student_name"].is_null());
    assert!(json_missing["event_title"].is_null());
    assert!(json_missing["certificate_code"].is_null());

    // Seed an event + unattended registration -> valid: false, reason: "not_attended"
    sqlx::query(
        "INSERT INTO events (title, slug, date, category, is_past) VALUES ('Test Workshop', 'test-workshop-reason', '2026-10-10', 'Workshop', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (event_id,): (i64,) =
        sqlx::query_as("SELECT id FROM events WHERE slug = 'test-workshop-reason'")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query(
        "INSERT INTO event_registrations (event_id, student_name, roll_number, email, department, batch, attended, certificate_code) VALUES (?1, 'Ayesha Khan', '2023-BSME-001', 'ayesha@pieas.edu.pk', 'DMME', '2023', 0, 'PMS-UNATTENDED-001')",
    )
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();

    let req_unattended = Request::builder()
        .method(Method::GET)
        .uri("/api/certificates/verify?code=PMS-UNATTENDED-001")
        .body(Body::empty())
        .unwrap();
    let res_unattended = app.oneshot(req_unattended).await.unwrap();
    assert_eq!(res_unattended.status(), StatusCode::OK);
    let json_unattended: serde_json::Value = serde_json::from_slice(
        &res_unattended
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();
    assert_eq!(json_unattended["valid"], false);
    assert_eq!(json_unattended["reason"], "not_attended");
    assert!(json_unattended["student_name"].is_null());
    assert!(json_unattended["event_title"].is_null());
    assert!(json_unattended["certificate_code"].is_null());
}

#[tokio::test]
#[serial]
async fn test_sitemap_includes_seeded_post_slug_and_events() {
    let (app, pool) = setup_test_app_and_pool().await;
    db::seed_data(&pool).await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/sitemap.xml")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let xml = String::from_utf8_lossy(&body);
    assert!(xml.contains("/blog/material-of-the-month-zircaloy-inconel</loc>"));
    assert!(xml.contains("/events</loc>"));
    assert!(!xml.contains('#'));
}

#[tokio::test]
async fn test_team_member_default_tenure_year_is_computed() {
    let app = setup_test_app().await;
    let token = create_token(1, "admin@pieas.edu.pk").unwrap();

    use chrono::{Datelike, Utc};
    let now = Utc::now();
    let y = now.year();
    let expected_tenure = if now.month() >= 8 {
        format!("{}-{}", y, y + 1)
    } else {
        format!("{}-{}", y - 1, y)
    };

    let payload = serde_json::json!({
        "name": "Test Coordinator",
        "role": "Coordinator"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/team")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let req_get = Request::builder()
        .method(Method::GET)
        .uri("/api/team")
        .body(Body::empty())
        .unwrap();
    let res_get = app.oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&res_get.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body[0]["tenure_year"], expected_tenure);
}
