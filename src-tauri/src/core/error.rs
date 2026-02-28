use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    AuthRequired,
    PermissionDenied,
    NotFound,
    ValidationError,
    RateLimited,
    UpstreamError,
    ExecutionError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    pub fingerprint: String,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        let message = message.into();
        let fingerprint = compute_fingerprint(code, &message);

        Self {
            code,
            message,
            retryable,
            fingerprint,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationError, message, false)
    }

    pub fn permission(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PermissionDenied, message, false)
    }

    pub fn execution(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ExecutionError, message, true)
    }
}

fn compute_fingerprint(code: ErrorCode, message: &str) -> String {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    message.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_stable_for_same_input() {
        let a = AppError::validation("bad input");
        let b = AppError::validation("bad input");
        assert_eq!(a.fingerprint, b.fingerprint);
    }

    #[test]
    fn fingerprint_changes_when_message_changes() {
        let a = AppError::validation("bad input");
        let b = AppError::validation("other bad input");
        assert_ne!(a.fingerprint, b.fingerprint);
    }

    #[test]
    fn display_includes_code_and_message() {
        let err = AppError::permission("no access");
        let text = err.to_string();
        assert!(text.contains("PermissionDenied"));
        assert!(text.contains("no access"));
    }
}
