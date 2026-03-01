use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RulesetSummary {
    pub id: u64,
    pub name: String,
    pub target: Option<String>,
    pub enforcement: Option<String>,
}

pub fn parse_ruleset_summaries(payload: &str) -> Result<Vec<RulesetSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse rulesets payload: {}", err),
            false,
        )
    })
}

pub fn parse_ruleset(payload: &str) -> Result<RulesetSummary, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse ruleset payload: {}", err),
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rulesets_payload() {
        let json = r#"[{"id":1,"name":"Protect main","target":"branch","enforcement":"active"}]"#;
        let items = parse_ruleset_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Protect main");
    }

    #[test]
    fn parses_ruleset_payload() {
        let json = r#"{"id":1,"name":"Protect main","target":"branch","enforcement":"active"}"#;
        let item = parse_ruleset(json).expect("payload should parse");
        assert_eq!(item.id, 1);
    }
}
