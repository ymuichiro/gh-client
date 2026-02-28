use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseSummary {
    #[serde(rename = "tagName")]
    pub tag_name: String,
    pub name: Option<String>,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    #[serde(rename = "isPrerelease")]
    pub is_prerelease: bool,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "isLatest")]
    pub is_latest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCreated {
    pub tag_name: String,
    pub url: String,
}

pub fn parse_release_summaries(payload: &str) -> Result<Vec<ReleaseSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse release list payload: {}", err),
            false,
        )
    })
}

pub fn parse_release_created_output(
    tag_name: &str,
    payload: &str,
) -> Result<ReleaseCreated, AppError> {
    let url = payload.trim();
    if url.is_empty() {
        return Err(AppError::new(
            ErrorCode::UpstreamError,
            "release create output was empty",
            false,
        ));
    }

    Ok(ReleaseCreated {
        tag_name: tag_name.to_string(),
        url: url.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_list_payload() {
        let json = r#"[{"tagName":"v1.0.0","name":"v1","isDraft":false,"isPrerelease":false,"publishedAt":"2026-01-01T00:00:00Z","createdAt":"2026-01-01T00:00:00Z","isLatest":true}]"#;
        let list = parse_release_summaries(json).expect("payload should parse");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].tag_name, "v1.0.0");
    }

    #[test]
    fn release_list_parse_fails_for_invalid_shape() {
        let err = parse_release_summaries("[{\"tagName\":123}]")
            .expect_err("invalid payload should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }

    #[test]
    fn parses_release_create_output() {
        let created = parse_release_created_output(
            "v1.0.0",
            "https://github.com/octocat/hello/releases/tag/v1.0.0\n",
        )
        .expect("output should parse");
        assert_eq!(created.tag_name, "v1.0.0");
        assert!(created.url.contains("releases/tag/v1.0.0"));
    }
}
