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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchCommit {
    pub sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchSummary {
    pub name: String,
    pub protected: bool,
    pub commit: BranchCommit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitDetails {
    pub message: String,
    pub author: Option<CommitAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitSummary {
    pub sha: String,
    pub commit: CommitDetails,
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

pub fn parse_branch_summaries(payload: &str) -> Result<Vec<BranchSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse branch payload: {}", err),
            false,
        )
    })
}

pub fn parse_commit_summaries(payload: &str) -> Result<Vec<CommitSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse commit payload: {}", err),
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

    #[test]
    fn parses_valid_branch_payload() {
        let json = r#"
        [
          {
            "name": "main",
            "protected": true,
            "commit": {"sha": "abc123"}
          }
        ]
        "#;

        let branches = parse_branch_summaries(json).expect("branch payload should parse");
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "main");
        assert_eq!(branches[0].commit.sha, "abc123");
    }

    #[test]
    fn parses_valid_commit_payload() {
        let json = r#"
        [
          {
            "sha": "abc123",
            "commit": {
              "message": "initial commit",
              "author": {
                "name": "octocat",
                "email": "octocat@example.com",
                "date": "2026-01-01T00:00:00Z"
              }
            }
          }
        ]
        "#;

        let commits = parse_commit_summaries(json).expect("commit payload should parse");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].sha, "abc123");
        assert_eq!(commits[0].commit.message, "initial commit");
    }
}
