use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Announcements
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Announcement {
    pub id: i64,
    pub message: String,
    pub link_text: Option<String>,
    pub link_url: Option<String>,
    pub priority: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAnnouncement {
    pub message: String,
    pub link_text: Option<String>,
    pub link_url: Option<String>,
    pub priority: Option<String>,
    pub is_active: Option<bool>,
}

// Site Stats
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct SiteStats {
    pub id: i64,
    pub active_members: i64,
    pub events_held: i64,
    pub years_active: i64,
    pub industrial_partners: i64,
    pub constitution_pdf_url: Option<String>,
    pub office_location: Option<String>,
    pub contact_email: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSiteStats {
    pub active_members: Option<i64>,
    pub events_held: Option<i64>,
    pub years_active: Option<i64>,
    pub industrial_partners: Option<i64>,
    pub constitution_pdf_url: Option<String>,
    pub office_location: Option<String>,
    pub contact_email: Option<String>,
}

// Events
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub date: String,
    pub time_range: Option<String>,
    pub venue: Option<String>,
    pub category: String,
    pub short_description: Option<String>,
    pub full_summary: Option<String>,
    pub poster_url: Option<String>,
    pub registration_link: Option<String>,
    pub speaker_or_organizer: Option<String>,
    pub is_past: bool,
    pub is_featured: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEvent {
    pub title: String,
    pub slug: Option<String>,
    pub date: String,
    pub time_range: Option<String>,
    pub venue: Option<String>,
    pub category: Option<String>,
    pub short_description: Option<String>,
    pub full_summary: Option<String>,
    pub poster_url: Option<String>,
    pub registration_link: Option<String>,
    pub speaker_or_organizer: Option<String>,
    pub is_past: Option<bool>,
    pub is_featured: Option<bool>,
}

// Event Registrations
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct EventRegistration {
    pub id: i64,
    pub event_id: i64,
    pub student_name: String,
    pub roll_number: String,
    pub batch: String,
    pub department: String,
    pub email: String,
    pub phone: Option<String>,
    pub certificate_code: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEventRegistration {
    #[serde(default)]
    #[allow(dead_code)]
    pub event_id: Option<i64>,
    pub student_name: String,
    pub roll_number: String,
    pub batch: String,
    pub department: String,
    pub email: String,
    pub phone: Option<String>,
}

// Team Members
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct TeamMember {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub role_responsibility: Option<String>,
    pub photo_url: Option<String>,
    pub batch: Option<String>,
    pub tenure_year: String,
    pub is_current: bool,
    pub is_faculty_advisor: bool,
    pub bio: Option<String>,
    pub linkedin_url: Option<String>,
    pub email: Option<String>,
    pub display_order: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTeamMember {
    pub name: String,
    pub role: String,
    pub role_responsibility: Option<String>,
    pub photo_url: Option<String>,
    pub batch: Option<String>,
    pub tenure_year: Option<String>,
    pub is_current: Option<bool>,
    pub is_faculty_advisor: Option<bool>,
    pub bio: Option<String>,
    pub linkedin_url: Option<String>,
    pub email: Option<String>,
    pub display_order: Option<i64>,
}

// Gallery Images
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct GalleryImage {
    pub id: i64,
    pub image_url: String,
    pub caption: Option<String>,
    pub event_id: Option<i64>,
    pub event_name: Option<String>,
    pub year: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGalleryImage {
    pub image_url: String,
    pub caption: Option<String>,
    pub event_id: Option<i64>,
    pub event_name: Option<String>,
    pub year: Option<String>,
}

// Posts (Blog & Material of the Month)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub body_markdown: Option<String>,
    pub author_name: Option<String>,
    pub author_batch: Option<String>,
    pub cover_image_url: Option<String>,
    pub category: String,
    pub material_formula: Option<String>,
    pub crystal_structure: Option<String>,
    pub published_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePost {
    pub title: String,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub body_markdown: Option<String>,
    pub author_name: Option<String>,
    pub author_batch: Option<String>,
    pub cover_image_url: Option<String>,
    pub category: Option<String>,
    pub material_formula: Option<String>,
    pub crystal_structure: Option<String>,
}

// Resources
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Resource {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub category: String,
    pub software_tag: Option<String>,
    pub description: Option<String>,
    pub is_pinned: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateResource {
    pub title: String,
    pub url: String,
    pub category: Option<String>,
    pub software_tag: Option<String>,
    pub description: Option<String>,
    pub is_pinned: Option<bool>,
}

// Opportunities
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Opportunity {
    pub id: i64,
    pub title: String,
    pub organization: String,
    pub opportunity_type: String,
    pub location: Option<String>,
    pub deadline: Option<String>,
    pub apply_url: Option<String>,
    pub description: Option<String>,
    pub posted_by_alumnus: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOpportunity {
    pub title: String,
    pub organization: String,
    pub opportunity_type: Option<String>,
    pub location: Option<String>,
    pub deadline: Option<String>,
    pub apply_url: Option<String>,
    pub description: Option<String>,
    pub posted_by_alumnus: Option<String>,
}

// Alumni
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Alumnus {
    pub id: i64,
    pub name: String,
    pub batch: String,
    pub current_position: Option<String>,
    pub organization: Option<String>,
    pub location: Option<String>,
    pub photo_url: Option<String>,
    pub quote_testimonial: Option<String>,
    pub linkedin_url: Option<String>,
    pub research_domain: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAlumnus {
    pub name: String,
    pub batch: String,
    pub current_position: Option<String>,
    pub organization: Option<String>,
    pub location: Option<String>,
    pub photo_url: Option<String>,
    pub quote_testimonial: Option<String>,
    pub linkedin_url: Option<String>,
    pub research_domain: Option<String>,
}

// Projects
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Project {
    pub id: i64,
    pub title: String,
    pub student_names: String,
    pub batch: String,
    pub supervisor_name: Option<String>,
    pub project_type: String,
    pub abstract_summary: Option<String>,
    pub image_url: Option<String>,
    pub paper_or_repo_url: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProject {
    pub title: String,
    pub student_names: String,
    pub batch: String,
    pub supervisor_name: Option<String>,
    pub project_type: Option<String>,
    pub abstract_summary: Option<String>,
    pub image_url: Option<String>,
    pub paper_or_repo_url: Option<String>,
    pub tags: Option<String>,
}

// Sponsors
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Sponsor {
    pub id: i64,
    pub name: String,
    pub logo_url: Option<String>,
    pub website_url: Option<String>,
    pub tier: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSponsor {
    pub name: String,
    pub logo_url: Option<String>,
    pub website_url: Option<String>,
    pub tier: Option<String>,
}

// FAQs
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Faq {
    pub id: i64,
    pub question: String,
    pub answer: String,
    pub display_order: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFaq {
    pub question: String,
    pub answer: String,
    pub display_order: Option<i64>,
}

// Membership Applications
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct MembershipApplication {
    pub id: i64,
    pub full_name: String,
    pub roll_number: String,
    pub batch: String,
    pub department: String,
    pub email: String,
    pub phone: Option<String>,
    pub interests: Option<String>,
    pub why_join: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMembershipApplication {
    pub full_name: String,
    pub roll_number: String,
    pub batch: String,
    pub department: String,
    pub email: String,
    pub phone: Option<String>,
    pub interests: Option<String>,
    pub why_join: Option<String>,
}

// Contact Messages
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ContactMessage {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub subject: Option<String>,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateContactMessage {
    pub name: String,
    pub email: String,
    pub subject: Option<String>,
    pub message: String,
}

// Newsletter Subscribers
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct NewsletterSubscriber {
    pub id: i64,
    pub email: String,
    pub subscribed_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNewsletterSubscriber {
    pub email: String,
}

// Auth
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminUser {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: i64,
    pub email: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
}

/// Generic response for delete operations
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteResponse {
    pub message: String,
}

/// Certificate verification response
#[derive(Debug, Serialize, ToSchema)]
pub struct CertificateVerification {
    pub valid: bool,
    pub student_name: Option<String>,
    pub event_title: Option<String>,
    pub certificate_code: Option<String>,
}

// Site Sections (Editable Content Across Website)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct SiteSection {
    pub id: i64,
    pub section_key: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub content: String,
    pub badge: Option<String>,
    pub extra_json: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSiteSection {
    pub section_key: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub content: String,
    pub badge: Option<String>,
    pub extra_json: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSiteSection {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub content: Option<String>,
    pub badge: Option<String>,
    pub extra_json: Option<String>,
}
