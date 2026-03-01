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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretSummary {
    pub name: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VariableSummary {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookConfig {
    pub url: Option<String>,
    #[serde(rename = "content_type")]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookSummary {
    pub id: u64,
    pub name: Option<String>,
    pub active: Option<bool>,
    pub events: Option<Vec<String>>,
    pub config: Option<WebhookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchProtection {
    pub url: Option<String>,
    #[serde(rename = "required_pull_request_reviews")]
    pub required_pull_request_reviews: Option<RequiredPullRequestReviews>,
    #[serde(rename = "enforce_admins")]
    pub enforce_admins: Option<EnforceAdmins>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredPullRequestReviews {
    #[serde(rename = "required_approving_review_count")]
    pub required_approving_review_count: Option<u8>,
    #[serde(rename = "dismiss_stale_reviews")]
    pub dismiss_stale_reviews: Option<bool>,
    #[serde(rename = "require_code_owner_reviews")]
    pub require_code_owner_reviews: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnforceAdmins {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeployKeySummary {
    pub id: u64,
    pub title: String,
    pub key: Option<String>,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependabotAlertDependency {
    pub package: Option<DependabotPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependabotPackage {
    pub name: Option<String>,
    pub ecosystem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependabotAlert {
    pub number: Option<u64>,
    pub state: Option<String>,
    pub dependency: Option<DependabotAlertDependency>,
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

pub fn parse_secret_summaries(payload: &str) -> Result<Vec<SecretSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse secret payload: {}", err),
            false,
        )
    })
}

pub fn parse_variable_summaries(payload: &str) -> Result<Vec<VariableSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse variable payload: {}", err),
            false,
        )
    })
}

pub fn parse_webhook_summaries(payload: &str) -> Result<Vec<WebhookSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse webhook payload: {}", err),
            false,
        )
    })
}

pub fn parse_branch_protection(payload: &str) -> Result<BranchProtection, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse branch protection payload: {}", err),
            false,
        )
    })
}

pub fn parse_deploy_keys(payload: &str) -> Result<Vec<DeployKeySummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse deploy keys payload: {}", err),
            false,
        )
    })
}

pub fn parse_dependabot_alerts(payload: &str) -> Result<Vec<DependabotAlert>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse dependabot alert payload: {}", err),
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

    #[test]
    fn parses_secret_payload() {
        let json =
            r#"[{"name":"TOKEN","updatedAt":"2026-01-01T00:00:00Z","visibility":"private"}]"#;
        let items = parse_secret_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "TOKEN");
    }

    #[test]
    fn parses_variable_payload() {
        let json = r#"[{"name":"ENV","value":"prod","updatedAt":"2026-01-01T00:00:00Z","createdAt":"2026-01-01T00:00:00Z","visibility":"private"}]"#;
        let items = parse_variable_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "ENV");
    }

    #[test]
    fn parses_webhook_payload() {
        let json = r#"[{"id":1,"name":"web","active":true,"events":["push"],"config":{"url":"https://example.com","content_type":"json"}}]"#;
        let items = parse_webhook_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 1);
    }

    #[test]
    fn parses_branch_protection_payload() {
        let json = r#"{"url":"https://example.com","required_pull_request_reviews":{"required_approving_review_count":1,"dismiss_stale_reviews":true,"require_code_owner_reviews":false},"enforce_admins":{"enabled":true}}"#;
        let item = parse_branch_protection(json).expect("payload should parse");
        assert_eq!(item.enforce_admins.and_then(|v| v.enabled), Some(true));
    }

    #[test]
    fn parses_deploy_keys_payload() {
        let json = r#"[{"id":1,"title":"ci","key":"ssh-rsa AAA","read_only":true}]"#;
        let items = parse_deploy_keys(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 1);
    }

    #[test]
    fn parses_dependabot_alert_payload() {
        let json = r#"[{"number":1,"state":"open","dependency":{"package":{"name":"openssl","ecosystem":"npm"}}}]"#;
        let items = parse_dependabot_alerts(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].number, Some(1));
    }
}
