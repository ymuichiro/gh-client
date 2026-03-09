use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueAuthor {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueLabel {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueAssignee {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueSummary {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub url: String,
    pub author: Option<IssueAuthor>,
    pub labels: Option<Vec<IssueLabel>>,
    pub assignees: Option<Vec<IssueAssignee>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueCreated {
    pub number: u64,
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueCommentDetail {
    pub id: Option<u64>,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    pub author: Option<IssueAuthor>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueDetail {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub url: String,
    pub body: String,
    pub author: Option<IssueAuthor>,
    pub labels: Option<Vec<IssueLabel>>,
    pub assignees: Option<Vec<IssueAssignee>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub comments: Vec<IssueCommentDetail>,
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

pub fn parse_issue_detail(payload: &str) -> Result<IssueDetail, AppError> {
    let value: Value = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse issue detail payload: {}", err),
            false,
        )
    })?;

    let object = value.as_object().ok_or_else(|| {
        AppError::new(
            ErrorCode::UpstreamError,
            "issue detail payload must be a JSON object",
            false,
        )
    })?;

    let number = object
        .get("number")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UpstreamError,
                "issue detail payload missing number",
                false,
            )
        })?;
    let title = required_string(object.get("title"), "title")?;
    let state = required_string(object.get("state"), "state")?;
    let url = required_string(object.get("url"), "url")?;
    let body = optional_string(object.get("body")).unwrap_or_default();
    let author = parse_author(object.get("author"));
    let labels = parse_labels(object.get("labels"));
    let assignees = parse_assignees(object.get("assignees"));
    let updated_at = optional_string(object.get("updatedAt"));
    let comments = parse_comments(object.get("comments"));

    Ok(IssueDetail {
        number,
        title,
        state,
        url,
        body,
        author,
        labels,
        assignees,
        updated_at,
        comments,
    })
}

fn required_string(value: Option<&Value>, field: &str) -> Result<String, AppError> {
    optional_string(value).ok_or_else(|| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("issue detail payload missing {}", field),
            false,
        )
    })
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

fn parse_author(value: Option<&Value>) -> Option<IssueAuthor> {
    let Some(object) = value.and_then(Value::as_object) else {
        return None;
    };

    let login = object
        .get("login")
        .and_then(Value::as_str)
        .or_else(|| object.get("name").and_then(Value::as_str))
        .map(str::trim)
        .filter(|v| !v.is_empty())?;

    Some(IssueAuthor {
        login: login.to_string(),
    })
}

fn parse_labels(value: Option<&Value>) -> Option<Vec<IssueLabel>> {
    let values = value.and_then(Value::as_array)?;

    let mut labels = Vec::new();
    for entry in values {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let Some(name) = object
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };

        labels.push(IssueLabel {
            name: name.to_string(),
        });
    }

    if labels.is_empty() {
        None
    } else {
        Some(labels)
    }
}

fn parse_assignees(value: Option<&Value>) -> Option<Vec<IssueAssignee>> {
    let values = value.and_then(Value::as_array)?;

    let mut assignees = Vec::new();
    for entry in values {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let Some(login) = object
            .get("login")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };

        assignees.push(IssueAssignee {
            login: login.to_string(),
        });
    }

    if assignees.is_empty() {
        None
    } else {
        Some(assignees)
    }
}

fn parse_comments(value: Option<&Value>) -> Vec<IssueCommentDetail> {
    let Some(raw) = value else {
        return Vec::new();
    };

    let entries = if let Some(values) = raw.as_array() {
        values.clone()
    } else if let Some(nodes) = raw
        .as_object()
        .and_then(|object| object.get("nodes"))
        .and_then(Value::as_array)
    {
        nodes.clone()
    } else {
        return Vec::new();
    };

    let mut comments = Vec::new();
    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };

        let body = object
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if body.is_empty() {
            continue;
        }

        let id = object
            .get("id")
            .and_then(Value::as_u64)
            .or_else(|| object.get("databaseId").and_then(Value::as_u64));
        let created_at = optional_string(object.get("createdAt"))
            .or_else(|| optional_string(object.get("created_at")));
        let author =
            parse_author(object.get("author")).or_else(|| parse_author(object.get("user")));
        let url =
            optional_string(object.get("url")).or_else(|| optional_string(object.get("html_url")));

        comments.push(IssueCommentDetail {
            id,
            body,
            created_at,
            author,
            url,
        });
    }

    comments
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

    #[test]
    fn parses_issue_detail_payload_with_comments() {
        let json = r#"
        {
          "number": 26,
          "title": "Issue title",
          "state": "OPEN",
          "url": "https://github.com/octocat/hello/issues/26",
          "body": "Issue body",
          "author": { "login": "octocat" },
          "labels": [{ "name": "bug" }],
          "assignees": [{ "login": "hubot" }],
          "updatedAt": "2026-03-09T00:00:00Z",
          "comments": [
            {
              "id": 101,
              "body": "First comment",
              "createdAt": "2026-03-09T01:00:00Z",
              "author": { "login": "maintainer" },
              "url": "https://github.com/octocat/hello/issues/26#issuecomment-101"
            }
          ]
        }
        "#;

        let detail = parse_issue_detail(json).expect("detail payload should parse");
        assert_eq!(detail.number, 26);
        assert_eq!(detail.comments.len(), 1);
        assert_eq!(detail.comments[0].id, Some(101));
        assert_eq!(
            detail.comments[0].author.as_ref().map(|a| a.login.as_str()),
            Some("maintainer")
        );
    }

    #[test]
    fn parses_issue_detail_comment_nodes_shape() {
        let json = r#"
        {
          "number": 1,
          "title": "Issue title",
          "state": "OPEN",
          "url": "https://example.test/issues/1",
          "body": "Body",
          "comments": {
            "nodes": [
              {
                "databaseId": 10,
                "body": "Node comment",
                "createdAt": "2026-03-09T01:00:00Z",
                "author": { "login": "reviewer-a" }
              }
            ]
          }
        }
        "#;

        let detail = parse_issue_detail(json).expect("detail payload should parse");
        assert_eq!(detail.comments.len(), 1);
        assert_eq!(detail.comments[0].id, Some(10));
    }
}
