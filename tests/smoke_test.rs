use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use pms_platform::{auth::create_token, create_app, db};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn setup_test_app() -> axum::Router {
    // In-memory SQLite for testing
    let pool = SqlitePoolOptions::new()
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
