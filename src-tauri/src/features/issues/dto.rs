use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueAuthor {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueSummary {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub url: String,
    pub author: Option<IssueAuthor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueCreated {
    pub number: u64,
    pub url: String,
    pub state: String,
}

pub fn parse_issue_summaries(payload: &str) -> Result<Vec<IssueSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse issue list payload: {}", err),
            false,
        )
    })
}

pub fn parse_issue_created_output(payload: &str) -> Result<IssueCreated, AppError> {
    let url = payload.trim();
    if url.is_empty() {
        return Err(AppError::new(
            ErrorCode::UpstreamError,
            "issue create output was empty",
            false,
        ));
    }

    let number = url
        .rsplit('/')
        .next()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UpstreamError,
                format!("failed to parse issue number from URL: {}", url),
                false,
            )
        })?
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UpstreamError,
                format!("failed to parse issue number from URL: {}", url),
                false,
            )
        })?;

    Ok(IssueCreated {
        number,
        url: url.to_string(),
        state: "OPEN".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_list_payload() {
        let json = r#"
        [
          {
            "number": 10,
            "title": "Bug",
            "state": "OPEN",
            "url": "https://github.com/octocat/hello/issues/10",
            "author": {"login": "octocat"}
          }
        ]
        "#;

        let items = parse_issue_summaries(json).expect("list payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].number, 10);
    }

    #[test]
    fn issue_list_parse_returns_error_for_invalid_shape() {
        let err = parse_issue_summaries("[{\"number\":\"bad\"}]")
            .expect_err("invalid payload should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }

    #[test]
    fn parses_issue_created_output_url() {
        let created = parse_issue_created_output("https://github.com/octocat/hello/issues/42\n")
            .expect("url output should parse");
        assert_eq!(created.number, 42);
        assert_eq!(created.state, "OPEN");
    }

    #[test]
    fn issue_created_output_fails_for_invalid_url() {
        let err = parse_issue_created_output("not-a-url").expect_err("invalid output should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }
}
