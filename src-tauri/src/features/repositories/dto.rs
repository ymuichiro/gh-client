use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoSummary {
    pub name: String,
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
    pub description: Option<String>,
    pub url: String,
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
    #[serde(rename = "viewerPermission")]
    pub viewer_permission: String,
}

pub fn parse_repo_summaries(payload: &str) -> Result<Vec<RepoSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse repository payload: {}", err),
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_json_payload() {
        let json = r#"
        [
          {
            "name": "repo-a",
            "nameWithOwner": "octocat/repo-a",
            "description": "hello",
            "url": "https://github.com/octocat/repo-a",
            "isPrivate": false,
            "viewerPermission": "ADMIN"
          }
        ]
        "#;

        let repos = parse_repo_summaries(json).expect("payload should parse");
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name_with_owner, "octocat/repo-a");
    }

    #[test]
    fn returns_error_when_json_shape_is_invalid() {
        let json = r#"[{"name":"repo-a"}]"#;
        let err = parse_repo_summaries(json).expect_err("invalid shape should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
        assert!(err.message.contains("parse"));
    }
}
