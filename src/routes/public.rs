use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{
    Alumnus, Announcement, CertificateVerification, ContactMessage, CreateContactMessage,
    CreateEventRegistration, CreateMembershipApplication, CreateNewsletterSubscriber, Event,
    EventRegistration, Faq, GalleryImage, MembershipApplication, NewsletterSubscriber, Opportunity,
    PaginationParams, Post, Project, Resource, SiteSection, SiteStats, Sponsor, TeamMember,
};
use crate::rate_limit::{ClientIp, RateLimitState};
use crate::util::{
    is_unique_violation, normalize_filter, today_date_string, validate_email,
    validate_optional_str, validate_required_str,
};

/// Query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct EventFilter {
    pub category: Option<String>,
    pub is_past: Option<bool>,
    pub is_featured: Option<bool>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[derive(Debug, Deserialize)]
pub struct TeamFilter {
    pub tenure_year: Option<String>,
    pub is_current: Option<bool>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[derive(Debug, Deserialize)]
pub struct PostFilter {
    pub category: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[derive(Debug, Deserialize)]
pub struct ResourceFilter {
    pub category: Option<String>,
    pub software_tag: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[derive(Debug, Deserialize)]
pub struct OpportunityFilter {
    pub opportunity_type: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[derive(Debug, Deserialize)]
pub struct ProjectFilter {
    pub project_type: Option<String>,
    pub batch: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[derive(Debug, Deserialize)]
pub struct GalleryFilter {
    pub year: Option<String>,
    pub event_name: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
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

/// List active announcements.
///
/// # Errors
/// Returns `AppError::Database` if the query fails.
#[axum::debug_handler]
pub async fn get_announcements(
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Announcement>>, AppError> {
    let rows = sqlx::query_as::<_, Announcement>(
        "SELECT id, message, link_text, link_url, priority, is_active, created_at FROM announcements WHERE is_active = 1 ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// Retrieve site statistics and public contact details.
///
/// # Errors
/// Returns `AppError::NotFound` or `AppError::Database`.
#[axum::debug_handler]
pub async fn get_stats(State(pool): State<SqlitePool>) -> Result<Json<SiteStats>, AppError> {
    let row = sqlx::query_as::<_, SiteStats>(
        "SELECT id, active_members, events_held, years_active, industrial_partners, constitution_pdf_url, office_location, contact_email FROM site_stats WHERE id = 1",
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

/// List events with optional category, featured, and past/upcoming filters.
///
/// # Errors
/// Returns `AppError::Database` if the query fails.
#[axum::debug_handler]
pub async fn get_events(
    State(pool): State<SqlitePool>,
    Query(filter): Query<EventFilter>,
) -> Result<Json<Vec<Event>>, AppError> {
    let today = today_date_string();
    let category = normalize_filter(filter.category);

    let mut rows = match filter.is_past {
        Some(true) => {
            sqlx::query_as::<_, Event>(
                "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events
                 WHERE (?1 IS NULL OR category = ?1)
                   AND (?2 IS NULL OR is_featured = ?2)
                   AND (is_past = 1 OR date < ?3)
                 ORDER BY date DESC
                 LIMIT ?4 OFFSET ?5",
            )
            .bind(&category)
            .bind(filter.is_featured)
            .bind(&today)
            .bind(filter.pagination.limit())
            .bind(filter.pagination.offset())
            .fetch_all(&pool)
            .await?
        }
        Some(false) => {
            sqlx::query_as::<_, Event>(
                "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events
                 WHERE (?1 IS NULL OR category = ?1)
                   AND (?2 IS NULL OR is_featured = ?2)
                   AND (is_past = 0 AND date >= ?3)
                 ORDER BY date ASC
                 LIMIT ?4 OFFSET ?5",
            )
            .bind(&category)
            .bind(filter.is_featured)
            .bind(&today)
            .bind(filter.pagination.limit())
            .bind(filter.pagination.offset())
            .fetch_all(&pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Event>(
                "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events
                 WHERE (?1 IS NULL OR category = ?1)
                   AND (?2 IS NULL OR is_featured = ?2)
                 ORDER BY CASE WHEN (is_past = 1 OR date < ?3) THEN 1 ELSE 0 END ASC,
                          CASE WHEN (is_past = 1 OR date < ?3) THEN '' ELSE date END ASC,
                          date DESC
                 LIMIT ?4 OFFSET ?5",
            )
            .bind(&category)
            .bind(filter.is_featured)
            .bind(&today)
            .bind(filter.pagination.limit())
            .bind(filter.pagination.offset())
            .fetch_all(&pool)
            .await?
        }
    };

    for row in &mut rows {
        row.is_past = row.is_past || row.date < today;
    }

    Ok(Json(rows))
}

/// Get a single event by ID.
///
/// # Errors
/// Returns `AppError::NotFound` or `AppError::Database`.
#[axum::debug_handler]
pub async fn get_event(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Event>, AppError> {
    let mut row = sqlx::query_as::<_, Event>(
        "SELECT id, title, slug, date, time_range, venue, category, short_description, full_summary, poster_url, registration_link, speaker_or_organizer, is_past, is_featured FROM events WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let today = today_date_string();
    row.is_past = row.is_past || row.date < today;

    Ok(Json(row))
}

fn generate_certificate_code(event_id: i64) -> String {
    let rand_hex = uuid::Uuid::new_v4().simple().to_string();
    format!("PMS-{event_id}-{}", rand_hex[..16].to_uppercase())
}

/// Register a student for an upcoming event.
///
/// # Errors
/// Returns `AppError::TooManyRequests`, `AppError::BadRequest`, `AppError::NotFound`,
/// `AppError::Conflict`, or `AppError::Database`.
#[axum::debug_handler]
pub async fn register_for_event(
    State(pool): State<SqlitePool>,
    Path(event_id): Path<i64>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateEventRegistration>,
) -> Result<Json<EventRegistration>, AppError> {
    RateLimitState::global().check_register(ip)?;

    let student_name = validate_required_str(&payload.student_name, "student_name", 200)?;
    let email = validate_email(&payload.email)?;
    let roll_number = validate_required_str(&payload.roll_number, "roll_number", 50)?;
    let batch = validate_required_str(&payload.batch, "batch", 50)?;
    let department = validate_required_str(&payload.department, "department", 200)?;
    let phone = validate_optional_str(payload.phone.as_deref(), "phone", 50)?;

    // Check event exists and is not past (both flag and date)
    let (is_past, date): (bool, String) =
        sqlx::query_as("SELECT is_past, date FROM events WHERE id = ?1")
            .bind(event_id)
            .fetch_optional(&pool)
            .await?
            .ok_or(AppError::NotFound)?;

    let today = today_date_string();
    if is_past || date < today {
        return Err(AppError::BadRequest(
            "Registration is closed for past events".into(),
        ));
    }

    let mut cert_code = String::new();
    for attempt in 0..3 {
        cert_code = generate_certificate_code(event_id);
        let insert_res = sqlx::query(
            "INSERT INTO event_registrations (event_id, student_name, roll_number, batch, department, email, phone, certificate_code, attended) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
        )
        .bind(event_id)
        .bind(&student_name)
        .bind(&roll_number)
        .bind(&batch)
        .bind(&department)
        .bind(&email)
        .bind(&phone)
        .bind(&cert_code)
        .execute(&pool)
        .await;

        match insert_res {
            Ok(_) => break,
            Err(err) if is_unique_violation(&err) => {
                if attempt < 2 && err.to_string().contains("certificate_code") {
                    continue;
                }
                return Err(AppError::Conflict(
                    "You are already registered for this event".into(),
                ));
            }
            Err(err) => return Err(AppError::Database(err)),
        }
    }

    let reg = sqlx::query_as::<_, EventRegistration>(
        "SELECT id, event_id, student_name, roll_number, batch, department, email, phone, certificate_code, attended, created_at FROM event_registrations WHERE certificate_code = ?1",
    )
    .bind(&cert_code)
    .fetch_one(&pool)
    .await?;

    Ok(Json(reg))
}

/// Verify a participation certificate code.
///
/// # Errors
/// Returns `AppError::TooManyRequests` or `AppError::Database`.
#[axum::debug_handler]
pub async fn verify_certificate(
    State(pool): State<SqlitePool>,
    ClientIp(ip): ClientIp,
    Query(q): Query<CertificateQuery>,
) -> Result<Json<CertificateVerification>, AppError> {
    RateLimitState::global().check_certificate_verify(ip)?;

    let code_trimmed = q.code.trim();
    if code_trimmed.is_empty() {
        return Ok(Json(CertificateVerification {
            valid: false,
            student_name: None,
            event_title: None,
            certificate_code: None,
            reason: Some("not_found".to_string()),
        }));
    }

    let result = sqlx::query_as::<_, (String, String, String, bool)>(
        "SELECT er.student_name, e.title, er.certificate_code, er.attended FROM event_registrations er JOIN events e ON er.event_id = e.id WHERE er.certificate_code = ?1 COLLATE NOCASE",
    )
    .bind(code_trimmed)
    .fetch_optional(&pool)
    .await?;

    match result {
        Some((name, title, code, true)) => Ok(Json(CertificateVerification {
            valid: true,
            student_name: Some(name),
            event_title: Some(title),
            certificate_code: Some(code),
            reason: None,
        })),
        Some((_, _, _, false)) => Ok(Json(CertificateVerification {
            valid: false,
            student_name: None,
            event_title: None,
            certificate_code: None,
            reason: Some("not_attended".to_string()),
        })),
        None => Ok(Json(CertificateVerification {
            valid: false,
            student_name: None,
            event_title: None,
            certificate_code: None,
            reason: Some("not_found".to_string()),
        })),
    }
}

/// List team members.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_team(
    State(pool): State<SqlitePool>,
    Query(filter): Query<TeamFilter>,
) -> Result<Json<Vec<TeamMember>>, AppError> {
    let tenure_year = normalize_filter(filter.tenure_year);
    let rows = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, role, role_responsibility, photo_url, batch, tenure_year, is_current, is_faculty_advisor, bio, linkedin_url, email, display_order FROM team_members
         WHERE (?1 IS NULL OR tenure_year = ?1)
           AND (?2 IS NULL OR is_current = ?2)
         ORDER BY display_order ASC
         LIMIT ?3 OFFSET ?4",
    )
    .bind(&tenure_year)
    .bind(filter.is_current)
    .bind(filter.pagination.limit())
    .bind(filter.pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// List gallery images.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_gallery(
    State(pool): State<SqlitePool>,
    Query(filter): Query<GalleryFilter>,
) -> Result<Json<Vec<GalleryImage>>, AppError> {
    let year = normalize_filter(filter.year);
    let event_name = normalize_filter(filter.event_name);
    let rows = sqlx::query_as::<_, GalleryImage>(
        "SELECT id, image_url, caption, event_id, event_name, year, created_at FROM gallery_images
         WHERE (?1 IS NULL OR year = ?1)
           AND (?2 IS NULL OR event_name = ?2)
         ORDER BY created_at DESC
         LIMIT ?3 OFFSET ?4",
    )
    .bind(&year)
    .bind(&event_name)
    .bind(filter.pagination.limit())
    .bind(filter.pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// List published posts (excluding `body_markdown` for bandwidth efficiency).
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_posts(
    State(pool): State<SqlitePool>,
    Query(filter): Query<PostFilter>,
) -> Result<Json<Vec<Post>>, AppError> {
    let category = normalize_filter(filter.category);
    let rows = sqlx::query_as::<_, Post>(
        "SELECT id, title, slug, excerpt, CAST(NULL AS TEXT) as body_markdown, author_name, author_role, author_batch, cover_image_url, category, material_formula, crystal_structure, read_time_minutes, is_featured, published_at FROM posts
         WHERE (?1 IS NULL OR category = ?1)
         ORDER BY published_at DESC
         LIMIT ?2 OFFSET ?3",
    )
    .bind(&category)
    .bind(filter.pagination.limit())
    .bind(filter.pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// Get full post by slug.
///
/// # Errors
/// Returns `AppError::NotFound` or `AppError::Database`.
#[axum::debug_handler]
pub async fn get_post_by_slug(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
) -> Result<Json<Post>, AppError> {
    let row = sqlx::query_as::<_, Post>(
        "SELECT id, title, slug, excerpt, body_markdown, author_name, author_role, author_batch, cover_image_url, category, material_formula, crystal_structure, read_time_minutes, is_featured, published_at FROM posts WHERE slug = ?1",
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

/// List academic resources.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_resources(
    State(pool): State<SqlitePool>,
    Query(filter): Query<ResourceFilter>,
) -> Result<Json<Vec<Resource>>, AppError> {
    let category = normalize_filter(filter.category);
    let software_tag = normalize_filter(filter.software_tag);
    let rows = sqlx::query_as::<_, Resource>(
        "SELECT id, title, url, category, software_tag, description, is_pinned FROM resources
         WHERE (?1 IS NULL OR category = ?1)
           AND (?2 IS NULL OR software_tag = ?2)
         ORDER BY is_pinned DESC, title ASC
         LIMIT ?3 OFFSET ?4",
    )
    .bind(&category)
    .bind(&software_tag)
    .bind(filter.pagination.limit())
    .bind(filter.pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// List active non-expired opportunities.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_opportunities(
    State(pool): State<SqlitePool>,
    Query(filter): Query<OpportunityFilter>,
) -> Result<Json<Vec<Opportunity>>, AppError> {
    let today = today_date_string();
    let opportunity_type = normalize_filter(filter.opportunity_type);
    let rows = sqlx::query_as::<_, Opportunity>(
        "SELECT id, title, organization, opportunity_type, location, deadline, apply_url, description, posted_by_alumnus, is_active, created_at FROM opportunities
         WHERE is_active = 1
           AND (?1 IS NULL OR opportunity_type = ?1)
           AND (deadline IS NULL OR deadline >= ?2)
         ORDER BY CASE WHEN deadline IS NULL THEN 1 ELSE 0 END ASC, deadline ASC
         LIMIT ?3 OFFSET ?4",
    )
    .bind(&opportunity_type)
    .bind(&today)
    .bind(filter.pagination.limit())
    .bind(filter.pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// List alumni profiles.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_alumni(
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

/// List student projects.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_projects(
    State(pool): State<SqlitePool>,
    Query(filter): Query<ProjectFilter>,
) -> Result<Json<Vec<Project>>, AppError> {
    let project_type = normalize_filter(filter.project_type);
    let batch = normalize_filter(filter.batch);
    let rows = sqlx::query_as::<_, Project>(
        "SELECT id, title, students, batch, supervisors, project_type, abstract, image_url, paper_or_repo_url, tags FROM projects
         WHERE (?1 IS NULL OR project_type = ?1)
           AND (?2 IS NULL OR batch = ?2)
         ORDER BY batch DESC, title ASC
         LIMIT ?3 OFFSET ?4",
    )
    .bind(&project_type)
    .bind(&batch)
    .bind(filter.pagination.limit())
    .bind(filter.pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// List sponsors.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_sponsors(
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Sponsor>>, AppError> {
    let rows = sqlx::query_as::<_, Sponsor>(
        "SELECT id, name, logo_url, website_url, tier, since_year, display_order FROM sponsors ORDER BY display_order ASC, CASE tier WHEN 'Platinum' THEN 1 WHEN 'Gold' THEN 2 WHEN 'Silver' THEN 3 ELSE 4 END LIMIT ?1 OFFSET ?2",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// List FAQs.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_faqs(
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

// ---------------------------------------------------------------------------
// Submit endpoints (with validation)
// ---------------------------------------------------------------------------

/// Validate and normalize a membership application payload.
///
/// # Errors
/// Returns `AppError::BadRequest` on invalid fields.
pub(crate) fn validate_membership_payload(
    payload: &CreateMembershipApplication,
) -> Result<ValidatedMembership, AppError> {
    Ok(ValidatedMembership {
        full_name: validate_required_str(&payload.full_name, "full_name", 200)?,
        roll_number: validate_required_str(&payload.roll_number, "roll_number", 50)?,
        batch: validate_required_str(&payload.batch, "batch", 50)?,
        department: validate_required_str(&payload.department, "department", 200)?,
        email: validate_email(&payload.email)?,
        phone: validate_optional_str(payload.phone.as_deref(), "phone", 50)?,
        interests: validate_optional_str(payload.interests.as_deref(), "interests", 2000)?,
        why_join: validate_optional_str(payload.why_join.as_deref(), "why_join", 2000)?,
    })
}

pub(crate) struct ValidatedMembership {
    pub full_name: String,
    pub roll_number: String,
    pub batch: String,
    pub department: String,
    pub email: String,
    pub phone: Option<String>,
    pub interests: Option<String>,
    pub why_join: Option<String>,
}

/// Submit a membership application.
///
/// # Errors
/// Returns `AppError::TooManyRequests`, `AppError::BadRequest`, `AppError::Conflict`, or `AppError::Database`.
#[axum::debug_handler]
pub async fn submit_membership(
    State(pool): State<SqlitePool>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateMembershipApplication>,
) -> Result<Json<MembershipApplication>, AppError> {
    RateLimitState::global().check_membership(ip)?;

    let valid = validate_membership_payload(&payload)?;

    let insert_res = sqlx::query(
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

    let res = match insert_res {
        Ok(r) => r,
        Err(err) => {
            if is_unique_violation(&err) {
                return Err(AppError::Conflict(
                    "An application with this Roll Number or Email already exists".into(),
                ));
            }
            return Err(AppError::Database(err));
        }
    };

    let inserted_id = res.last_insert_rowid();

    let app = sqlx::query_as::<_, MembershipApplication>(
        "SELECT id, full_name, roll_number, batch, department, email, phone, interests, why_join, status, created_at FROM membership_applications WHERE id = ?1",
    )
    .bind(inserted_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(app))
}

/// Submit a contact message.
///
/// # Errors
/// Returns `AppError::TooManyRequests`, `AppError::BadRequest`, or `AppError::Database`.
#[axum::debug_handler]
pub async fn submit_contact(
    State(pool): State<SqlitePool>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateContactMessage>,
) -> Result<Json<ContactMessage>, AppError> {
    RateLimitState::global().check_contact(ip)?;

    let name = validate_required_str(&payload.name, "name", 200)?;
    let email = validate_email(&payload.email)?;
    let subject = validate_optional_str(payload.subject.as_deref(), "subject", 300)?;
    let message = validate_required_str(&payload.message, "message", 5000)?;

    let res = sqlx::query(
        "INSERT INTO contact_messages (name, email, subject, message) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&name)
    .bind(&email)
    .bind(&subject)
    .bind(&message)
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

/// Subscribe an email address to the newsletter.
///
/// # Errors
/// Returns `AppError::TooManyRequests`, `AppError::BadRequest`, or `AppError::Database`.
#[axum::debug_handler]
pub async fn subscribe_newsletter(
    State(pool): State<SqlitePool>,
    ClientIp(ip): ClientIp,
    Json(payload): Json<CreateNewsletterSubscriber>,
) -> Result<Json<NewsletterSubscriber>, AppError> {
    RateLimitState::global().check_newsletter(ip)?;

    let email = validate_email(&payload.email)?;

    sqlx::query("INSERT OR IGNORE INTO newsletter_subscribers (email) VALUES (?1)")
        .bind(&email)
        .execute(&pool)
        .await?;

    let sub = sqlx::query_as::<_, NewsletterSubscriber>(
        "SELECT id, email, subscribed_at FROM newsletter_subscribers WHERE email = ?1",
    )
    .bind(&email)
    .fetch_one(&pool)
    .await?;

    Ok(Json(sub))
}

/// List editable site sections.
///
/// # Errors
/// Returns `AppError::Database`.
#[axum::debug_handler]
pub async fn get_sections(
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
