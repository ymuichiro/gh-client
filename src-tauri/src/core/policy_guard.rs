use crate::core::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RepoPermission {
    Viewer,
    Write,
    Admin,
}

impl RepoPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Write => "write",
            Self::Admin => "admin",
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PolicyGuard;

impl PolicyGuard {
    pub fn require(
        &self,
        required: RepoPermission,
        actual: RepoPermission,
        action: &str,
    ) -> Result<(), AppError> {
        if actual >= required {
            return Ok(());
        }

        Err(AppError::permission(format!(
            "action `{}` requires {} permission (actual: {})",
            action,
            required.as_str(),
            actual.as_str()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_when_permission_is_sufficient() {
        let guard = PolicyGuard;
        let result = guard.require(RepoPermission::Write, RepoPermission::Admin, "repo.create");
        assert!(result.is_ok());
    }

    #[test]
    fn denies_when_permission_is_insufficient() {
        let guard = PolicyGuard;
        let result = guard.require(RepoPermission::Admin, RepoPermission::Write, "repo.delete");
        let err = result.expect_err("expected permission error");
        assert!(err.message.contains("repo.delete"));
        assert!(err.message.contains("admin"));
    }
}
