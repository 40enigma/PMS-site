use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use pms_platform::{auth::create_token, create_app, db};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn setup_test_app() -> axum::Router {
    // In-memory SQLite for testing (max_connections(1) so in-memory db is shared across queries)
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory test db");

    db::run_migrations(&pool).await;

    // Insert dummy stats row (id=1)
    sqlx::query(
        "INSERT OR IGNORE INTO site_stats (id, active_members, events_held, years_active, industrial_partners) VALUES (1, 150, 30, 8, 12)"
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

    create_app(pool)
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
async fn test_login_invalid_credentials_returns_401_and_rate_limits() {
    let app = setup_test_app().await;

    // Set a custom test IP via X-Forwarded-For to test this specific IP's rate limiter bucket
    let test_ip = "198.51.100.42";

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
            .header("X-Forwarded-For", test_ip)
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let res = app_clone.oneshot(req).await.unwrap();
        if res.status() == StatusCode::TOO_MANY_REQUESTS {
            got_429 = true;
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

    // With TRUST_PROXY=true, rightmost hop is extracted
    std::env::set_var("TRUST_PROXY", "true");
    let ip_trusted = client_ip_from_headers(&headers);
    assert_eq!(
        ip_trusted,
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(203, 0, 113, 195)),
        "X-Forwarded-For should be respected when TRUST_PROXY=true"
    );
    std::env::remove_var("TRUST_PROXY");

    // ConnectInfo extraction test
    let mut parts = Request::builder()
        .uri("/")
        .body(())
        .unwrap()
        .into_parts()
        .0;
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
        StatusCode::BAD_REQUEST,
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


