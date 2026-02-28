use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestAuthor {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestSummary {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub url: String,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    pub author: Option<PullRequestAuthor>,
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
    #[serde(rename = "baseRefName")]
    pub base_ref_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestCreated {
    pub number: u64,
    #[serde(rename = "html_url")]
    pub url: String,
    pub state: String,
}

pub fn parse_pull_request_summaries(payload: &str) -> Result<Vec<PullRequestSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse pull request list payload: {}", err),
            false,
        )
    })
}

pub fn parse_pull_request_created(payload: &str) -> Result<PullRequestCreated, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse pull request create payload: {}", err),
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pull_request_list_payload() {
        let json = r#"
        [
          {
            "number": 10,
            "title": "Add feature",
            "state": "OPEN",
            "url": "https://github.com/octocat/hello/pull/10",
            "isDraft": false,
            "author": {"login": "octocat"},
            "headRefName": "feature-1",
            "baseRefName": "main"
          }
        ]
        "#;

        let prs = parse_pull_request_summaries(json).expect("list payload should parse");
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 10);
        assert_eq!(prs[0].head_ref_name, "feature-1");
    }

    #[test]
    fn list_parse_returns_error_for_invalid_shape() {
        let json = r#"[{"number":"bad"}]"#;
        let err = parse_pull_request_summaries(json).expect_err("invalid list shape should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }

    #[test]
    fn parses_pull_request_create_payload() {
        let json = r#"
        {
          "number": 12,
          "html_url": "https://github.com/octocat/hello/pull/12",
          "state": "OPEN"
        }
        "#;

        let created = parse_pull_request_created(json).expect("create payload should parse");
        assert_eq!(created.number, 12);
        assert_eq!(created.state, "OPEN");
    }

    #[test]
    fn create_parse_returns_error_for_invalid_shape() {
        let json = r#"{"html_url":true}"#;
        let err = parse_pull_request_created(json).expect_err("invalid create shape should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }
}
