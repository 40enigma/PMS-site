//! Shared utility and validation functions for PMS platform.

use crate::error::AppError;

/// Generate a URL-friendly slug from a title string.
#[must_use]
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
#[must_use]
pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.is_unique_violation() {
            return true;
        }
    }
    err.to_string().contains("UNIQUE constraint failed")
}

/// Check if a database error is a SQLite FOREIGN KEY constraint violation.
#[must_use]
pub fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.is_foreign_key_violation() {
            return true;
        }
    }
    err.to_string().contains("FOREIGN KEY constraint failed")
}

/// Normalize optional query filter strings by trimming whitespace and turning empty strings into `None`.
#[must_use]
pub fn normalize_filter(raw: Option<String>) -> Option<String> {
    raw.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

/// Returns `true` when `RUST_ENV` is set to `production` (case-insensitive).
#[must_use]
pub fn is_production() -> bool {
    std::env::var("RUST_ENV")
        .unwrap_or_default()
        .eq_ignore_ascii_case("production")
}

/// Escape XML special characters (`&`, `<`, `>`, `"`, `'`).
#[must_use]
pub fn xml_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

fn configured_now() -> chrono::DateTime<chrono::FixedOffset> {
    let offset_minutes = std::env::var("TIMEZONE_OFFSET_MINUTES")
        .ok()
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(300)
        .clamp(-720, 840);
    let now_utc = chrono::Utc::now();
    let offset = chrono::FixedOffset::east_opt(offset_minutes * 60)
        .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("UTC offset"));
    now_utc.with_timezone(&offset)
}

/// Return today's date (`YYYY-MM-DD`) in the configured local timezone
/// (`TIMEZONE_OFFSET_MINUTES`, defaulting to `+300` for Pakistan Standard Time UTC+5).
#[must_use]
pub fn today_date_string() -> String {
    configured_now().format("%Y-%m-%d").to_string()
}

/// Compute the current academic tenure label (e.g. `"2026-2027"` from August onward)
/// in the configured local timezone (`TIMEZONE_OFFSET_MINUTES`, default `+300`).
#[must_use]
pub fn current_academic_year() -> String {
    use chrono::Datelike;
    let now = configured_now();
    let year = now.year();
    if now.month() >= 8 {
        format!("{year}-{}", year + 1)
    } else {
        format!("{}-{year}", year - 1)
    }
}

/// Validate and trim a required string field with a maximum character length.
///
/// # Errors
/// Returns `AppError::BadRequest` if the trimmed string is empty or exceeds `max_chars`.
pub fn validate_required_str(raw: &str, field: &str, max_chars: usize) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{field} is required")));
    }
    if trimmed.chars().count() > max_chars {
        return Err(AppError::BadRequest(format!(
            "{field} is too long (max {max_chars} chars)"
        )));
    }
    Ok(trimmed.to_string())
}

/// Validate and trim an optional string field (`None` or empty becomes `Ok(None)`).
///
/// # Errors
/// Returns `AppError::BadRequest` if the trimmed string exceeds `max_chars`.
pub fn validate_optional_str(
    raw: Option<&str>,
    field: &str,
    max_chars: usize,
) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.chars().count() > max_chars {
                return Err(AppError::BadRequest(format!(
                    "{field} is too long (max {max_chars} chars)"
                )));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

/// Validate a clearable optional string for `COALESCE`/`CASE WHEN '' THEN NULL` updates.
/// `None` preserves the existing value; `Some(whitespace)` becomes `Some("")` to clear to `NULL`.
///
/// # Errors
/// Returns `AppError::BadRequest` if non-empty trimmed value exceeds `max_chars`.
pub fn validate_clearable_str(
    raw: Option<&str>,
    field: &str,
    max_chars: usize,
) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Ok(Some(String::new()));
            }
            if trimmed.chars().count() > max_chars {
                return Err(AppError::BadRequest(format!(
                    "{field} is too long (max {max_chars} chars)"
                )));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

/// Validate and normalize (trim + lowercase) an email address.
///
/// # Errors
/// Returns `AppError::BadRequest` if the email is empty, too long, or syntactically invalid.
pub fn validate_email(raw: &str) -> Result<String, AppError> {
    let email = raw.trim().to_lowercase();
    if email.is_empty() {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    if email.len() > 254 {
        return Err(AppError::BadRequest("email is too long".into()));
    }
    if email.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    let Some((local, domain)) = email.split_once('@') else {
        return Err(AppError::BadRequest("A valid email is required".into()));
    };
    if local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
    {
        return Err(AppError::BadRequest("A valid email is required".into()));
    }
    Ok(email)
}

/// Validate a URL against safe protocols (`https://`, `http://`, single-slash `/...`, or `#...`).
///
/// # Errors
/// Returns `AppError::BadRequest` if the URL is empty, too long, contains control/whitespace characters,
/// or uses an unsafe scheme (such as `javascript:` or `data:`).
pub fn validate_safe_url(raw: &str, field: &str, max_chars: usize) -> Result<String, AppError> {
    let trimmed = validate_required_str(raw, field, max_chars)?;
    if trimmed.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(AppError::BadRequest(format!(
            "{field} must not contain whitespace or control characters"
        )));
    }
    let lower = trimmed.to_ascii_lowercase();
    let is_safe = lower.starts_with("https://")
        || lower.starts_with("http://")
        || (trimmed.starts_with('/') && !trimmed.starts_with("//"))
        || trimmed.starts_with('#');
    if !is_safe {
        return Err(AppError::BadRequest(format!(
            "{field} must be an http(s) URL or a site-relative path"
        )));
    }
    Ok(trimmed)
}

/// Validate an optional URL (`None` or empty becomes `Ok(None)`).
///
/// # Errors
/// Returns `AppError::BadRequest` if non-empty and invalid.
pub fn validate_optional_safe_url(
    raw: Option<&str>,
    field: &str,
    max_chars: usize,
) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => validate_safe_url(s, field, max_chars).map(Some),
    }
}

/// Validate a clearable URL for partial `COALESCE` updates.
///
/// # Errors
/// Returns `AppError::BadRequest` if non-empty and invalid.
pub fn validate_clearable_safe_url(
    raw: Option<&str>,
    field: &str,
    max_chars: usize,
) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(Some(String::new())),
        Some(s) => validate_safe_url(s, field, max_chars).map(Some),
    }
}

/// Validate an ISO calendar date string (`YYYY-MM-DD`).
///
/// # Errors
/// Returns `AppError::BadRequest` if empty or not a valid `YYYY-MM-DD` date.
pub fn validate_iso_date(raw: &str, field: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{field} is required")));
    }
    let parsed = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest(format!("{field} must be a valid YYYY-MM-DD date")))?;
    Ok(parsed.format("%Y-%m-%d").to_string())
}

/// Validate an optional ISO calendar date (`YYYY-MM-DD`).
///
/// # Errors
/// Returns `AppError::BadRequest` if non-empty and not a valid `YYYY-MM-DD` date.
pub fn validate_optional_iso_date(
    raw: Option<&str>,
    field: &str,
) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => validate_iso_date(s, field).map(Some),
    }
}

/// Validate a value against an allowlist, returning `default` if `None` or empty.
///
/// # Errors
/// Returns `AppError::BadRequest` if the provided value is not in `allowed`.
pub fn validate_enum(
    raw: Option<&str>,
    default: &str,
    allowed: &[&str],
    field: &str,
) -> Result<String, AppError> {
    let val = raw
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default);
    if let Some(matched) = allowed.iter().find(|&&opt| opt.eq_ignore_ascii_case(val)) {
        return Ok((*matched).to_string());
    }
    Err(AppError::BadRequest(format!(
        "{field} must be one of: {}",
        allowed.join(", ")
    )))
}
