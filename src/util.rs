//! Shared utility functions for PMS platform.

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
