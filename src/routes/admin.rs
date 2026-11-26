use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use sqlx::SqlitePool;
use time::Duration;

use crate::auth::{create_token, jwt_cookie_name, jwt_expiry_hours};
use crate::error::AppError;
use crate::models::{
    AdminClaims, AdminUser, Alumnus, Announcement, ContactMessage, CreateAlumnus,
    CreateAnnouncement, CreateEvent, CreateFaq, CreateGalleryImage, CreateMembershipApplication,
    CreateOpportunity, CreatePost, CreateProject, CreateResource, CreateSiteSection, CreateSponsor,
    CreateTeamMember, DeleteResponse, Event, EventRegistration, Faq, GalleryImage, LoginRequest,
    LoginResponse, MembershipApplication, NewsletterSubscriber, Opportunity, PaginationParams,
    Post, Project, Resource, SiteSection, SiteStats, Sponsor, TeamMember, UpdateEventRegistration,
    UpdateSiteSection, UpdateSiteStats, UploadResponse,
};
use crate::rate_limit::{ClientIp, RateLimitState};
use crate::routes::public::validate_membership_payload;
use crate::util::{
    current_academic_year, is_foreign_key_violation, is_production, is_unique_violation, slugify,
    validate_clearable_safe_url, validate_clearable_str, validate_email, validate_enum,
    validate_iso_date, validate_optional_iso_date, validate_optional_safe_url,
    validate_optional_str, validate_required_str, validate_safe_url,
};

// ---------------------------------------------------------------------------
// Allowed MIME types & Enumerations
// ---------------------------------------------------------------------------

const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/webp",
    "application/pdf",
    "text/plain",
];

/// Extensions that are explicitly blocked regardless of content-type header.
const BLOCKED_EXTENSIONS: &[&str] = &[
    ".exe", ".sh", ".bash", ".php", ".pl", ".cgi", ".js", ".mjs", ".py", ".rb", ".bat", ".cmd",
    ".ps1", ".vbs", ".jar", ".war", ".class", ".elf", ".dll", ".so", ".msi",
];

const ANNOUNCEMENT_PRIORITIES: &[&str] =
    &["info", "urgent", "warning", "event", "success", "normal"];
const EVENT_CATEGORIES: &[&str] = &[
    "Seminar",
    "Workshop",
    "Competition",
    "Industrial Visit",
    "Symposium",
    "Lab Visit",
    "Alumni Talk",
];
const POST_CATEGORIES: &[&str] = &[
    "Material of the Month",
    "Explainer",
    "Technical",
    "Alloy Highlight",
    "Characterization",
    "Additive Manufacturing",
    "Student Research",
];
const RESOURCE_CATEGORIES: &[&str] = &[
    "Standards",
    "Software",
    "Safety",
    "Databases",
    "Books",
    "Tools",
];
const OPPORTUNITY_TYPES: &[&str] = &[
    "Internship",
    "Scholarship",
    "Job",
    "Research",
    "Research Position",
    "Conference",
    "Competition",
];
const PROJECT_TYPES: &[&str] = &["FYP", "Graduate Research", "Term Project", "Competition"];
const SPONSOR_TIERS: &[&str] = &["Platinum", "Gold", "Silver", "Bronze", "Partner"];
const FAQ_CATEGORIES: &[&str] = &[
    "General",
    "Membership",
    "Events",
    "Academic",
    "Careers",
    "Council",
    "Certificates",
    "Collaboration",
];

pub fn admin_routes() -> Router<SqlitePool> {
    Router::new()
        // Auth
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(verify_session))
        // Upload
        .route(
            "/api/admin/upload",
            post(upload_file).layer(DefaultBodyLimit::max(20 * 1024 * 1024)),
        )
        .route("/api/admin/uploads", get(list_uploads))
        .route("/api/admin/uploads/{filename}", delete(delete_upload))
        // Generic CRUD for all models
        .route(
            "/api/admin/announcements",
            get(list_announcements).post(create_announcement),
        )
        .route(
            "/api/admin/announcements/{id}",
            put(update_announcement).delete(delete_announcement),
        )
        .route("/api/admin/stats", get(get_stats).put(update_stats))
        .route("/api/admin/events", get(list_events).post(create_event))
        .route(
            "/api/admin/events/{id}",
            put(update_event).delete(delete_event),
        )
        .route("/api/admin/registrations", get(list_registrations))
        .route(
            "/api/admin/registrations/{id}",
            put(update_registration).delete(delete_registration),
        )
        .route("/api/admin/team", get(list_team).post(create_team_member))
        .route(
            "/api/admin/team/{id}",
            put(update_team_member).delete(delete_team_member),
        )
        .route(
            "/api/admin/gallery",
            get(list_gallery).post(create_gallery_image),
        )
        .route(
            "/api/admin/gallery/{id}",
            put(update_gallery_image).delete(delete_gallery_image),
        )
        .route("/api/admin/posts", get(list_posts).post(create_post))
        .route(
            "/api/admin/posts/{id}",
            put(update_post).delete(delete_post),
        )
        .route(
            "/api/admin/resources",
            get(list_resources).post(create_resource),
        )
        .route(
            "/api/admin/resources/{id}",
            put(update_resource).delete(delete_resource),
        )
        .route(
            "/api/admin/opportunities",
            get(list_opportunities).post(create_opportunity),
        )
        .route(
            "/api/admin/opportunities/{id}",
            put(update_opportunity).delete(delete_opportunity),
        )
        .route("/api/admin/alumni", get(list_alumni).post(create_alumnus))
        .route(
            "/api/admin/alumni/{id}",
            put(update_alumnus).delete(delete_alumnus),
        )
        .route(
            "/api/admin/projects",
            get(list_projects).post(create_project),
        )
        .route(
            "/api/admin/projects/{id}",
            put(update_project).delete(delete_project),
        )
        .route(
            "/api/admin/sponsors",
            get(list_sponsors).post(create_sponsor),
        )
        .route(
            "/api/admin/sponsors/{id}",
            put(update_sponsor).delete(delete_sponsor),
        )
        .route("/api/admin/faqs", get(list_faqs).post(create_faq))
        .route("/api/admin/faqs/{id}", put(update_faq).delete(delete_faq))
        .route(
            "/api/admin/membership",
            get(list_membership).post(create_membership_from_admin),
        )
        .route(
            "/api/admin/membership/{id}",
            put(update_membership_status).delete(delete_membership),
        )
        .route("/api/admin/contacts", get(list_contacts))
        .route("/api/admin/contacts/{id}", delete(delete_contact))
        .route("/api/admin/newsletter", get(list_newsletter))
        .route("/api/admin/newsletter/{id}", delete(delete_subscriber))
        .route(
            "/api/admin/sections",
            get(list_sections).post(create_section),
        )
        .route(
            "/api/admin/sections/{id}",
            put(update_section).delete(delete_section),
        )
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

/// Returns `true` if the auth cookie should have the `Secure` flag set.
fn is_cookie_secure() -> bool {
    std::env::var("JWT_COOKIE_SECURE")
        .map_or_else(|_| is_production(), |v| v.to_lowercase() != "false")
}

/// Build the auth cookie with appropriate security attributes.
fn build_auth_cookie(token: String) -> Cookie<'static> {
    let mut cookie = Cookie::new(jwt_cookie_name(), token);
    cookie.set_http_only(true);
    cookie.set_secure(is_cookie_secure());
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(Duration::hours(jwt_expiry_hours()));
    cookie
}

/// Authenticate an administrator and issue an httpOnly JWT cookie (plus optional JSON token).
///
/// # Errors
/// Returns `AppError::TooManyRequests`, `AppError::BadRequest`, `AppError::Unauthorized`, or `AppError::Internal`.
#[axum::debug_handler]
pub async fn login(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    ClientIp(ip): ClientIp,
    headers: axum::http::HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
    RateLimitState::global().check_login(ip)?;

    let email = payload.email.trim().to_lowercase();

    if email.is_empty() || payload.password.is_empty() {
        return Err(AppError::BadRequest(
            "Email and password are required".into(),
        ));
    }
    if email.len() > 254 {
        return Err(AppError::BadRequest("Email too long".into()));
    }

    let user = sqlx::query_as::<_, AdminUser>(
        "SELECT id, email, password_hash, name FROM admin_users WHERE email = ?1",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await?;

    let (hash_to_verify, user) = match user {
        Some(u) => (u.password_hash.clone(), Some(u)),
        None => (
            "$argon2id$v=19$m=19456,t=2,p=1$kc2irIUf5Ibqnm5+yA7gMg$qyiSjMzAeQTOHoQjm3Gj+qTPQyVVJFlZzN1wL7zW1FM".to_string(),
            None,
        ),
    };

    let password = payload.password.clone();
    let verify_res = tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash_to_verify)
            .map_err(|_| AppError::Internal("Hash parse error".into()))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = match user {
        Some(u) => {
            verify_res?;
            u
        }
        None => {
            return Err(AppError::Unauthorized);
        }
    };

    let token = create_token(user.id, &user.email)?;

    let cookie = build_auth_cookie(token.clone());
    let updated_jar = jar.add(cookie);

    let return_token = headers
        .get("x-return-token")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.trim() == "1");

    let response_token = if return_token { Some(token) } else { None };

    tracing::info!(user_id = user.id, "Admin login successful");

    Ok((
        updated_jar,
        Json(LoginResponse {
            token: response_token,
            name: user.name,
            email: user.email,
        }),
    ))
}

async fn revoke_if_valid(pool: &SqlitePool, token: &str) {
    if let Ok(claims) = crate::auth::verify_token(token) {
        let th = crate::auth::hash_token(token);
        let exp = i64::try_from(claims.exp).unwrap_or(i64::MAX);
        if let Err(err) = sqlx::query(
            "INSERT OR IGNORE INTO revoked_tokens (token_hash, expires_at) VALUES (?1, ?2)",
        )
        .bind(&th)
        .bind(exp)
        .execute(pool)
        .await
        {
            tracing::warn!(error = %err, "Failed to record revoked token on logout");
        }
    }
}

/// Revoke active JWT session(s) and clear the session cookie.
///
/// # Errors
/// Returns `AppError::TooManyRequests` or `AppError::Forbidden` if cookie-authenticated without CSRF header.
#[axum::debug_handler]
pub async fn logout(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    ClientIp(ip): ClientIp,
    headers: axum::http::HeaderMap,
) -> Result<(CookieJar, Json<serde_json::Value>), AppError> {
    RateLimitState::global().check_submit(ip)?;

    let bearer_opt = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(str::to_string);

    let cookie_opt = jar.get(jwt_cookie_name()).map(|c| c.value().to_string());

    // Enforce CSRF protection when logout relies on cookie credentials (no Bearer header)
    if bearer_opt.is_none() && cookie_opt.is_some() {
        let req_with = headers
            .get("x-requested-with")
            .and_then(|v| v.to_str().ok());
        if req_with != Some("PMS-Client") {
            return Err(AppError::Forbidden(
                "CSRF protection: missing or invalid X-Requested-With header".into(),
            ));
        }
    }

    if let Some(ref bearer) = bearer_opt {
        revoke_if_valid(&pool, bearer).await;
    }
    if let Some(ref cookie_tok) = cookie_opt {
        if bearer_opt.as_deref() != Some(cookie_tok.as_str()) {
            revoke_if_valid(&pool, cookie_tok).await;
        }
    }

    let removal = Cookie::build((jwt_cookie_name(), ""))
        .path("/")
        .http_only(true)
        .secure(is_cookie_secure())
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .build();

    let updated_jar = jar.remove(removal);
    Ok((
        updated_jar,
        Json(serde_json::json!({"message": "Logged out successfully"})),
    ))
}

/// Verify the current admin session and return profile metadata.
///
/// # Errors
/// Returns `AppError::Unauthorized` or `AppError::Database`.
#[axum::debug_handler]
pub async fn verify_session(
    claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query_as::<_, AdminUser>(
        "SELECT id, email, password_hash, name FROM admin_users WHERE id = ?1",
    )
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok(Json(LoginResponse {
        token: None,
        name: user.name,
        email: user.email,
    }))
}

// ---------------------------------------------------------------------------
// File Upload (hardened)
// ---------------------------------------------------------------------------

#[axum::debug_handler(state = SqlitePool)]
async fn upload_file(
    _claims: AdminClaims,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let raw_name = field
            .file_name()
            .map_or_else(|| format!("{}.bin", uuid::Uuid::new_v4()), str::to_string);

        // Strip directory traversal attempts.
        let file_name = std::path::Path::new(&raw_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload.bin")
            .to_string();

        // Check for blocked extensions.
        let lower = file_name.to_lowercase();
        for blocked in BLOCKED_EXTENSIONS {
            if lower.ends_with(blocked) {
                return Err(AppError::BadRequest(format!(
                    "File type not allowed: {blocked}"
                )));
            }
        }

        let declared_ct = field.content_type().map(str::to_string);

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        if data.is_empty() {
            return Err(AppError::BadRequest("Uploaded file is empty".into()));
        }
        if data.len() > 20 * 1024 * 1024 {
            return Err(AppError::BadRequest("File exceeds 20MB limit".into()));
        }

        let is_declared_text_plain = declared_ct
            .as_deref()
            .is_some_and(|ct| ct.trim().to_ascii_lowercase().starts_with("text/plain"));
        let has_txt_ext = std::path::Path::new(&lower)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"));

        // Sniff the actual MIME type from magic bytes.
        let sniffed_mime = if let Some(t) = infer::get(&data) {
            t.mime_type()
        } else if let Ok(s) = std::str::from_utf8(&data) {
            let s_lower = s.to_lowercase();
            if s_lower.contains("<script")
                || s_lower.contains("<html")
                || s_lower.contains("<?php")
                || s_lower.contains("<!doctype")
                || s_lower.contains("<svg")
                || s_lower.contains("<iframe")
                || s_lower.contains("<object")
                || s_lower.contains("<embed")
            {
                return Err(AppError::BadRequest(
                    "Text file contains disallowed script or markup content".into(),
                ));
            }
            if (is_declared_text_plain || has_txt_ext) && !data.contains(&0) {
                "text/plain"
            } else {
                "application/octet-stream"
            }
        } else {
            "application/octet-stream"
        };

        // Derive the extension from the sniffed MIME type (png/jpg/webp/pdf/txt)
        let ext = match sniffed_mime {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            "application/pdf" => "pdf",
            "text/plain" => "txt",
            _ => {
                return Err(AppError::BadRequest(format!(
                    "File content type '{}' is not allowed. Allowed: {}",
                    sniffed_mime,
                    ALLOWED_MIME_TYPES.join(", ")
                )));
            }
        };

        // Cross-check declared vs sniffed content type (informational log).
        if let Some(ref declared) = declared_ct {
            if !declared.to_ascii_lowercase().starts_with(sniffed_mime) {
                tracing::warn!(
                    declared = %declared,
                    sniffed = %sniffed_mime,
                    "Content-Type mismatch between declared and sniffed MIME"
                );
            }
        }

        // Do not use client-supplied filename; name files {uuid}.{ext}
        let upload_directory = crate::upload_dir();
        tokio::fs::create_dir_all(&upload_directory)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let safe_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
        let path = format!("{upload_directory}/{safe_name}");

        tokio::fs::write(&path, &data)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Set permissions to rw-r--r-- (0o644) — non-executable on Unix platforms.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        tracing::info!(
            filename = %safe_name,
            mime = %sniffed_mime,
            size = data.len(),
            "File uploaded successfully"
        );

        return Ok(Json(UploadResponse {
            url: format!("/uploads/{safe_name}"),
            filename: safe_name,
        }));
    }

    Err(AppError::BadRequest("No file provided".into()))
}

#[derive(serde::Serialize)]
pub struct UploadedFileItem {
    pub filename: String,
    pub url: String,
    pub size: u64,
}

#[axum::debug_handler(state = SqlitePool)]
async fn list_uploads(
    _claims: AdminClaims,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<UploadedFileItem>>, AppError> {
    let dir = crate::upload_dir();
    let mut entries = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut files = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(meta) = entry.metadata().await {
            if meta.is_file() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with('.') {
                        continue;
                    }
                    let size = meta.len();
                    files.push(UploadedFileItem {
                        url: format!("/uploads/{name}"),
                        filename: name,
                        size,
                    });
                }
            }
        }
    }
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    let offset = usize::try_from(pagination.offset()).unwrap_or(0);
    let limit = usize::try_from(pagination.limit()).unwrap_or(50);
    let paged = files.into_iter().skip(offset).take(limit).collect();
    Ok(Json(paged))
}

#[axum::debug_handler(state = SqlitePool)]
async fn delete_upload(
    _claims: AdminClaims,
    Path(filename): Path<String>,
) -> Result<Json<DeleteResponse>, AppError> {
    if filename.contains('/') || filename.contains('\\') || filename.starts_with('.') {
        return Err(AppError::BadRequest("Invalid filename".into()));
    }
    let safe_filename = std::path::Path::new(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.starts_with('.'))
        .ok_or_else(|| AppError::BadRequest("Invalid filename".into()))?;

    let path = format!("{}/{}", crate::upload_dir(), safe_filename);
    match tokio::fs::remove_file(&path).await {
        Ok(()) => Ok(Json(DeleteResponse {
            message: "File deleted successfully".into(),
        })),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(AppError::NotFound),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Announcements
// ---------------------------------------------------------------------------

struct ValidAnnouncement {
    message: String,
    link_text: Option<String>,
    link_url: Option<String>,
    priority: String,
    is_active: bool,
}

fn validate_announcement(p: &CreateAnnouncement) -> Result<ValidAnnouncement, AppError> {
    Ok(ValidAnnouncement {
        message: validate_required_str(&p.message, "message", 1000)?,
        link_text: validate_optional_str(p.link_text.as_deref(), "link_text", 200)?,
        link_url: validate_optional_safe_url(p.link_url.as_deref(), "link_url", 2000)?,
        priority: validate_enum(
            p.priority.as_deref(),
            "info",
            ANNOUNCEMENT_PRIORITIES,
            "priority",
        )?,
        is_active: p.is_active.unwrap_or(true),
    })
}

#[axum::debug_handler]
async fn list_announcements(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Announcement>>, AppError> {
    let rows = sqlx::query_as::<_, Announcement>(
        "SELECT id, message, link_text, link_url, priority, is_active, created_at FROM announcements ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_announcement(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateAnnouncement>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_announcement(&p)?;

    sqlx::query("INSERT INTO announcements (message, link_text, link_url, priority, is_active) VALUES (?1,?2,?3,?4,?5)")
        .bind(&v.message)
        .bind(&v.link_text)
        .bind(&v.link_url)
        .bind(&v.priority)
        .bind(v.is_active)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_announcement(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateAnnouncement>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_announcement(&p)?;

    let res = sqlx::query("UPDATE announcements SET message=?1, link_text=?2, link_url=?3, priority=?4, is_active=?5 WHERE id=?6")
        .bind(&v.message)
        .bind(&v.link_text)
        .bind(&v.link_url)
        .bind(&v.priority)
        .bind(v.is_active)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_announcement(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM announcements WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Site Stats
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn get_stats(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<SiteStats>, AppError> {
    let row = sqlx::query_as::<_, SiteStats>(
        "SELECT id, active_members, events_held, years_active, industrial_partners, constitution_pdf_url, office_location, contact_email FROM site_stats WHERE id=1",
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

#[axum::debug_handler]
async fn update_stats(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<UpdateSiteStats>,
) -> Result<Json<DeleteResponse>, AppError> {
    for (val, label) in [
        (p.active_members, "active_members"),
        (p.events_held, "events_held"),
        (p.years_active, "years_active"),
        (p.industrial_partners, "industrial_partners"),
    ] {
        if let Some(n) = val {
            if n < 0 {
                return Err(AppError::BadRequest(format!("{label} cannot be negative")));
            }
        }
    }

    let constitution_pdf_url = validate_clearable_safe_url(
        p.constitution_pdf_url.as_deref(),
        "constitution_pdf_url",
        2000,
    )?;
    let office_location =
        validate_clearable_str(p.office_location.as_deref(), "office_location", 500)?;
    let contact_email = match p.contact_email.as_deref() {
        None => None,
        Some(s) if s.trim().is_empty() => Some(String::new()),
        Some(s) => Some(validate_email(s)?),
    };

    let res = sqlx::query(
        "UPDATE site_stats SET \
         active_members=COALESCE(?1,active_members), \
         events_held=COALESCE(?2,events_held), \
         years_active=COALESCE(?3,years_active), \
         industrial_partners=COALESCE(?4,industrial_partners), \
         constitution_pdf_url=CASE WHEN ?5 = '' THEN NULL ELSE COALESCE(?5,constitution_pdf_url) END, \
         office_location=CASE WHEN ?6 = '' THEN NULL ELSE COALESCE(?6,office_location) END, \
         contact_email=CASE WHEN ?7 = '' THEN NULL ELSE COALESCE(?7,contact_email) END \
         WHERE id=1",
    )
    .bind(p.active_members)
    .bind(p.events_held)
    .bind(p.years_active)
    .bind(p.industrial_partners)
    .bind(&constitution_pdf_url)
    .bind(&office_location)
    .bind(&contact_email)
    .execute(&pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

// ---------------------------------------------------------------------------
// Events CRUD
// ---------------------------------------------------------------------------

struct ValidEvent {
    title: String,
    slug: String,
    date: String,
    time_range: Option<String>,
    venue: Option<String>,
    category: String,
    short_description: Option<String>,
    full_summary: Option<String>,
    poster_url: Option<String>,
    registration_link: Option<String>,
    speaker_or_organizer: Option<String>,
    is_past: bool,
    is_featured: bool,
}

fn validate_event(p: &CreateEvent) -> Result<ValidEvent, AppError> {
    let title = validate_required_str(&p.title, "title", 300)?;
    let date = validate_iso_date(&p.date, "date")?;
    let slug = match p.slug.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => slugify(s),
        None => slugify(&title),
    };
    Ok(ValidEvent {
        title,
        slug,
        date,
        time_range: validate_optional_str(p.time_range.as_deref(), "time_range", 200)?,
        venue: validate_optional_str(p.venue.as_deref(), "venue", 300)?,
        category: validate_enum(
            p.category.as_deref(),
            "Seminar",
            EVENT_CATEGORIES,
            "category",
        )?,
        short_description: validate_optional_str(
            p.short_description.as_deref(),
            "short_description",
            2000,
        )?,
        full_summary: validate_optional_str(p.full_summary.as_deref(), "full_summary", 20000)?,
        poster_url: validate_optional_safe_url(p.poster_url.as_deref(), "poster_url", 2000)?,
        registration_link: validate_optional_safe_url(
            p.registration_link.as_deref(),
            "registration_link",
            2000,
        )?,
        speaker_or_organizer: validate_optional_str(
            p.speaker_or_organizer.as_deref(),
            "speaker_or_organizer",
            300,
        )?,
        is_past: p.is_past.unwrap_or(false),
        is_featured: p.is_featured.unwrap_or(false),
    })
}

#[axum::debug_handler]
async fn list_events(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Event>>, AppError> {
    let rows = sqlx::query_as::<_, Event>(
        "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events ORDER BY date DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_event(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateEvent>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_event(&p)?;
    let res = sqlx::query("INSERT INTO events (title,slug,date,time_range,venue,category,short_description,full_summary,poster_url,registration_link,speaker_or_organizer,is_past,is_featured) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)")
        .bind(&v.title)
        .bind(&v.slug)
        .bind(&v.date)
        .bind(&v.time_range)
        .bind(&v.venue)
        .bind(&v.category)
        .bind(&v.short_description)
        .bind(&v.full_summary)
        .bind(&v.poster_url)
        .bind(&v.registration_link)
        .bind(&v.speaker_or_organizer)
        .bind(v.is_past)
        .bind(v.is_featured)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => Ok(Json(DeleteResponse {
            message: "Created".into(),
        })),
        Err(err) if is_unique_violation(&err) => Err(AppError::Conflict(
            "An event with this title or slug already exists".into(),
        )),
        Err(err) => Err(AppError::Database(err)),
    }
}

#[axum::debug_handler]
async fn update_event(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateEvent>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_event(&p)?;
    let res = sqlx::query("UPDATE events SET title=?1,slug=?2,date=?3,time_range=?4,venue=?5,category=?6,short_description=?7,full_summary=?8,poster_url=?9,registration_link=?10,speaker_or_organizer=?11,is_past=?12,is_featured=?13 WHERE id=?14")
        .bind(&v.title)
        .bind(&v.slug)
        .bind(&v.date)
        .bind(&v.time_range)
        .bind(&v.venue)
        .bind(&v.category)
        .bind(&v.short_description)
        .bind(&v.full_summary)
        .bind(&v.poster_url)
        .bind(&v.registration_link)
        .bind(&v.speaker_or_organizer)
        .bind(v.is_past)
        .bind(v.is_featured)
        .bind(id)
        .execute(&pool)
        .await;

    let res = match res {
        Ok(r) => r,
        Err(err) if is_unique_violation(&err) => {
            return Err(AppError::Conflict(
                "An event with this title or slug already exists".into(),
            ));
        }
        Err(err) => return Err(AppError::Database(err)),
    };
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_event(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM events WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Registrations (Read/Update/Delete)
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_registrations(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<EventRegistration>>, AppError> {
    let rows = sqlx::query_as::<_, EventRegistration>(
        "SELECT id, event_id, student_name, roll_number, batch, department, email, phone, certificate_code, attended, created_at FROM event_registrations ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn update_registration(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateEventRegistration>,
) -> Result<Json<EventRegistration>, AppError> {
    let current: Option<(bool,)> =
        sqlx::query_as("SELECT attended FROM event_registrations WHERE id = ?1")
            .bind(id)
            .fetch_optional(&pool)
            .await?;

    let (current_attended,) = current.ok_or(AppError::NotFound)?;
    let new_attended = payload.attended.unwrap_or(!current_attended);

    sqlx::query("UPDATE event_registrations SET attended = ?1 WHERE id = ?2")
        .bind(new_attended)
        .bind(id)
        .execute(&pool)
        .await?;

    let updated = sqlx::query_as::<_, EventRegistration>(
        "SELECT id, event_id, student_name, roll_number, batch, department, email, phone, certificate_code, attended, created_at FROM event_registrations WHERE id = ?1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated))
}

#[axum::debug_handler]
async fn delete_registration(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM event_registrations WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Team Members CRUD
// ---------------------------------------------------------------------------

struct ValidTeamMember {
    name: String,
    role: String,
    role_responsibility: Option<String>,
    photo_url: Option<String>,
    batch: Option<String>,
    tenure_year: String,
    is_current: bool,
    is_faculty_advisor: bool,
    bio: Option<String>,
    linkedin_url: Option<String>,
    email: Option<String>,
    display_order: i64,
}

fn validate_team_member(p: &CreateTeamMember) -> Result<ValidTeamMember, AppError> {
    let tenure_year = match validate_optional_str(p.tenure_year.as_deref(), "tenure_year", 50)? {
        Some(t) => t,
        None => current_academic_year(),
    };
    let email = match p.email.as_deref() {
        None => None,
        Some(s) if s.trim().is_empty() => None,
        Some(s) => Some(validate_email(s)?),
    };
    Ok(ValidTeamMember {
        name: validate_required_str(&p.name, "name", 200)?,
        role: validate_required_str(&p.role, "role", 200)?,
        role_responsibility: validate_optional_str(
            p.role_responsibility.as_deref(),
            "role_responsibility",
            1000,
        )?,
        photo_url: validate_optional_safe_url(p.photo_url.as_deref(), "photo_url", 2000)?,
        batch: validate_optional_str(p.batch.as_deref(), "batch", 50)?,
        tenure_year,
        is_current: p.is_current.unwrap_or(true),
        is_faculty_advisor: p.is_faculty_advisor.unwrap_or(false),
        bio: validate_optional_str(p.bio.as_deref(), "bio", 5000)?,
        linkedin_url: validate_optional_safe_url(p.linkedin_url.as_deref(), "linkedin_url", 2000)?,
        email,
        display_order: p.display_order.unwrap_or(0),
    })
}

#[axum::debug_handler]
async fn list_team(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<TeamMember>>, AppError> {
    let rows = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, role, role_responsibility, photo_url, batch, tenure_year, is_current, is_faculty_advisor, bio, linkedin_url, email, display_order FROM team_members ORDER BY display_order ASC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_team_member(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateTeamMember>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_team_member(&p)?;
    sqlx::query("INSERT INTO team_members (name,role,role_responsibility,photo_url,batch,tenure_year,is_current,is_faculty_advisor,bio,linkedin_url,email,display_order) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)")
        .bind(&v.name)
        .bind(&v.role)
        .bind(&v.role_responsibility)
        .bind(&v.photo_url)
        .bind(&v.batch)
        .bind(&v.tenure_year)
        .bind(v.is_current)
        .bind(v.is_faculty_advisor)
        .bind(&v.bio)
        .bind(&v.linkedin_url)
        .bind(&v.email)
        .bind(v.display_order)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_team_member(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateTeamMember>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_team_member(&p)?;
    let res = sqlx::query("UPDATE team_members SET name=?1,role=?2,role_responsibility=?3,photo_url=?4,batch=?5,tenure_year=?6,is_current=?7,is_faculty_advisor=?8,bio=?9,linkedin_url=?10,email=?11,display_order=?12 WHERE id=?13")
        .bind(&v.name)
        .bind(&v.role)
        .bind(&v.role_responsibility)
        .bind(&v.photo_url)
        .bind(&v.batch)
        .bind(&v.tenure_year)
        .bind(v.is_current)
        .bind(v.is_faculty_advisor)
        .bind(&v.bio)
        .bind(&v.linkedin_url)
        .bind(&v.email)
        .bind(v.display_order)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_team_member(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM team_members WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Gallery CRUD
// ---------------------------------------------------------------------------

struct ValidGalleryImage {
    image_url: String,
    caption: Option<String>,
    event_id: Option<i64>,
    event_name: Option<String>,
    year: Option<String>,
}

fn validate_gallery_image(p: &CreateGalleryImage) -> Result<ValidGalleryImage, AppError> {
    Ok(ValidGalleryImage {
        image_url: validate_safe_url(&p.image_url, "image_url", 2000)?,
        caption: validate_optional_str(p.caption.as_deref(), "caption", 500)?,
        event_id: p.event_id,
        event_name: validate_optional_str(p.event_name.as_deref(), "event_name", 300)?,
        year: validate_optional_str(p.year.as_deref(), "year", 20)?,
    })
}

#[axum::debug_handler]
async fn list_gallery(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<GalleryImage>>, AppError> {
    let rows = sqlx::query_as::<_, GalleryImage>(
        "SELECT id, image_url, caption, event_id, event_name, year, created_at FROM gallery_images ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

async fn ensure_event_exists(pool: &SqlitePool, event_id: Option<i64>) -> Result<(), AppError> {
    if let Some(eid) = event_id {
        let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM events WHERE id = ?1")
            .bind(eid)
            .fetch_optional(pool)
            .await?;
        if exists.is_none() {
            return Err(AppError::BadRequest(
                "Referenced event_id does not exist".into(),
            ));
        }
    }
    Ok(())
}

#[axum::debug_handler]
async fn create_gallery_image(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateGalleryImage>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_gallery_image(&p)?;
    ensure_event_exists(&pool, v.event_id).await?;
    sqlx::query("INSERT INTO gallery_images (image_url,caption,event_id,event_name,year) VALUES (?1,?2,?3,?4,?5)")
        .bind(&v.image_url)
        .bind(&v.caption)
        .bind(v.event_id)
        .bind(&v.event_name)
        .bind(&v.year)
        .execute(&pool)
        .await
        .map_err(|e| {
            if is_foreign_key_violation(&e) {
                AppError::BadRequest("Referenced event_id does not exist".into())
            } else {
                AppError::from(e)
            }
        })?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_gallery_image(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateGalleryImage>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_gallery_image(&p)?;
    ensure_event_exists(&pool, v.event_id).await?;
    let res = sqlx::query("UPDATE gallery_images SET image_url=?1,caption=?2,event_id=?3,event_name=?4,year=?5 WHERE id=?6")
        .bind(&v.image_url)
        .bind(&v.caption)
        .bind(v.event_id)
        .bind(&v.event_name)
        .bind(&v.year)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            if is_foreign_key_violation(&e) {
                AppError::BadRequest("Referenced event_id does not exist".into())
            } else {
                AppError::from(e)
            }
        })?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_gallery_image(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM gallery_images WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Posts CRUD
// ---------------------------------------------------------------------------

struct ValidPost {
    title: String,
    slug: String,
    excerpt: Option<String>,
    body_markdown: Option<String>,
    author_name: Option<String>,
    author_role: Option<String>,
    author_batch: Option<String>,
    cover_image_url: Option<String>,
    category: String,
    material_formula: Option<String>,
    crystal_structure: Option<String>,
    read_time_minutes: i64,
    is_featured: bool,
}

fn validate_post(p: &CreatePost) -> Result<ValidPost, AppError> {
    let title = validate_required_str(&p.title, "title", 300)?;
    let slug = match p.slug.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => slugify(s),
        None => slugify(&title),
    };
    let read_time = p.read_time_minutes.unwrap_or(5);
    if !(1..=300).contains(&read_time) {
        return Err(AppError::BadRequest(
            "read_time_minutes must be between 1 and 300".into(),
        ));
    }
    Ok(ValidPost {
        title,
        slug,
        excerpt: validate_optional_str(p.excerpt.as_deref(), "excerpt", 2000)?,
        body_markdown: validate_optional_str(p.body_markdown.as_deref(), "body_markdown", 100_000)?,
        author_name: validate_optional_str(p.author_name.as_deref(), "author_name", 200)?,
        author_role: validate_optional_str(p.author_role.as_deref(), "author_role", 200)?,
        author_batch: validate_optional_str(p.author_batch.as_deref(), "author_batch", 50)?,
        cover_image_url: validate_optional_safe_url(
            p.cover_image_url.as_deref(),
            "cover_image_url",
            2000,
        )?,
        category: validate_enum(
            p.category.as_deref(),
            "Explainer",
            POST_CATEGORIES,
            "category",
        )?,
        material_formula: validate_optional_str(
            p.material_formula.as_deref(),
            "material_formula",
            200,
        )?,
        crystal_structure: validate_optional_str(
            p.crystal_structure.as_deref(),
            "crystal_structure",
            200,
        )?,
        read_time_minutes: read_time,
        is_featured: p.is_featured.unwrap_or(false),
    })
}

#[axum::debug_handler]
async fn list_posts(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Post>>, AppError> {
    let rows = sqlx::query_as::<_, Post>(
        "SELECT id, title, slug, excerpt, body_markdown, author_name, author_role, author_batch, cover_image_url, category, material_formula, crystal_structure, read_time_minutes, is_featured, published_at FROM posts ORDER BY published_at DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_post(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreatePost>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_post(&p)?;
    let res = sqlx::query("INSERT INTO posts (title,slug,excerpt,body_markdown,author_name,author_role,author_batch,cover_image_url,category,material_formula,crystal_structure,read_time_minutes,is_featured) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)")
        .bind(&v.title)
        .bind(&v.slug)
        .bind(&v.excerpt)
        .bind(&v.body_markdown)
        .bind(&v.author_name)
        .bind(&v.author_role)
        .bind(&v.author_batch)
        .bind(&v.cover_image_url)
        .bind(&v.category)
        .bind(&v.material_formula)
        .bind(&v.crystal_structure)
        .bind(v.read_time_minutes)
        .bind(v.is_featured)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => Ok(Json(DeleteResponse {
            message: "Created".into(),
        })),
        Err(err) if is_unique_violation(&err) => Err(AppError::Conflict(
            "An article with this title or slug already exists".into(),
        )),
        Err(err) => Err(AppError::Database(err)),
    }
}

#[axum::debug_handler]
async fn update_post(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreatePost>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_post(&p)?;
    let res = sqlx::query("UPDATE posts SET title=?1,slug=?2,excerpt=?3,body_markdown=?4,author_name=?5,author_role=?6,author_batch=?7,cover_image_url=?8,category=?9,material_formula=?10,crystal_structure=?11,read_time_minutes=?12,is_featured=?13 WHERE id=?14")
        .bind(&v.title)
        .bind(&v.slug)
        .bind(&v.excerpt)
        .bind(&v.body_markdown)
        .bind(&v.author_name)
        .bind(&v.author_role)
        .bind(&v.author_batch)
        .bind(&v.cover_image_url)
        .bind(&v.category)
        .bind(&v.material_formula)
        .bind(&v.crystal_structure)
        .bind(v.read_time_minutes)
        .bind(v.is_featured)
        .bind(id)
        .execute(&pool)
        .await;

    let res = match res {
        Ok(r) => r,
        Err(err) if is_unique_violation(&err) => {
            return Err(AppError::Conflict(
                "An article with this title or slug already exists".into(),
            ));
        }
        Err(err) => return Err(AppError::Database(err)),
    };
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_post(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM posts WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Resources CRUD
// ---------------------------------------------------------------------------

struct ValidResource {
    title: String,
    url: String,
    category: String,
    software_tag: Option<String>,
    description: Option<String>,
    is_pinned: bool,
}

fn validate_resource(p: &CreateResource) -> Result<ValidResource, AppError> {
    Ok(ValidResource {
        title: validate_required_str(&p.title, "title", 300)?,
        url: validate_safe_url(&p.url, "url", 2000)?,
        category: validate_enum(
            p.category.as_deref(),
            "Software",
            RESOURCE_CATEGORIES,
            "category",
        )?,
        software_tag: validate_optional_str(p.software_tag.as_deref(), "software_tag", 100)?,
        description: validate_optional_str(p.description.as_deref(), "description", 5000)?,
        is_pinned: p.is_pinned.unwrap_or(false),
    })
}

#[axum::debug_handler]
async fn list_resources(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Resource>>, AppError> {
    let rows = sqlx::query_as::<_, Resource>(
        "SELECT id, title, url, category, software_tag, description, is_pinned FROM resources ORDER BY is_pinned DESC, id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_resource(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateResource>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_resource(&p)?;
    sqlx::query("INSERT INTO resources (title,url,category,software_tag,description,is_pinned) VALUES (?1,?2,?3,?4,?5,?6)")
        .bind(&v.title)
        .bind(&v.url)
        .bind(&v.category)
        .bind(&v.software_tag)
        .bind(&v.description)
        .bind(v.is_pinned)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_resource(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateResource>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_resource(&p)?;
    let res = sqlx::query("UPDATE resources SET title=?1,url=?2,category=?3,software_tag=?4,description=?5,is_pinned=?6 WHERE id=?7")
        .bind(&v.title)
        .bind(&v.url)
        .bind(&v.category)
        .bind(&v.software_tag)
        .bind(&v.description)
        .bind(v.is_pinned)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_resource(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM resources WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Opportunities CRUD
// ---------------------------------------------------------------------------

struct ValidOpportunity {
    title: String,
    organization: String,
    opportunity_type: String,
    location: Option<String>,
    deadline: Option<String>,
    apply_url: Option<String>,
    description: Option<String>,
    posted_by_alumnus: Option<String>,
    is_active: bool,
}

fn validate_opportunity(p: &CreateOpportunity) -> Result<ValidOpportunity, AppError> {
    Ok(ValidOpportunity {
        title: validate_required_str(&p.title, "title", 300)?,
        organization: validate_required_str(&p.organization, "organization", 300)?,
        opportunity_type: validate_enum(
            p.opportunity_type.as_deref(),
            "Internship",
            OPPORTUNITY_TYPES,
            "opportunity_type",
        )?,
        location: validate_optional_str(p.location.as_deref(), "location", 200)?,
        deadline: validate_optional_iso_date(p.deadline.as_deref(), "deadline")?,
        apply_url: validate_optional_safe_url(p.apply_url.as_deref(), "apply_url", 2000)?,
        description: validate_optional_str(p.description.as_deref(), "description", 10000)?,
        posted_by_alumnus: validate_optional_str(
            p.posted_by_alumnus.as_deref(),
            "posted_by_alumnus",
            200,
        )?,
        is_active: p.is_active.unwrap_or(true),
    })
}

#[axum::debug_handler]
async fn list_opportunities(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Opportunity>>, AppError> {
    let rows = sqlx::query_as::<_, Opportunity>(
        "SELECT id, title, organization, opportunity_type, location, deadline, apply_url, description, posted_by_alumnus, is_active, created_at FROM opportunities ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_opportunity(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateOpportunity>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_opportunity(&p)?;
    sqlx::query("INSERT INTO opportunities (title,organization,opportunity_type,location,deadline,apply_url,description,posted_by_alumnus,is_active) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")
        .bind(&v.title)
        .bind(&v.organization)
        .bind(&v.opportunity_type)
        .bind(&v.location)
        .bind(&v.deadline)
        .bind(&v.apply_url)
        .bind(&v.description)
        .bind(&v.posted_by_alumnus)
        .bind(v.is_active)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_opportunity(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateOpportunity>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_opportunity(&p)?;
    let res = sqlx::query("UPDATE opportunities SET title=?1,organization=?2,opportunity_type=?3,location=?4,deadline=?5,apply_url=?6,description=?7,posted_by_alumnus=?8,is_active=?9 WHERE id=?10")
        .bind(&v.title)
        .bind(&v.organization)
        .bind(&v.opportunity_type)
        .bind(&v.location)
        .bind(&v.deadline)
        .bind(&v.apply_url)
        .bind(&v.description)
        .bind(&v.posted_by_alumnus)
        .bind(v.is_active)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_opportunity(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM opportunities WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Alumni CRUD
// ---------------------------------------------------------------------------

struct ValidAlumnus {
    name: String,
    graduating_batch: String,
    current_role: Option<String>,
    current_organization: Option<String>,
    industry_sector: Option<String>,
    location: Option<String>,
    photo_url: Option<String>,
    quote_testimonial: Option<String>,
    linkedin_url: Option<String>,
}

fn validate_alumnus(p: &CreateAlumnus) -> Result<ValidAlumnus, AppError> {
    Ok(ValidAlumnus {
        name: validate_required_str(&p.name, "name", 200)?,
        graduating_batch: validate_required_str(&p.graduating_batch, "graduating_batch", 50)?,
        current_role: validate_optional_str(p.current_role.as_deref(), "current_role", 200)?,
        current_organization: validate_optional_str(
            p.current_organization.as_deref(),
            "current_organization",
            300,
        )?,
        industry_sector: validate_optional_str(
            p.industry_sector.as_deref(),
            "industry_sector",
            200,
        )?,
        location: validate_optional_str(p.location.as_deref(), "location", 200)?,
        photo_url: validate_optional_safe_url(p.photo_url.as_deref(), "photo_url", 2000)?,
        quote_testimonial: validate_optional_str(
            p.quote_testimonial.as_deref(),
            "quote_testimonial",
            5000,
        )?,
        linkedin_url: validate_optional_safe_url(p.linkedin_url.as_deref(), "linkedin_url", 2000)?,
    })
}

#[axum::debug_handler]
async fn list_alumni(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Alumnus>>, AppError> {
    let rows = sqlx::query_as::<_, Alumnus>(
        "SELECT id, name, graduating_batch, current_role, current_organization, industry_sector, location, photo_url, quote_testimonial, linkedin_url FROM alumni ORDER BY graduating_batch DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_alumnus(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateAlumnus>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_alumnus(&p)?;
    sqlx::query("INSERT INTO alumni (name,graduating_batch,current_role,current_organization,industry_sector,location,photo_url,quote_testimonial,linkedin_url) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")
        .bind(&v.name)
        .bind(&v.graduating_batch)
        .bind(&v.current_role)
        .bind(&v.current_organization)
        .bind(&v.industry_sector)
        .bind(&v.location)
        .bind(&v.photo_url)
        .bind(&v.quote_testimonial)
        .bind(&v.linkedin_url)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_alumnus(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateAlumnus>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_alumnus(&p)?;
    let res = sqlx::query("UPDATE alumni SET name=?1,graduating_batch=?2,current_role=?3,current_organization=?4,industry_sector=?5,location=?6,photo_url=?7,quote_testimonial=?8,linkedin_url=?9 WHERE id=?10")
        .bind(&v.name)
        .bind(&v.graduating_batch)
        .bind(&v.current_role)
        .bind(&v.current_organization)
        .bind(&v.industry_sector)
        .bind(&v.location)
        .bind(&v.photo_url)
        .bind(&v.quote_testimonial)
        .bind(&v.linkedin_url)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_alumnus(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM alumni WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Projects CRUD
// ---------------------------------------------------------------------------

struct ValidProject {
    title: String,
    students: Option<String>,
    batch: String,
    supervisors: Option<String>,
    project_type: String,
    r#abstract: Option<String>,
    image_url: Option<String>,
    paper_or_repo_url: Option<String>,
    tags: Option<String>,
}

fn validate_project(p: &CreateProject) -> Result<ValidProject, AppError> {
    Ok(ValidProject {
        title: validate_required_str(&p.title, "title", 300)?,
        students: validate_optional_str(p.students.as_deref(), "students", 500)?,
        batch: validate_required_str(&p.batch, "batch", 50)?,
        supervisors: validate_optional_str(p.supervisors.as_deref(), "supervisors", 500)?,
        project_type: validate_enum(
            p.project_type.as_deref(),
            "FYP",
            PROJECT_TYPES,
            "project_type",
        )?,
        r#abstract: validate_optional_str(p.r#abstract.as_deref(), "abstract", 10000)?,
        image_url: validate_optional_safe_url(p.image_url.as_deref(), "image_url", 2000)?,
        paper_or_repo_url: validate_optional_safe_url(
            p.paper_or_repo_url.as_deref(),
            "paper_or_repo_url",
            2000,
        )?,
        tags: validate_optional_str(p.tags.as_deref(), "tags", 500)?,
    })
}

#[axum::debug_handler]
async fn list_projects(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Project>>, AppError> {
    let rows = sqlx::query_as::<_, Project>(
        "SELECT id, title, students, batch, supervisors, project_type, abstract, image_url, paper_or_repo_url, tags FROM projects ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_project(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateProject>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_project(&p)?;
    sqlx::query("INSERT INTO projects (title,students,batch,supervisors,project_type,abstract,image_url,paper_or_repo_url,tags) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")
        .bind(&v.title)
        .bind(&v.students)
        .bind(&v.batch)
        .bind(&v.supervisors)
        .bind(&v.project_type)
        .bind(&v.r#abstract)
        .bind(&v.image_url)
        .bind(&v.paper_or_repo_url)
        .bind(&v.tags)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_project(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateProject>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_project(&p)?;
    let res = sqlx::query("UPDATE projects SET title=?1,students=?2,batch=?3,supervisors=?4,project_type=?5,abstract=?6,image_url=?7,paper_or_repo_url=?8,tags=?9 WHERE id=?10")
        .bind(&v.title)
        .bind(&v.students)
        .bind(&v.batch)
        .bind(&v.supervisors)
        .bind(&v.project_type)
        .bind(&v.r#abstract)
        .bind(&v.image_url)
        .bind(&v.paper_or_repo_url)
        .bind(&v.tags)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_project(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM projects WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Sponsors CRUD
// ---------------------------------------------------------------------------

struct ValidSponsor {
    name: String,
    logo_url: Option<String>,
    website_url: Option<String>,
    tier: String,
    since_year: Option<String>,
    display_order: i64,
}

fn validate_sponsor(p: &CreateSponsor) -> Result<ValidSponsor, AppError> {
    Ok(ValidSponsor {
        name: validate_required_str(&p.name, "name", 300)?,
        logo_url: validate_optional_safe_url(p.logo_url.as_deref(), "logo_url", 2000)?,
        website_url: validate_optional_safe_url(p.website_url.as_deref(), "website_url", 2000)?,
        tier: validate_enum(p.tier.as_deref(), "Partner", SPONSOR_TIERS, "tier")?,
        since_year: validate_optional_str(p.since_year.as_deref(), "since_year", 20)?,
        display_order: p.display_order.unwrap_or(0),
    })
}

#[axum::debug_handler]
async fn list_sponsors(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Sponsor>>, AppError> {
    let rows = sqlx::query_as::<_, Sponsor>(
        "SELECT id, name, logo_url, website_url, tier, since_year, display_order FROM sponsors ORDER BY display_order ASC, id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_sponsor(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateSponsor>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_sponsor(&p)?;
    sqlx::query("INSERT INTO sponsors (name,logo_url,website_url,tier,since_year,display_order) VALUES (?1,?2,?3,?4,?5,?6)")
        .bind(&v.name)
        .bind(&v.logo_url)
        .bind(&v.website_url)
        .bind(&v.tier)
        .bind(&v.since_year)
        .bind(v.display_order)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_sponsor(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateSponsor>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_sponsor(&p)?;
    let res = sqlx::query("UPDATE sponsors SET name=?1,logo_url=?2,website_url=?3,tier=?4,since_year=?5,display_order=?6 WHERE id=?7")
        .bind(&v.name)
        .bind(&v.logo_url)
        .bind(&v.website_url)
        .bind(&v.tier)
        .bind(&v.since_year)
        .bind(v.display_order)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_sponsor(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM sponsors WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// FAQs CRUD
// ---------------------------------------------------------------------------

struct ValidFaq {
    question: String,
    answer: String,
    category: String,
    display_order: i64,
}

fn validate_faq(p: &CreateFaq) -> Result<ValidFaq, AppError> {
    Ok(ValidFaq {
        question: validate_required_str(&p.question, "question", 1000)?,
        answer: validate_required_str(&p.answer, "answer", 10000)?,
        category: validate_enum(p.category.as_deref(), "General", FAQ_CATEGORIES, "category")?,
        display_order: p.display_order.unwrap_or(0),
    })
}

#[axum::debug_handler]
async fn list_faqs(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Faq>>, AppError> {
    let rows = sqlx::query_as::<_, Faq>(
        "SELECT id, question, answer, category, display_order FROM faqs ORDER BY display_order ASC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_faq(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateFaq>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_faq(&p)?;
    sqlx::query("INSERT INTO faqs (question,answer,category,display_order) VALUES (?1,?2,?3,?4)")
        .bind(&v.question)
        .bind(&v.answer)
        .bind(&v.category)
        .bind(v.display_order)
        .execute(&pool)
        .await?;
    Ok(Json(DeleteResponse {
        message: "Created".into(),
    }))
}

#[axum::debug_handler]
async fn update_faq(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<CreateFaq>,
) -> Result<Json<DeleteResponse>, AppError> {
    let v = validate_faq(&p)?;
    let res = sqlx::query(
        "UPDATE faqs SET question=?1,answer=?2,category=?3,display_order=?4 WHERE id=?5",
    )
    .bind(&v.question)
    .bind(&v.answer)
    .bind(&v.category)
    .bind(v.display_order)
    .bind(id)
    .execute(&pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_faq(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM faqs WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Membership Applications
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_membership(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<MembershipApplication>>, AppError> {
    let rows = sqlx::query_as::<_, MembershipApplication>(
        "SELECT id, full_name, roll_number, batch, department, email, phone, interests, why_join, status, created_at FROM membership_applications ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn update_membership_status(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<DeleteResponse>, AppError> {
    let status = body
        .get("status")
        .and_then(|s| s.as_str())
        .ok_or_else(|| AppError::BadRequest("status is required".into()))?;

    if !["Pending", "Approved", "Rejected"].contains(&status) {
        return Err(AppError::BadRequest(
            "Status must be one of: Pending, Approved, Rejected".into(),
        ));
    }

    let res = sqlx::query("UPDATE membership_applications SET status=?1 WHERE id=?2")
        .bind(status)
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated".into(),
    }))
}

#[axum::debug_handler]
async fn delete_membership(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM membership_applications WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Contact Messages
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_contacts(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<ContactMessage>>, AppError> {
    let rows = sqlx::query_as::<_, ContactMessage>(
        "SELECT id, name, email, subject, message, created_at FROM contact_messages ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn delete_contact(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM contact_messages WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

// ---------------------------------------------------------------------------
// Newsletter
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_newsletter(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<NewsletterSubscriber>>, AppError> {
    let rows = sqlx::query_as::<_, NewsletterSubscriber>(
        "SELECT id, email, subscribed_at FROM newsletter_subscribers ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn delete_subscriber(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM newsletter_subscribers WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted".into(),
    }))
}

#[axum::debug_handler]
async fn create_membership_from_admin(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateMembershipApplication>,
) -> Result<Json<DeleteResponse>, AppError> {
    let valid = validate_membership_payload(&payload)?;
    let res = sqlx::query(
        "INSERT INTO membership_applications (full_name, roll_number, batch, department, email, phone, interests, why_join) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&valid.full_name)
    .bind(&valid.roll_number)
    .bind(&valid.batch)
    .bind(&valid.department)
    .bind(&valid.email)
    .bind(&valid.phone)
    .bind(&valid.interests)
    .bind(&valid.why_join)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok(Json(DeleteResponse {
            message: "Created membership record".into(),
        })),
        Err(err) if is_unique_violation(&err) => Err(AppError::Conflict(
            "An application with this Roll Number or Email already exists".into(),
        )),
        Err(err) => Err(AppError::Database(err)),
    }
}

// ---------------------------------------------------------------------------
// Sections (Editable Content)
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_sections(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<SiteSection>>, AppError> {
    let rows = sqlx::query_as::<_, SiteSection>(
        "SELECT id, section_key, title, subtitle, content, badge, extra_json, updated_at FROM site_sections ORDER BY id ASC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_section(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateSiteSection>,
) -> Result<Json<DeleteResponse>, AppError> {
    let section_key = validate_required_str(&p.section_key, "section_key", 100)?;
    let title = validate_required_str(&p.title, "title", 300)?;
    let content = validate_required_str(&p.content, "content", 50000)?;
    let subtitle = validate_optional_str(p.subtitle.as_deref(), "subtitle", 500)?;
    let badge = validate_optional_str(p.badge.as_deref(), "badge", 100)?;
    let extra_json = validate_optional_str(p.extra_json.as_deref(), "extra_json", 50000)?;

    if let Some(ref json_str) = extra_json {
        if serde_json::from_str::<serde_json::Value>(json_str).is_err() {
            return Err(AppError::BadRequest("extra_json must be valid JSON".into()));
        }
    }

    let res = sqlx::query(
        "INSERT INTO site_sections (section_key, title, subtitle, content, badge, extra_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&section_key)
    .bind(&title)
    .bind(&subtitle)
    .bind(&content)
    .bind(&badge)
    .bind(&extra_json)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok(Json(DeleteResponse {
            message: "Created section".into(),
        })),
        Err(err) if is_unique_violation(&err) => Err(AppError::Conflict(
            "A section with this section_key already exists".into(),
        )),
        Err(err) => Err(AppError::Database(err)),
    }
}

#[axum::debug_handler]
async fn update_section(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<UpdateSiteSection>,
) -> Result<Json<DeleteResponse>, AppError> {
    let title = match p.title.as_deref() {
        None => None,
        Some(t) => Some(validate_required_str(t, "title", 300)?),
    };
    let content = match p.content.as_deref() {
        None => None,
        Some(c) => Some(validate_required_str(c, "content", 50000)?),
    };
    let subtitle = validate_clearable_str(p.subtitle.as_deref(), "subtitle", 500)?;
    let badge = validate_clearable_str(p.badge.as_deref(), "badge", 100)?;
    let extra_json = validate_clearable_str(p.extra_json.as_deref(), "extra_json", 50000)?;

    if let Some(ref json_str) = extra_json {
        if !json_str.is_empty() && serde_json::from_str::<serde_json::Value>(json_str).is_err() {
            return Err(AppError::BadRequest("extra_json must be valid JSON".into()));
        }
    }

    let res = sqlx::query(
        "UPDATE site_sections SET \
         title=COALESCE(?1,title), \
         subtitle=CASE WHEN ?2 = '' THEN NULL ELSE COALESCE(?2,subtitle) END, \
         content=COALESCE(?3,content), \
         badge=CASE WHEN ?4 = '' THEN NULL ELSE COALESCE(?4,badge) END, \
         extra_json=CASE WHEN ?5 = '' THEN NULL ELSE COALESCE(?5,extra_json) END, \
         updated_at=datetime('now') \
         WHERE id=?6",
    )
    .bind(&title)
    .bind(&subtitle)
    .bind(&content)
    .bind(&badge)
    .bind(&extra_json)
    .bind(id)
    .execute(&pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Updated section".into(),
    }))
}

#[axum::debug_handler]
async fn delete_section(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query("DELETE FROM site_sections WHERE id=?1")
        .bind(id)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResponse {
        message: "Deleted section".into(),
    }))
}
