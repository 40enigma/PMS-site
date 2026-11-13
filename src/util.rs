//! Shared utility functions for PMS platform.

use crate::error::AppError;
use crate::models::CreateMembershipApplication;

/// Generate a URL-friendly slug from a title string.
pub fn slugify(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let collapsed = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if collapsed.is_empty() {
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    } else {
        collapsed
    }
}

/// Check if a database error is a SQLite UNIQUE constraint violation.
pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.is_unique_violation() {
            return true;
        }
    }
    err.to_string().contains("UNIQUE constraint failed")
}

/// Today's date in the society's local timezone (PKT).
///
/// Events are calendrical dates, so comparing them against a UTC "today" closes
/// registration a day early for anything happening before 05:00 local time.
pub fn today_local() -> String {
    let offset = chrono::FixedOffset::east_opt(5 * 3600).expect("valid PKT offset");
    chrono::Utc::now()
        .with_timezone(&offset)
        .format("%Y-%m-%d")
        .to_string()
}

/// Validate a `YYYY-MM-DD` calendrical date before it is stored.
///
/// Event dates are compared as strings and rendered into the sitemap, so a
/// free-text value silently breaks ordering, "past" detection and lastmod tags.
pub fn validate_iso_date(field: &str, value: &str) -> Result<(), AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{field} is required")));
    }
    chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| AppError::BadRequest(format!("{field} must be a YYYY-MM-DD date")))
}

/// Field limits for a membership application, shared by the public submit
/// endpoint and admin-created records so both enforce the same rules.
pub fn validate_membership(payload: &CreateMembershipApplication) -> Result<(), AppError> {
    if payload.full_name.trim().is_empty() {
        return Err(AppError::BadRequest("full_name is required".into()));
    }
    if payload.full_name.len() > 200 {
        return Err(AppError::BadRequest(
            "full_name is too long (max 200 chars)".into(),
        ));
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
    if payload.roll_number.len() > 50 {
        return Err(AppError::BadRequest(
            "roll_number is too long (max 50 chars)".into(),
        ));
    }
    if payload.batch.trim().is_empty() {
        return Err(AppError::BadRequest("batch is required".into()));
    }
    if payload.batch.len() > 50 {
        return Err(AppError::BadRequest(
            "batch is too long (max 50 chars)".into(),
        ));
    }
    if payload.department.trim().is_empty() {
        return Err(AppError::BadRequest("department is required".into()));
    }
    if payload.department.len() > 200 {
        return Err(AppError::BadRequest(
            "department is too long (max 200 chars)".into(),
        ));
    }
    if let Some(phone) = payload.phone.as_ref() {
        if phone.len() > 50 {
            return Err(AppError::BadRequest(
                "phone is too long (max 50 chars)".into(),
            ));
        }
    }
    if let Some(interests) = payload.interests.as_ref() {
        if interests.len() > 2000 {
            return Err(AppError::BadRequest(
                "interests is too long (max 2000 chars)".into(),
            ));
        }
    }
    if let Some(why_join) = payload.why_join.as_ref() {
        if why_join.len() > 2000 {
            return Err(AppError::BadRequest(
                "why_join is too long (max 2000 chars)".into(),
            ));
        }
    }
    Ok(())
}
