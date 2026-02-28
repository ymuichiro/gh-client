use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowSummary {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSummary {
    #[serde(rename = "databaseId")]
    pub database_id: u64,
    #[serde(rename = "workflowName")]
    pub workflow_name: Option<String>,
    #[serde(rename = "headBranch")]
    pub head_branch: Option<String>,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: String,
    #[serde(rename = "displayTitle")]
    pub display_title: Option<String>,
}

pub fn parse_workflow_summaries(payload: &str) -> Result<Vec<WorkflowSummary>, AppError> {
    if payload.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse workflow list payload: {}", err),
            false,
        )
    })
}

pub fn parse_run_summaries(payload: &str) -> Result<Vec<RunSummary>, AppError> {
    if payload.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse run list payload: {}", err),
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_workflow_list_payload() {
        let json = r#"[{"id":1,"name":"CI","path":".github/workflows/ci.yml","state":"active"}]"#;
        let items = parse_workflow_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "CI");
    }

    #[test]
    fn workflow_list_parse_fails_for_invalid_shape() {
        let err =
            parse_workflow_summaries("[{\"id\":\"x\"}]").expect_err("invalid payload should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }

    #[test]
    fn parses_run_list_payload() {
        let json = r#"[{"databaseId":10,"workflowName":"CI","headBranch":"main","status":"completed","conclusion":"success","url":"https://example/run/10","displayTitle":"build"}]"#;
        let items = parse_run_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].database_id, 10);
    }

    #[test]
    fn run_list_parse_fails_for_invalid_shape() {
        let err = parse_run_summaries("[{\"databaseId\":\"x\"}]")
            .expect_err("invalid payload should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }

    #[test]
    fn empty_payload_is_treated_as_empty_list() {
        let items = parse_workflow_summaries("").expect("empty payload should be allowed");
        assert!(items.is_empty());

        let runs = parse_run_summaries("").expect("empty payload should be allowed");
        assert!(runs.is_empty());
    }
}
