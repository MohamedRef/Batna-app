//! Reads that failed (engineering audit F10). A read error is never shown as
//! "no data": the last good copy stays in use, Today says what couldn't be
//! read, and Retry reads it again.

use std::sync::Mutex;

static READ_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// Reports a read that failed, to the log and on Today.
pub fn read_failed(what: &str, e: &dyn std::fmt::Display) {
    crate::diag::report!("reading {what}: {e}");
    *READ_ERROR.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(format!("{what}: {e}"));
}

/// Clears `what`'s earlier failure once it reads again.
pub fn read_recovered(what: &str) {
    let mut error = READ_ERROR.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if error.as_deref().is_some_and(|m| m.starts_with(&format!("{what}: "))) {
        *error = None;
    }
}

/// The last read that failed and hasn't recovered, if any.
pub fn read_error() -> Option<String> {
    READ_ERROR.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
}
