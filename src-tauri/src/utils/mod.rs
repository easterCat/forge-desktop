//! Cross-cutting utilities used by multiple command modules.

/// Returns the current UTC timestamp as an RFC3339 string.
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Returns a future UTC timestamp `offset_secs` seconds from now as RFC3339.
pub fn future_rfc3339(offset_secs: i64) -> String {
    chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(offset_secs))
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339()
}

pub mod path_guard;

pub use path_guard::{PathGuard, PathGuardError};