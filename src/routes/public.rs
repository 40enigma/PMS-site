use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::*;
use crate::rate_limit::{ClientIp, RateLimitState};

/// Query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct EventFilter {
    pub category: Option<String>,
    pub is_past: Option<bool>,
    pub is_featured: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TeamFilter {
    pub tenure_year: Option<String>,
    pub is_current: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PostFilter {
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceFilter {
    pub category: Option<String>,
    pub software_tag: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpportunityFilter {
    pub opportunity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectFilter {
    pub project_type: Option<String>,
    pub batch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GalleryFilter {
    pub year: Option<String>,
    pub event_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CertificateQuery {
    pub code: String,
}

pub fn public_routes() -> Router<SqlitePool> {
    Router::new()
        // Announcements
        .route("/api/announcements", get(get_announcements))
        // Site Stats
        .route("/api/stats", get(get_stats))
        // Events
        .route("/api/events", get(get_events))
        .route("/api/events/{id}", get(get_event))
        // Event Registration (rate-limited)
        .route("/api/events/{id}/register", post(register_for_event))
        // Certificate Verification
        .route("/api/certificates/verify", get(verify_certificate))
        // Team Members
        .route("/api/team", get(get_team))
        // Gallery
        .route("/api/gallery", get(get_gallery))
        // Posts / Blog
        .route("/api/posts", get(get_posts))
        .route("/api/posts/{slug}", get(get_post_by_slug))
        // Resources
        .route("/api/resources", get(get_resources))
        // Opportunities
        .route("/api/opportunities", get(get_opportunities))
        // Alumni
        .route("/api/alumni", get(get_alumni))
        // Projects
        .route("/api/projects", get(get_projects))
        // Sponsors
        .route("/api/sponsors", get(get_sponsors))
        // FAQs
        .route("/api/faqs", get(get_faqs))
        // Membership Application (rate-limited)
        .route("/api/membership", post(submit_membership))
        // Contact Message (rate-limited)
        .route("/api/contact", post(submit_contact))
        // Newsletter (rate-limited)
        .route("/api/newsletter", post(subscribe_newsletter))
        // Site Sections (Editable Content)
        .route("/api/sections", get(get_sections))
}

// ---------------------------------------------------------------------------
// Public read endpoints
// ---------------------------------------------------------------------------

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/announcements",
    responses(
        (status = 200, description = "List active announcements", body = Vec<Announcement>)
    )
)]
pub async fn get_announcements(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Announcement>>, AppError> {
    let rows = sqlx::query_as::<_, Announcement>(
        "SELECT id, message, link_text, link_url, priority, is_active, created_at FROM announcements WHERE is_active = 1 ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/stats",
    responses(
        (status = 200, description = "Get public site statistics", body = SiteStats),
        (status = 404, description = "Stats not found")
    )
)]
pub async fn get_stats(State(pool): State<SqlitePool>) -> Result<Json<SiteStats>, AppError> {
    let row = sqlx::query_as::<_, SiteStats>(
        "SELECT id, active_members, events_held, years_active, industrial_partners, constitution_pdf_url, office_location, contact_email FROM site_stats WHERE id = 1"
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/events",
    responses(
        (status = 200, description = "List events", body = Vec<Event>)
    )
)]
pub async fn get_events(
    State(pool): State<SqlitePool>,
    Query(filter): Query<EventFilter>,
) -> Result<Json<Vec<Event>>, AppError> {
    let rows = sqlx::query_as::<_, Event>(
        "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events
         WHERE (?1 IS NULL OR category = ?1)
           AND (?2 IS NULL OR is_past = ?2)
           AND (?3 IS NULL OR is_featured = ?3)
         ORDER BY date DESC"
    )
    .bind(&filter.category)
    .bind(filter.is_past)
    .bind(filter.is_featured)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/events/{id}",
    params(
        ("id" = i64, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Get event by ID", body = Event),
        (status = 404, description = "Event not found")
    )
)]
pub async fn get_event(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Event>, AppError> {
    let row = sqlx::query_as::<_, Event>(
        "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events WHERE id = ?1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/api/events/{id}/register",
    params(
        ("id" = i64, Path, description = "Event ID")
    ),
    request_body = CreateEventRegistration,
    responses(
        (status = 200, description = "Registered successfully", body = EventRegistration),
        (status = 400, description = "Invalid registration input"),
        (status = 429, description = "Rate limit exceeded")
    )
)]
pub async fn register_for_event(
    State(pool): State<SqlitePool>,
    Path(event_id): Path<i64>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateEventRegistration>,
) -> Result<Json<EventRegistration>, AppError> {
    RateLimitState::global().check_submit(ip)?;

    // Input validation
    if payload.student_name.trim().is_empty() {
        return Err(AppError::BadRequest("student_name is required".into()));
    }
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    if payload.roll_number.trim().is_empty() {
        return Err(AppError::BadRequest("roll_number is required".into()));
    }
    if payload.batch.trim().is_empty() {
        return Err(AppError::BadRequest("batch is required".into()));
    }
    if payload.department.trim().is_empty() {
        return Err(AppError::BadRequest("department is required".into()));
    }

    // Check event exists
    let _event = sqlx::query("SELECT id FROM events WHERE id = ?1")
        .bind(event_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let cert_code = format!(
        "PMS-{}-{}",
        event_id,
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    );

    sqlx::query(
        "INSERT INTO event_registrations (event_id, student_name, roll_number, batch, department, email, phone, certificate_code) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )
    .bind(event_id)
    .bind(&payload.student_name)
    .bind(&payload.roll_number)
    .bind(&payload.batch)
    .bind(&payload.department)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&cert_code)
    .execute(&pool)
    .await?;

    let reg = sqlx::query_as::<_, EventRegistration>(
        "SELECT id, event_id, student_name, roll_number, batch, department, email, phone, certificate_code, created_at FROM event_registrations WHERE certificate_code = ?1"
    )
    .bind(&cert_code)
    .fetch_one(&pool)
    .await?;

    Ok(Json(reg))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/certificates/verify",
    params(
        ("code" = String, Query, description = "Certificate verification code")
    ),
    responses(
        (status = 200, description = "Verification outcome", body = CertificateVerification)
    )
)]
pub async fn verify_certificate(
    State(pool): State<SqlitePool>,
    Query(q): Query<CertificateQuery>,
) -> Result<Json<CertificateVerification>, AppError> {
    let code_trimmed = q.code.trim();
    let result = sqlx::query_as::<_, (String, String, String)>(
        "SELECT er.student_name, e.title, er.certificate_code FROM event_registrations er JOIN events e ON er.event_id = e.id WHERE er.certificate_code = ?1 COLLATE NOCASE"
    )
    .bind(code_trimmed)
    .fetch_optional(&pool)
    .await?;

    match result {
        Some((name, title, code)) => Ok(Json(CertificateVerification {
            valid: true,
            student_name: Some(name),
            event_title: Some(title),
            certificate_code: Some(code),
        })),
        None => Ok(Json(CertificateVerification {
            valid: false,
            student_name: None,
            event_title: None,
            certificate_code: None,
        })),
    }
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/team",
    responses(
        (status = 200, description = "List council and team members", body = Vec<TeamMember>)
    )
)]
pub async fn get_team(
    State(pool): State<SqlitePool>,
    Query(filter): Query<TeamFilter>,
) -> Result<Json<Vec<TeamMember>>, AppError> {
    let rows = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, role, role_responsibility, photo_url, batch, tenure_year, is_current, is_faculty_advisor, bio, linkedin_url, email, display_order FROM team_members
         WHERE (?1 IS NULL OR tenure_year = ?1)
           AND (?2 IS NULL OR is_current = ?2)
         ORDER BY display_order ASC"
    )
    .bind(&filter.tenure_year)
    .bind(filter.is_current)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/gallery",
    responses(
        (status = 200, description = "List gallery items", body = Vec<GalleryImage>)
    )
)]
pub async fn get_gallery(
    State(pool): State<SqlitePool>,
    Query(filter): Query<GalleryFilter>,
) -> Result<Json<Vec<GalleryImage>>, AppError> {
    let rows = sqlx::query_as::<_, GalleryImage>(
        "SELECT id, image_url, caption, event_id, event_name, year, created_at FROM gallery_images
         WHERE (?1 IS NULL OR year = ?1)
           AND (?2 IS NULL OR event_name = ?2)
         ORDER BY created_at DESC",
    )
    .bind(&filter.year)
    .bind(&filter.event_name)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/posts",
    responses(
        (status = 200, description = "List articles", body = Vec<Post>)
    )
)]
pub async fn get_posts(
    State(pool): State<SqlitePool>,
    Query(filter): Query<PostFilter>,
) -> Result<Json<Vec<Post>>, AppError> {
    let rows = sqlx::query_as::<_, Post>(
        "SELECT id, title, slug, excerpt, body_markdown, author_name, author_batch, cover_image_url, category, material_formula, crystal_structure, published_at FROM posts
         WHERE (?1 IS NULL OR category = ?1)
         ORDER BY published_at DESC"
    )
    .bind(&filter.category)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/posts/{slug}",
    params(
        ("slug" = String, Path, description = "Post slug")
    ),
    responses(
        (status = 200, description = "Get article details", body = Post),
        (status = 404, description = "Post not found")
    )
)]
pub async fn get_post_by_slug(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
) -> Result<Json<Post>, AppError> {
    let row = sqlx::query_as::<_, Post>(
        "SELECT id, title, slug, excerpt, body_markdown, author_name, author_batch, cover_image_url, category, material_formula, crystal_structure, published_at FROM posts WHERE slug = ?1"
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/resources",
    responses(
        (status = 200, description = "List toolkit resources", body = Vec<Resource>)
    )
)]
pub async fn get_resources(
    State(pool): State<SqlitePool>,
    Query(filter): Query<ResourceFilter>,
) -> Result<Json<Vec<Resource>>, AppError> {
    let rows = sqlx::query_as::<_, Resource>(
        "SELECT id, title, url, category, software_tag, description, is_pinned FROM resources
         WHERE (?1 IS NULL OR category = ?1)
           AND (?2 IS NULL OR software_tag = ?2)
         ORDER BY is_pinned DESC, title ASC",
    )
    .bind(&filter.category)
    .bind(&filter.software_tag)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/opportunities",
    responses(
        (status = 200, description = "List opportunities", body = Vec<Opportunity>)
    )
)]
pub async fn get_opportunities(
    State(pool): State<SqlitePool>,
    Query(filter): Query<OpportunityFilter>,
) -> Result<Json<Vec<Opportunity>>, AppError> {
    let rows = sqlx::query_as::<_, Opportunity>(
        "SELECT id, title, organization, opportunity_type, location, deadline, apply_url, description, posted_by_alumnus FROM opportunities
         WHERE (?1 IS NULL OR opportunity_type = ?1)
         ORDER BY deadline ASC"
    )
    .bind(&filter.opportunity_type)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/alumni",
    responses(
        (status = 200, description = "List alumni", body = Vec<Alumnus>)
    )
)]
pub async fn get_alumni(State(pool): State<SqlitePool>) -> Result<Json<Vec<Alumnus>>, AppError> {
    let rows = sqlx::query_as::<_, Alumnus>(
        "SELECT id, name, batch, current_position, organization, location, photo_url, quote_testimonial, linkedin_url, research_domain FROM alumni ORDER BY batch DESC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/projects",
    responses(
        (status = 200, description = "List student projects", body = Vec<Project>)
    )
)]
pub async fn get_projects(
    State(pool): State<SqlitePool>,
    Query(filter): Query<ProjectFilter>,
) -> Result<Json<Vec<Project>>, AppError> {
    let rows = sqlx::query_as::<_, Project>(
        "SELECT id, title, student_names, batch, supervisor_name, project_type, abstract_summary, image_url, paper_or_repo_url, tags FROM projects
         WHERE (?1 IS NULL OR project_type = ?1)
           AND (?2 IS NULL OR batch = ?2)
         ORDER BY batch DESC, title ASC"
    )
    .bind(&filter.project_type)
    .bind(&filter.batch)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/sponsors",
    responses(
        (status = 200, description = "List sponsors", body = Vec<Sponsor>)
    )
)]
pub async fn get_sponsors(State(pool): State<SqlitePool>) -> Result<Json<Vec<Sponsor>>, AppError> {
    let rows = sqlx::query_as::<_, Sponsor>(
        "SELECT id, name, logo_url, website_url, tier FROM sponsors ORDER BY CASE tier WHEN 'Platinum' THEN 1 WHEN 'Gold' THEN 2 WHEN 'Silver' THEN 3 ELSE 4 END"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/faqs",
    responses(
        (status = 200, description = "List FAQs", body = Vec<Faq>)
    )
)]
pub async fn get_faqs(State(pool): State<SqlitePool>) -> Result<Json<Vec<Faq>>, AppError> {
    let rows = sqlx::query_as::<_, Faq>(
        "SELECT id, question, answer, display_order FROM faqs ORDER BY display_order ASC",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// Submit endpoints (with validation)
// ---------------------------------------------------------------------------

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/api/membership",
    request_body = CreateMembershipApplication,
    responses(
        (status = 200, description = "Membership application submitted", body = MembershipApplication),
        (status = 400, description = "Invalid application data"),
        (status = 429, description = "Rate limit exceeded")
    )
)]
pub async fn submit_membership(
    State(pool): State<SqlitePool>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateMembershipApplication>,
) -> Result<Json<MembershipApplication>, AppError> {
    RateLimitState::global().check_submit(ip)?;

    // Input validation
    if payload.full_name.trim().is_empty() {
        return Err(AppError::BadRequest("full_name is required".into()));
    }
    if payload.full_name.len() > 200 {
        return Err(AppError::BadRequest("full_name is too long".into()));
    }
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    if payload.email.len() > 254 {
        return Err(AppError::BadRequest("email is too long".into()));
    }
    if payload.roll_number.trim().is_empty() {
        return Err(AppError::BadRequest("roll_number is required".into()));
    }
    if payload.batch.trim().is_empty() {
        return Err(AppError::BadRequest("batch is required".into()));
    }
    if payload.department.trim().is_empty() {
        return Err(AppError::BadRequest("department is required".into()));
    }

    let res = sqlx::query(
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

    let inserted_id = res.last_insert_rowid();

    let app = sqlx::query_as::<_, MembershipApplication>(
        "SELECT id, full_name, roll_number, batch, department, email, phone, interests, why_join, status, created_at FROM membership_applications WHERE id = ?1"
    )
    .bind(inserted_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(app))
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/api/contact",
    request_body = CreateContactMessage,
    responses(
        (status = 200, description = "Contact message sent", body = ContactMessage),
        (status = 400, description = "Invalid contact input"),
        (status = 429, description = "Rate limit exceeded")
    )
)]
pub async fn submit_contact(
    State(pool): State<SqlitePool>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateContactMessage>,
) -> Result<Json<ContactMessage>, AppError> {
    RateLimitState::global().check_submit(ip)?;

    // Input validation
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if payload.name.len() > 200 {
        return Err(AppError::BadRequest("name is too long".into()));
    }
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    if payload.email.len() > 254 {
        return Err(AppError::BadRequest("email is too long".into()));
    }
    if payload.message.trim().is_empty() {
        return Err(AppError::BadRequest("message is required".into()));
    }
    if payload.message.len() > 5000 {
        return Err(AppError::BadRequest(
            "message is too long (max 5000 chars)".into(),
        ));
    }

    let res = sqlx::query(
        "INSERT INTO contact_messages (name, email, subject, message) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&payload.subject)
    .bind(&payload.message)
    .execute(&pool)
    .await?;

    let inserted_id = res.last_insert_rowid();

    let msg = sqlx::query_as::<_, ContactMessage>(
        "SELECT id, name, email, subject, message, created_at FROM contact_messages WHERE id = ?1",
    )
    .bind(inserted_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(msg))
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/api/newsletter",
    request_body = CreateNewsletterSubscriber,
    responses(
        (status = 200, description = "Subscribed successfully", body = NewsletterSubscriber),
        (status = 400, description = "Invalid email address"),
        (status = 429, description = "Rate limit exceeded")
    )
)]
pub async fn subscribe_newsletter(
    State(pool): State<SqlitePool>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateNewsletterSubscriber>,
) -> Result<Json<NewsletterSubscriber>, AppError> {
    RateLimitState::global().check_submit(ip)?;

    // Input validation
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    if payload.email.len() > 254 {
        return Err(AppError::BadRequest("email is too long".into()));
    }

    sqlx::query("INSERT OR IGNORE INTO newsletter_subscribers (email) VALUES (?1)")
        .bind(&payload.email)
        .execute(&pool)
        .await?;

    let sub = sqlx::query_as::<_, NewsletterSubscriber>(
        "SELECT id, email, subscribed_at FROM newsletter_subscribers WHERE email = ?1",
    )
    .bind(&payload.email)
    .fetch_one(&pool)
    .await?;

    Ok(Json(sub))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/sections",
    responses(
        (status = 200, description = "List dynamic site sections", body = Vec<SiteSection>)
    )
)]
pub async fn get_sections(State(pool): State<SqlitePool>) -> Result<Json<Vec<SiteSection>>, AppError> {
    let rows = sqlx::query_as::<_, SiteSection>(
        "SELECT id, section_key, title, subtitle, content, badge, extra_json, updated_at FROM site_sections ORDER BY id ASC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}
