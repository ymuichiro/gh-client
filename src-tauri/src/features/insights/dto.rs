use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrafficCount {
    pub timestamp: String,
    pub count: u64,
    pub uniques: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrafficOverview {
    pub count: u64,
    pub uniques: u64,
    pub views: Option<Vec<TrafficCount>>,
    pub clones: Option<Vec<TrafficCount>>,
}

pub fn parse_traffic_overview(payload: &str) -> Result<TrafficOverview, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse traffic payload: {}", err),
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_views_payload() {
        let json = r#"{"count":10,"uniques":5,"views":[{"timestamp":"2026-03-01T00:00:00Z","count":2,"uniques":1}]}"#;
        let parsed = parse_traffic_overview(json).expect("payload should parse");
        assert_eq!(parsed.count, 10);
        assert_eq!(parsed.views.expect("views should exist").len(), 1);
    }
}
