use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PagesSource {
    pub branch: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PagesInfo {
    pub url: Option<String>,
    pub status: Option<String>,
    pub cname: Option<String>,
    pub custom_404: Option<bool>,
    pub html_url: Option<String>,
    pub public: Option<bool>,
    pub source: Option<PagesSource>,
}

pub fn parse_pages_info(payload: &str) -> Result<PagesInfo, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse pages payload: {}", err),
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pages_payload() {
        let json = r#"{"url":"https://api.github.com/repos/octocat/hello/pages","status":"built","cname":null,"custom_404":false,"html_url":"https://octocat.github.io/hello/","public":true,"source":{"branch":"main","path":"/"}}"#;
        let info = parse_pages_info(json).expect("payload should parse");
        assert_eq!(info.status.as_deref(), Some("built"));
        assert_eq!(info.source.and_then(|s| s.branch).as_deref(), Some("main"));
    }
}
