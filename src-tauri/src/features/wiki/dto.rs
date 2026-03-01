use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiInfo {
    pub has_wiki: bool,
    pub html_url: String,
    pub wiki_url: String,
}

#[derive(Debug, Deserialize)]
struct RepositoryWikiPayload {
    #[serde(rename = "has_wiki")]
    has_wiki: bool,
    html_url: String,
}

pub fn parse_wiki_info(payload: &str) -> Result<WikiInfo, AppError> {
    let parsed: RepositoryWikiPayload = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse wiki payload: {}", err),
            false,
        )
    })?;

    let wiki_url = format!("{}/wiki", parsed.html_url.trim_end_matches('/'));

    Ok(WikiInfo {
        has_wiki: parsed.has_wiki,
        html_url: parsed.html_url,
        wiki_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wiki_payload() {
        let json = r#"{"has_wiki":true,"html_url":"https://github.com/octocat/hello"}"#;
        let info = parse_wiki_info(json).expect("payload should parse");
        assert!(info.has_wiki);
        assert_eq!(info.wiki_url, "https://github.com/octocat/hello/wiki");
    }
}
