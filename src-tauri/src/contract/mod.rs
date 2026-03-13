use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;

pub const PAYLOAD_CONTRACT_VERSION: &str = "2026-03-13.v5";

pub const STABLE_COMMAND_IDS: &[&str] = &[
    "auth.organizations.list",
    "auth.status",
    "issue.close",
    "issue.comment",
    "issue.edit",
    "issue.list",
    "issue.reopen",
    "issue.view",
    "pr.close",
    "pr.comments.list",
    "pr.diff.files.list",
    "pr.diff.raw.get",
    "pr.list",
    "pr.merge",
    "pr.reopen",
    "pr.review",
    "pr.review_threads.list",
    "pr.view",
    "repo.list",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContractRepoPermission {
    Viewer,
    Write,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrontendCommandEnvelope {
    pub contract_version: String,
    pub request_id: String,
    pub command_id: String,
    pub permission: Option<ContractRepoPermission>,
    pub payload: Value,
}

impl FrontendCommandEnvelope {
    pub fn new(
        request_id: impl Into<String>,
        command_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            contract_version: PAYLOAD_CONTRACT_VERSION.to_string(),
            request_id: request_id.into(),
            command_id: command_id.into(),
            permission: None,
            payload,
        }
    }

    pub fn validate(&self, registry: &CommandRegistry) -> Result<(), AppError> {
        if self.contract_version != PAYLOAD_CONTRACT_VERSION {
            return Err(AppError::validation(format!(
                "unsupported contract version: {}",
                self.contract_version
            )));
        }

        if self.request_id.trim().is_empty() {
            return Err(AppError::validation("request_id is required"));
        }

        if self.command_id.trim().is_empty() {
            return Err(AppError::validation("command_id is required"));
        }

        if !STABLE_COMMAND_IDS.contains(&self.command_id.as_str()) {
            return Err(AppError::validation(format!(
                "command_id `{}` is not part of the stable frontend contract",
                self.command_id
            )));
        }

        let registered = registry.command_ids();
        if !registered.contains(&self.command_id.as_str()) {
            return Err(AppError::validation(format!(
                "command_id `{}` is not registered in backend",
                self.command_id
            )));
        }

        Ok(())
    }
}

pub fn validate_registry_contract(registry: &CommandRegistry) -> Result<(), AppError> {
    let actual = registry.command_ids();

    for expected in STABLE_COMMAND_IDS {
        if !actual.contains(expected) {
            return Err(AppError::validation(format!(
                "stable contract mismatch: missing command `{}` in registry",
                expected
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command_registry::CommandRegistry;

    #[test]
    fn stable_contract_matches_default_registry() {
        let registry = CommandRegistry::with_defaults();
        validate_registry_contract(&registry).expect("stable contract should match registry");
    }

    #[test]
    fn envelope_validation_rejects_unknown_command() {
        let registry = CommandRegistry::with_defaults();
        let envelope = FrontendCommandEnvelope {
            contract_version: PAYLOAD_CONTRACT_VERSION.to_string(),
            request_id: "req-1".into(),
            command_id: "unknown.command".into(),
            permission: None,
            payload: serde_json::json!({"owner":"octocat"}),
        };

        let err = envelope
            .validate(&registry)
            .expect_err("unknown command must fail");
        assert!(err.message.contains("stable frontend contract"));
    }

    #[test]
    fn envelope_json_roundtrip_is_stable() {
        let registry = CommandRegistry::with_defaults();
        let envelope = FrontendCommandEnvelope {
            contract_version: PAYLOAD_CONTRACT_VERSION.to_string(),
            request_id: "req-2".into(),
            command_id: "repo.list".into(),
            permission: Some(ContractRepoPermission::Viewer),
            payload: serde_json::json!({"owner":"octocat","limit":20}),
        };

        envelope
            .validate(&registry)
            .expect("envelope should be valid");

        let json = serde_json::to_string(&envelope).expect("serialization should succeed");
        let parsed: FrontendCommandEnvelope =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(parsed.contract_version, PAYLOAD_CONTRACT_VERSION);
        assert_eq!(parsed.command_id, "repo.list");
    }
}
