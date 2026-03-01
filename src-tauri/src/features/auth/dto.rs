use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GhAuthStatus {
    pub host: String,
    pub logged_in: bool,
    pub account: Option<String>,
    pub active_account: Option<bool>,
    pub git_protocol: Option<String>,
    pub token_scopes: Vec<String>,
}

impl GhAuthStatus {
    pub fn logged_out_default(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            logged_in: false,
            account: None,
            active_account: None,
            git_protocol: None,
            token_scopes: Vec::new(),
        }
    }
}

pub fn parse_gh_auth_status(payload: &str) -> Result<GhAuthStatus, AppError> {
    if payload.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::UpstreamError,
            "failed to parse gh auth status payload: empty output",
            false,
        ));
    }

    let mut host = "github.com".to_string();
    let mut logged_in = false;
    let mut account = None;
    let mut active_account = None;
    let mut git_protocol = None;
    let mut token_scopes = Vec::new();

    for line in payload.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if !trimmed.starts_with('-')
            && !trimmed.starts_with('✓')
            && !trimmed.starts_with('!')
            && !trimmed.starts_with('x')
            && trimmed.contains(".")
            && !trimmed.contains(' ')
        {
            host = trimmed.to_string();
            continue;
        }

        if let Some(index) = trimmed.find("Logged in to ") {
            let segment = &trimmed[index + "Logged in to ".len()..];
            if let Some(account_start) = segment.find(" account ") {
                let account_segment = &segment[account_start + " account ".len()..];
                let end = account_segment
                    .find(' ')
                    .or_else(|| account_segment.find('('))
                    .unwrap_or(account_segment.len());
                let value = account_segment[..end].trim();
                if !value.is_empty() {
                    account = Some(value.to_string());
                    logged_in = true;
                }
            }
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- Active account:") {
            let normalized = value.trim().to_ascii_lowercase();
            active_account = match normalized.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- Git operations protocol:") {
            let value = value.trim();
            if !value.is_empty() {
                git_protocol = Some(value.to_string());
            }
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- Token scopes:") {
            token_scopes = value
                .split(',')
                .map(|scope| scope.trim().trim_matches('\'').trim_matches('"'))
                .filter(|scope| !scope.is_empty())
                .map(ToString::to_string)
                .collect();
            continue;
        }

        if trimmed.contains("not logged") {
            logged_in = false;
        }
    }

    Ok(GhAuthStatus {
        host,
        logged_in,
        account,
        active_account,
        git_protocol,
        token_scopes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_logged_in_status_payload() {
        let payload = r#"github.com
  ✓ Logged in to github.com account octocat (keyring)
  - Active account: true
  - Git operations protocol: https
  - Token: gho_********************
  - Token scopes: 'repo', 'read:org', 'workflow'"#;

        let status = parse_gh_auth_status(payload).expect("payload should parse");
        assert_eq!(status.host, "github.com");
        assert!(status.logged_in);
        assert_eq!(status.account.as_deref(), Some("octocat"));
        assert_eq!(status.active_account, Some(true));
        assert_eq!(status.git_protocol.as_deref(), Some("https"));
        assert_eq!(status.token_scopes.len(), 3);
    }

    #[test]
    fn parses_logged_out_status_payload() {
        let payload = "github.com\n  ! not logged into any hosts";
        let status = parse_gh_auth_status(payload).expect("payload should parse");
        assert_eq!(status.host, "github.com");
        assert!(!status.logged_in);
        assert!(status.account.is_none());
    }
}
