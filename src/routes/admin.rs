
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{Multipart, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use sqlx::SqlitePool;
use time::Duration;

use crate::auth::{create_token, jwt_cookie_name};
use crate::error::AppError;
use crate::models::*;
use crate::rate_limit::{ClientIp, RateLimitState};

// ---------------------------------------------------------------------------
// Allowed MIME types for file uploads
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
    ".exe", ".sh", ".bash", ".php", ".pl", ".cgi", ".js", ".py", ".rb", ".bat", ".cmd", ".ps1",
    ".vbs", ".jar", ".war", ".class", ".elf",
];

pub fn admin_routes() -> Router<SqlitePool> {
    Router::new()
        // Auth
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(verify_session))
        // Upload
        .route("/api/admin/upload", post(upload_file))
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
        .route("/api/admin/registrations/{id}", delete(delete_registration))
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

/// Returns `true` if the server is running in production mode.
fn is_production() -> bool {
    std::env::var("RUST_ENV").unwrap_or_default().to_lowercase() == "production"
}

/// Build the auth cookie with appropriate security attributes.
fn build_auth_cookie(token: String) -> Cookie<'static> {
    let secure = std::env::var("JWT_COOKIE_SECURE")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(is_production());

    let mut cookie = Cookie::new(jwt_cookie_name(), token);
    cookie.set_http_only(true);
    cookie.set_secure(secure);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(Duration::hours(24));
    cookie
}

#[axum::debug_handler]
pub async fn login(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    ClientIp(ip): ClientIp,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
    // Enforce rate limiting per client IP
    RateLimitState::global().check_login(ip)?;

    // Basic input validation
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::BadRequest(
            "Email and password are required".into(),
        ));
    }
    if payload.email.len() > 254 {
        return Err(AppError::BadRequest("Email too long".into()));
    }

    let user = sqlx::query_as::<_, AdminUser>(
        "SELECT id, email, password_hash, name FROM admin_users WHERE email = ?1",
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("Hash parse error".into()))?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized)?;

    let token = create_token(user.id, &user.email)?;

    // Set httpOnly cookie
    let cookie = build_auth_cookie(token.clone());
    let updated_jar = jar.add(cookie);

    tracing::info!(user_id = user.id, email = %user.email, "Admin login successful");

    Ok((
        updated_jar,
        Json(LoginResponse {
            token,
            name: user.name,
            email: user.email,
        }),
    ))
}

#[axum::debug_handler]
pub async fn logout(jar: CookieJar) -> (CookieJar, Json<serde_json::Value>) {
    let removal = Cookie::build((jwt_cookie_name(), ""))
        .path("/")
        .max_age(Duration::seconds(0))
        .build();

    let updated_jar = jar.remove(removal);
    (
        updated_jar,
        Json(serde_json::json!({"message": "Logged out successfully"})),
    )
}

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
        token: String::new(),
        name: user.name,
        email: user.email,
    }))
}


// ---------------------------------------------------------------------------
// File Upload (hardened)
// ---------------------------------------------------------------------------

#[axum::debug_handler]
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
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("{}.bin", uuid::Uuid::new_v4()));

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
                    "File type not allowed: {}",
                    blocked
                )));
            }
        }

        let declared_ct = field.content_type().map(|s| s.to_string());

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        if data.len() > 20 * 1024 * 1024 {
            return Err(AppError::BadRequest("File exceeds 20MB limit".into()));
        }

        // Sniff the actual MIME type from magic bytes.
        let sniffed_mime = if let Some(t) = infer::get(&data) {
            t.mime_type()
        } else if lower.ends_with(".txt") && std::str::from_utf8(&data).is_ok() {
            "text/plain"
        } else {
            "application/octet-stream"
        };

        // Allow only the whitelisted MIME types.
        if !ALLOWED_MIME_TYPES.contains(&sniffed_mime) {
            return Err(AppError::BadRequest(format!(
                "File content type '{}' is not allowed. Allowed: {}",
                sniffed_mime,
                ALLOWED_MIME_TYPES.join(", ")
            )));
        }

        // Cross-check declared vs sniffed content type (informational log).
        if let Some(ref declared) = declared_ct {
            if declared != sniffed_mime {
                tracing::warn!(
                    declared = %declared,
                    sniffed = %sniffed_mime,
                    "Content-Type mismatch between declared and sniffed MIME"
                );
            }
        }

        // Generate a safe filename (UUID prefix + original name, no path components).
        let upload_directory = crate::upload_dir();
        let safe_name = format!("{}_{}", uuid::Uuid::new_v4(), file_name);
        let path = format!("{}/{}", upload_directory, safe_name);

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
            url: format!("/uploads/{}", safe_name),
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

#[axum::debug_handler]
async fn list_uploads(
    _claims: AdminClaims,
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
                    let size = meta.len();
                    files.push(UploadedFileItem {
                        url: format!("/uploads/{}", name),
                        filename: name,
                        size,
                    });
                }
            }
        }
    }
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(Json(files))
}

#[axum::debug_handler]
async fn delete_upload(
    _claims: AdminClaims,
    Path(filename): Path<String>,
) -> Result<Json<DeleteResponse>, AppError> {
    let safe_filename = std::path::Path::new(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::BadRequest("Invalid filename".into()))?;

    let path = format!("{}/{}", crate::upload_dir(), safe_filename);
    if !std::path::Path::new(&path).exists() {
        return Err(AppError::NotFound);
    }

    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(DeleteResponse {
        message: "File deleted successfully".into(),
    }))
}

// ---------------------------------------------------------------------------
// Announcements
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_announcements(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Announcement>>, AppError> {
    let rows = sqlx::query_as::<_, Announcement>(
        "SELECT id, message, link_text, link_url, priority, is_active, created_at FROM announcements ORDER BY id DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_announcement(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateAnnouncement>,
) -> Result<Json<DeleteResponse>, AppError> {
    // Validate
    if p.message.trim().is_empty() {
        return Err(AppError::BadRequest("message is required".into()));
    }
    if p.message.len() > 1000 {
        return Err(AppError::BadRequest(
            "message must be at most 1000 characters".into(),
        ));
    }

    sqlx::query("INSERT INTO announcements (message, link_text, link_url, priority, is_active) VALUES (?1,?2,?3,?4,?5)")
        .bind(&p.message).bind(&p.link_text).bind(&p.link_url)
        .bind(p.priority.as_deref().unwrap_or("info"))
        .bind(p.is_active.unwrap_or(true))
        .execute(&pool).await?;
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
    if p.message.trim().is_empty() {
        return Err(AppError::BadRequest("message is required".into()));
    }
    let res = sqlx::query("UPDATE announcements SET message=?1, link_text=?2, link_url=?3, priority=?4, is_active=?5 WHERE id=?6")
        .bind(&p.message).bind(&p.link_text).bind(&p.link_url)
        .bind(p.priority.as_deref().unwrap_or("info"))
        .bind(p.is_active.unwrap_or(true))
        .bind(id)
        .execute(&pool).await?;
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
        "SELECT id, active_members, events_held, years_active, industrial_partners, constitution_pdf_url, office_location, contact_email FROM site_stats WHERE id=1"
    ).fetch_optional(&pool).await?.ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

#[axum::debug_handler]
async fn update_stats(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<UpdateSiteStats>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query(
        "UPDATE site_stats SET \
         active_members=COALESCE(?1,active_members), \
         events_held=COALESCE(?2,events_held), \
         years_active=COALESCE(?3,years_active), \
         industrial_partners=COALESCE(?4,industrial_partners), \
         constitution_pdf_url=?5, \
         office_location=?6, \
         contact_email=?7 \
         WHERE id=1"
    )
        .bind(p.active_members).bind(p.events_held).bind(p.years_active).bind(p.industrial_partners)
        .bind(&p.constitution_pdf_url).bind(&p.office_location).bind(&p.contact_email)
        .execute(&pool).await?;
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

#[axum::debug_handler]
async fn list_events(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Event>>, AppError> {
    let rows = sqlx::query_as::<_, Event>(
        "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events ORDER BY date DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_event(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateEvent>,
) -> Result<Json<DeleteResponse>, AppError> {
    // Validation
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    if p.title.len() > 300 {
        return Err(AppError::BadRequest(
            "title must be at most 300 characters".into(),
        ));
    }
    if p.date.trim().is_empty() {
        return Err(AppError::BadRequest("date is required".into()));
    }

    let slug = p.slug.unwrap_or_else(|| slugify(&p.title));
    let res = sqlx::query("INSERT INTO events (title,slug,date,time_range,venue,category,short_description,full_summary,poster_url,registration_link,speaker_or_organizer,is_past,is_featured) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)")
        .bind(&p.title).bind(&slug).bind(&p.date).bind(&p.time_range).bind(&p.venue)
        .bind(p.category.as_deref().unwrap_or("Seminar"))
        .bind(&p.short_description).bind(&p.full_summary).bind(&p.poster_url)
        .bind(&p.registration_link).bind(&p.speaker_or_organizer)
        .bind(p.is_past.unwrap_or(false)).bind(p.is_featured.unwrap_or(false))
        .execute(&pool).await;

    match res {
        Ok(_) => Ok(Json(DeleteResponse { message: "Created".into() })),
        Err(err) if is_unique_violation(&err) => {
            Err(AppError::Conflict("An event with this title or slug already exists".into()))
        }
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
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let slug = p.slug.unwrap_or_else(|| slugify(&p.title));
    let res = sqlx::query("UPDATE events SET title=?1,slug=?2,date=?3,time_range=?4,venue=?5,category=?6,short_description=?7,full_summary=?8,poster_url=?9,registration_link=?10,speaker_or_organizer=?11,is_past=?12,is_featured=?13 WHERE id=?14")
        .bind(&p.title).bind(&slug).bind(&p.date).bind(&p.time_range).bind(&p.venue)
        .bind(p.category.as_deref().unwrap_or("Seminar"))
        .bind(&p.short_description).bind(&p.full_summary).bind(&p.poster_url)
        .bind(&p.registration_link).bind(&p.speaker_or_organizer)
        .bind(p.is_past.unwrap_or(false)).bind(p.is_featured.unwrap_or(false))
        .bind(id)
        .execute(&pool).await;

    let res = match res {
        Ok(r) => r,
        Err(err) if is_unique_violation(&err) => {
            return Err(AppError::Conflict("An event with this title or slug already exists".into()));
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
// Registrations (Read/Delete)
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_registrations(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<EventRegistration>>, AppError> {
    let rows = sqlx::query_as::<_, EventRegistration>(
        "SELECT id, event_id, student_name, roll_number, batch, department, email, phone, certificate_code, created_at FROM event_registrations ORDER BY id DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
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

#[axum::debug_handler]
async fn list_team(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<TeamMember>>, AppError> {
    let rows = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, role, role_responsibility, photo_url, batch, tenure_year, is_current, is_faculty_advisor, bio, linkedin_url, email, display_order FROM team_members ORDER BY display_order ASC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_team_member(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateTeamMember>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if p.role.trim().is_empty() {
        return Err(AppError::BadRequest("role is required".into()));
    }
    sqlx::query("INSERT INTO team_members (name,role,role_responsibility,photo_url,batch,tenure_year,is_current,is_faculty_advisor,bio,linkedin_url,email,display_order) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)")
        .bind(&p.name).bind(&p.role).bind(&p.role_responsibility).bind(&p.photo_url)
        .bind(&p.batch).bind(p.tenure_year.as_deref().unwrap_or("2025-2026"))
        .bind(p.is_current.unwrap_or(true)).bind(p.is_faculty_advisor.unwrap_or(false))
        .bind(&p.bio).bind(&p.linkedin_url).bind(&p.email).bind(p.display_order.unwrap_or(99))
        .execute(&pool).await?;
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
    if p.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let res = sqlx::query("UPDATE team_members SET name=?1,role=?2,role_responsibility=?3,photo_url=?4,batch=?5,tenure_year=?6,is_current=?7,is_faculty_advisor=?8,bio=?9,linkedin_url=?10,email=?11,display_order=?12 WHERE id=?13")
        .bind(&p.name).bind(&p.role).bind(&p.role_responsibility).bind(&p.photo_url)
        .bind(&p.batch).bind(p.tenure_year.as_deref().unwrap_or("2025-2026"))
        .bind(p.is_current.unwrap_or(true)).bind(p.is_faculty_advisor.unwrap_or(false))
        .bind(&p.bio).bind(&p.linkedin_url).bind(&p.email).bind(p.display_order.unwrap_or(99))
        .bind(id)
        .execute(&pool).await?;
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

#[axum::debug_handler]
async fn list_gallery(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<GalleryImage>>, AppError> {
    let rows = sqlx::query_as::<_, GalleryImage>(
        "SELECT id, image_url, caption, event_id, event_name, year, created_at FROM gallery_images ORDER BY id DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_gallery_image(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateGalleryImage>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.image_url.trim().is_empty() {
        return Err(AppError::BadRequest("image_url is required".into()));
    }
    sqlx::query("INSERT INTO gallery_images (image_url,caption,event_id,event_name,year) VALUES (?1,?2,?3,?4,?5)")
        .bind(&p.image_url).bind(&p.caption).bind(p.event_id).bind(&p.event_name).bind(&p.year)
        .execute(&pool).await?;
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
    let res = sqlx::query("UPDATE gallery_images SET image_url=?1,caption=?2,event_id=?3,event_name=?4,year=?5 WHERE id=?6")
        .bind(&p.image_url).bind(&p.caption).bind(p.event_id).bind(&p.event_name).bind(&p.year).bind(id)
        .execute(&pool).await?;
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

#[axum::debug_handler]
async fn list_posts(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Post>>, AppError> {
    let rows = sqlx::query_as::<_, Post>(
        "SELECT id, title, slug, excerpt, body_markdown, author_name, author_role, author_batch, cover_image_url, category, material_formula, crystal_structure, read_time_minutes, is_featured, published_at FROM posts ORDER BY published_at DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_post(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreatePost>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    if p.title.len() > 300 {
        return Err(AppError::BadRequest(
            "title must be at most 300 characters".into(),
        ));
    }
    let slug = p.slug.unwrap_or_else(|| slugify(&p.title));
    let res = sqlx::query("INSERT INTO posts (title,slug,excerpt,body_markdown,author_name,author_role,author_batch,cover_image_url,category,material_formula,crystal_structure,read_time_minutes,is_featured) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)")
        .bind(&p.title).bind(&slug).bind(&p.excerpt).bind(&p.body_markdown)
        .bind(&p.author_name).bind(&p.author_role).bind(&p.author_batch).bind(&p.cover_image_url)
        .bind(p.category.as_deref().unwrap_or("Explainer"))
        .bind(&p.material_formula).bind(&p.crystal_structure)
        .bind(p.read_time_minutes.unwrap_or(5))
        .bind(p.is_featured.unwrap_or(false))
        .execute(&pool).await;

    match res {
        Ok(_) => Ok(Json(DeleteResponse { message: "Created".into() })),
        Err(err) if is_unique_violation(&err) => {
            Err(AppError::Conflict("An article with this title or slug already exists".into()))
        }
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
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let slug = p.slug.unwrap_or_else(|| slugify(&p.title));
    let res = sqlx::query("UPDATE posts SET title=?1,slug=?2,excerpt=?3,body_markdown=?4,author_name=?5,author_role=?6,author_batch=?7,cover_image_url=?8,category=?9,material_formula=?10,crystal_structure=?11,read_time_minutes=?12,is_featured=?13 WHERE id=?14")
        .bind(&p.title).bind(&slug).bind(&p.excerpt).bind(&p.body_markdown)
        .bind(&p.author_name).bind(&p.author_role).bind(&p.author_batch).bind(&p.cover_image_url)
        .bind(p.category.as_deref().unwrap_or("Explainer"))
        .bind(&p.material_formula).bind(&p.crystal_structure)
        .bind(p.read_time_minutes.unwrap_or(5))
        .bind(p.is_featured.unwrap_or(false))
        .bind(id)
        .execute(&pool).await;

    let res = match res {
        Ok(r) => r,
        Err(err) if is_unique_violation(&err) => {
            return Err(AppError::Conflict("An article with this title or slug already exists".into()));
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

#[axum::debug_handler]
async fn list_resources(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Resource>>, AppError> {
    let rows = sqlx::query_as::<_, Resource>(
        "SELECT id, title, url, category, software_tag, description, is_pinned FROM resources ORDER BY is_pinned DESC, id DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_resource(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateResource>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    if p.url.trim().is_empty() {
        return Err(AppError::BadRequest("url is required".into()));
    }
    sqlx::query("INSERT INTO resources (title,url,category,software_tag,description,is_pinned) VALUES (?1,?2,?3,?4,?5,?6)")
        .bind(&p.title).bind(&p.url).bind(p.category.as_deref().unwrap_or("Software"))
        .bind(&p.software_tag).bind(&p.description).bind(p.is_pinned.unwrap_or(false))
        .execute(&pool).await?;
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
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let res = sqlx::query("UPDATE resources SET title=?1,url=?2,category=?3,software_tag=?4,description=?5,is_pinned=?6 WHERE id=?7")
        .bind(&p.title).bind(&p.url).bind(p.category.as_deref().unwrap_or("Software"))
        .bind(&p.software_tag).bind(&p.description).bind(p.is_pinned.unwrap_or(false)).bind(id)
        .execute(&pool).await?;
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

#[axum::debug_handler]
async fn list_opportunities(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Opportunity>>, AppError> {
    let rows = sqlx::query_as::<_, Opportunity>(
        "SELECT id, title, organization, opportunity_type, location, deadline, apply_url, description, posted_by_alumnus, is_active, created_at FROM opportunities ORDER BY id DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_opportunity(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateOpportunity>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    if p.organization.trim().is_empty() {
        return Err(AppError::BadRequest("organization is required".into()));
    }
    sqlx::query("INSERT INTO opportunities (title,organization,opportunity_type,location,deadline,apply_url,description,posted_by_alumnus,is_active) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")
        .bind(&p.title).bind(&p.organization).bind(p.opportunity_type.as_deref().unwrap_or("Internship"))
        .bind(&p.location).bind(&p.deadline).bind(&p.apply_url).bind(&p.description).bind(&p.posted_by_alumnus)
        .bind(p.is_active.unwrap_or(true))
        .execute(&pool).await?;
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
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let res = sqlx::query("UPDATE opportunities SET title=?1,organization=?2,opportunity_type=?3,location=?4,deadline=?5,apply_url=?6,description=?7,posted_by_alumnus=?8,is_active=?9 WHERE id=?10")
        .bind(&p.title).bind(&p.organization).bind(p.opportunity_type.as_deref().unwrap_or("Internship"))
        .bind(&p.location).bind(&p.deadline).bind(&p.apply_url).bind(&p.description).bind(&p.posted_by_alumnus)
        .bind(p.is_active.unwrap_or(true))
        .bind(id)
        .execute(&pool).await?;
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

#[axum::debug_handler]
async fn list_alumni(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Alumnus>>, AppError> {
    let rows = sqlx::query_as::<_, Alumnus>(
        "SELECT id, name, graduating_batch, current_role, current_organization, industry_sector, location, photo_url, quote_testimonial, linkedin_url FROM alumni ORDER BY graduating_batch DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_alumnus(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateAlumnus>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if p.graduating_batch.trim().is_empty() {
        return Err(AppError::BadRequest("graduating_batch is required".into()));
    }
    sqlx::query("INSERT INTO alumni (name,graduating_batch,current_role,current_organization,industry_sector,location,photo_url,quote_testimonial,linkedin_url) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")
        .bind(&p.name).bind(&p.graduating_batch).bind(&p.current_role).bind(&p.current_organization)
        .bind(&p.industry_sector).bind(&p.location).bind(&p.photo_url).bind(&p.quote_testimonial)
        .bind(&p.linkedin_url)
        .execute(&pool).await?;
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
    if p.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let res = sqlx::query("UPDATE alumni SET name=?1,graduating_batch=?2,current_role=?3,current_organization=?4,industry_sector=?5,location=?6,photo_url=?7,quote_testimonial=?8,linkedin_url=?9 WHERE id=?10")
        .bind(&p.name).bind(&p.graduating_batch).bind(&p.current_role).bind(&p.current_organization)
        .bind(&p.industry_sector).bind(&p.location).bind(&p.photo_url).bind(&p.quote_testimonial)
        .bind(&p.linkedin_url).bind(id)
        .execute(&pool).await?;
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

#[axum::debug_handler]
async fn list_projects(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Project>>, AppError> {
    let rows = sqlx::query_as::<_, Project>(
        "SELECT id, title, students, batch, supervisors, project_type, abstract, image_url, paper_or_repo_url, tags FROM projects ORDER BY id DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_project(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateProject>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    if p.batch.trim().is_empty() {
        return Err(AppError::BadRequest("batch is required".into()));
    }
    sqlx::query("INSERT INTO projects (title,students,batch,supervisors,project_type,abstract,image_url,paper_or_repo_url,tags) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")
        .bind(&p.title).bind(&p.students).bind(&p.batch).bind(&p.supervisors)
        .bind(p.project_type.as_deref().unwrap_or("FYP"))
        .bind(&p.r#abstract).bind(&p.image_url).bind(&p.paper_or_repo_url).bind(&p.tags)
        .execute(&pool).await?;
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
    if p.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let res = sqlx::query("UPDATE projects SET title=?1,students=?2,batch=?3,supervisors=?4,project_type=?5,abstract=?6,image_url=?7,paper_or_repo_url=?8,tags=?9 WHERE id=?10")
        .bind(&p.title).bind(&p.students).bind(&p.batch).bind(&p.supervisors)
        .bind(p.project_type.as_deref().unwrap_or("FYP"))
        .bind(&p.r#abstract).bind(&p.image_url).bind(&p.paper_or_repo_url).bind(&p.tags)
        .bind(id)
        .execute(&pool).await?;
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

#[axum::debug_handler]
async fn list_sponsors(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Sponsor>>, AppError> {
    let rows = sqlx::query_as::<_, Sponsor>(
        "SELECT id, name, logo_url, website_url, tier, since_year, display_order FROM sponsors ORDER BY display_order ASC, id DESC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_sponsor(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateSponsor>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    sqlx::query("INSERT INTO sponsors (name,logo_url,website_url,tier,since_year,display_order) VALUES (?1,?2,?3,?4,?5,?6)")
        .bind(&p.name)
        .bind(&p.logo_url)
        .bind(&p.website_url)
        .bind(p.tier.as_deref().unwrap_or("Partner"))
        .bind(&p.since_year)
        .bind(p.display_order.unwrap_or(0))
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
    if p.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let res = sqlx::query("UPDATE sponsors SET name=?1,logo_url=?2,website_url=?3,tier=?4,since_year=?5,display_order=?6 WHERE id=?7")
        .bind(&p.name)
        .bind(&p.logo_url)
        .bind(&p.website_url)
        .bind(p.tier.as_deref().unwrap_or("Partner"))
        .bind(&p.since_year)
        .bind(p.display_order.unwrap_or(0))
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

#[axum::debug_handler]
async fn list_faqs(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Faq>>, AppError> {
    let rows = sqlx::query_as::<_, Faq>(
        "SELECT id, question, answer, category, display_order FROM faqs ORDER BY display_order ASC",
    )
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
    if p.question.trim().is_empty() {
        return Err(AppError::BadRequest("question is required".into()));
    }
    if p.answer.trim().is_empty() {
        return Err(AppError::BadRequest("answer is required".into()));
    }
    sqlx::query("INSERT INTO faqs (question,answer,category,display_order) VALUES (?1,?2,?3,?4)")
        .bind(&p.question)
        .bind(&p.answer)
        .bind(p.category.as_deref().unwrap_or("General"))
        .bind(p.display_order.unwrap_or(99))
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
    if p.question.trim().is_empty() {
        return Err(AppError::BadRequest("question is required".into()));
    }
    let res = sqlx::query("UPDATE faqs SET question=?1,answer=?2,category=?3,display_order=?4 WHERE id=?5")
        .bind(&p.question)
        .bind(&p.answer)
        .bind(p.category.as_deref().unwrap_or("General"))
        .bind(p.display_order.unwrap_or(99))
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
) -> Result<Json<Vec<MembershipApplication>>, AppError> {
    let rows = sqlx::query_as::<_, MembershipApplication>(
        "SELECT id, full_name, roll_number, batch, department, email, phone, interests, why_join, status, created_at FROM membership_applications ORDER BY id DESC"
    ).fetch_all(&pool).await?;
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
) -> Result<Json<Vec<ContactMessage>>, AppError> {
    let rows = sqlx::query_as::<_, ContactMessage>(
        "SELECT id, name, email, subject, message, created_at FROM contact_messages ORDER BY id DESC"
    ).fetch_all(&pool).await?;
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
) -> Result<Json<Vec<NewsletterSubscriber>>, AppError> {
    let rows = sqlx::query_as::<_, NewsletterSubscriber>(
        "SELECT id, email, subscribed_at FROM newsletter_subscribers ORDER BY id DESC",
    )
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
    if payload.full_name.trim().is_empty() {
        return Err(AppError::BadRequest("full_name is required".into()));
    }
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    sqlx::query(
        "INSERT INTO membership_applications (full_name, roll_number, batch, department, email, phone, interests, why_join) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )
    .bind(&payload.full_name)
    .bind(&payload.roll_number)
    .bind(&payload.batch)
    .bind(&payload.department)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.interests)
    .bind(&payload.why_join)
    .execute(&pool)
    .await?;
    Ok(Json(DeleteResponse {
        message: "Created membership record".into(),
    }))
}

// ---------------------------------------------------------------------------
// Sections (Editable Content)
// ---------------------------------------------------------------------------

#[axum::debug_handler]
async fn list_sections(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<SiteSection>>, AppError> {
    let rows = sqlx::query_as::<_, SiteSection>(
        "SELECT id, section_key, title, subtitle, content, badge, extra_json, updated_at FROM site_sections ORDER BY id ASC"
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
async fn create_section(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Json(p): Json<CreateSiteSection>,
) -> Result<Json<DeleteResponse>, AppError> {
    if p.section_key.trim().is_empty() {
        return Err(AppError::BadRequest("section_key is required".into()));
    }
    sqlx::query("INSERT INTO site_sections (section_key, title, subtitle, content, badge, extra_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .bind(&p.section_key)
        .bind(&p.title)
        .bind(&p.subtitle)
        .bind(&p.content)
        .bind(&p.badge)
        .bind(&p.extra_json)
        .execute(&pool).await?;
    Ok(Json(DeleteResponse {
        message: "Created section".into(),
    }))
}

#[axum::debug_handler]
async fn update_section(
    _claims: AdminClaims,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(p): Json<UpdateSiteSection>,
) -> Result<Json<DeleteResponse>, AppError> {
    let res = sqlx::query(
        "UPDATE site_sections SET title=COALESCE(?1,title), subtitle=?2, content=COALESCE(?3,content), badge=?4, extra_json=?5, updated_at=datetime('now') WHERE id=?6"
    )
    .bind(&p.title)
    .bind(&p.subtitle)
    .bind(&p.content)
    .bind(&p.badge)
    .bind(&p.extra_json)
    .bind(id)
    .execute(&pool).await?;
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

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn slugify(s: &str) -> String {
    let slug = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    if slug.is_empty() {
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    } else {
        slug
    }
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.is_unique_violation() {
            return true;
        }
    }
    err.to_string().contains("UNIQUE constraint failed")
}
