use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollaboratorPermissions {
    pub admin: Option<bool>,
    pub push: Option<bool>,
    pub pull: Option<bool>,
    pub maintain: Option<bool>,
    pub triage: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Collaborator {
    pub login: String,
    pub permissions: Option<CollaboratorPermissions>,
}

pub fn parse_collaborators(payload: &str) -> Result<Vec<Collaborator>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse collaborators payload: {}", err),
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_collaborators_payload() {
        let json = r#"[{"login":"octocat","permissions":{"admin":true,"push":true,"pull":true}}]"#;
        let items = parse_collaborators(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].login, "octocat");
    }

    #[test]
    fn collaborators_parse_fails_for_invalid_shape() {
        let err = parse_collaborators("[{\"login\":1}]").expect_err("invalid payload should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }
}
