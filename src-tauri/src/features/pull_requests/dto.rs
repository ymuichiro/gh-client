use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestAuthor {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestLabel {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestAssignee {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestSummary {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub url: String,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    pub author: Option<PullRequestAuthor>,
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
    #[serde(rename = "baseRefName")]
    pub base_ref_name: String,
    pub labels: Option<Vec<PullRequestLabel>>,
    pub assignees: Option<Vec<PullRequestAssignee>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "reviewDecision")]
    pub review_decision: Option<String>,
    #[serde(rename = "reviewRequests")]
    pub review_requests: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestCreated {
    pub number: u64,
    #[serde(rename = "html_url")]
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequestDetail {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub url: String,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    pub author: Option<PullRequestAuthor>,
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
    #[serde(rename = "baseRefName")]
    pub base_ref_name: String,
    #[serde(rename = "mergeStateStatus")]
    pub merge_state_status: Option<String>,
    #[serde(rename = "reviewDecision")]
    pub review_decision: Option<String>,
    #[serde(rename = "statusCheckRollup")]
    pub status_check_rollup: Option<Value>,
    pub additions: u64,
    pub deletions: u64,
    #[serde(rename = "changedFiles")]
    pub changed_files: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestCommentKind {
    IssueComment,
    ReviewComment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestComment {
    pub id: Option<u64>,
    pub kind: PullRequestCommentKind,
    pub body: String,
    pub created_at: String,
    pub author: Option<PullRequestAuthor>,
    pub url: Option<String>,
    pub reply_to_comment_id: Option<u64>,
    pub path: Option<String>,
    pub line: Option<u64>,
    pub side: Option<String>,
    pub commit_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestReviewThread {
    pub thread_id: String,
    pub is_resolved: bool,
    pub is_outdated: bool,
    pub path: Option<String>,
    pub line: Option<u64>,
    pub comments: Vec<PullRequestComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestDiffFile {
    pub filename: String,
    pub status: String,
    pub additions: u64,
    pub deletions: u64,
    pub changes: u64,
    #[serde(rename = "blob_url")]
    pub blob_url: Option<String>,
    #[serde(rename = "raw_url")]
    pub raw_url: Option<String>,
    pub patch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestRawDiff {
    pub text: String,
}

#[derive(Debug, Deserialize)]
struct ApiUser {
    login: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IssueCommentResponse {
    id: u64,
    body: String,
    created_at: String,
    user: Option<ApiUser>,
    html_url: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReviewCommentResponse {
    id: u64,
    body: String,
    created_at: String,
    user: Option<ApiUser>,
    html_url: Option<String>,
    url: Option<String>,
    in_reply_to_id: Option<u64>,
    path: Option<String>,
    line: Option<u64>,
    side: Option<String>,
    commit_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadsResponse {
    data: ReviewThreadsData,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadsData {
    repository: Option<ReviewThreadsRepository>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadsRepository {
    #[serde(rename = "pullRequest")]
    pull_request: Option<ReviewThreadsPullRequest>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadsPullRequest {
    #[serde(rename = "reviewThreads")]
    review_threads: ReviewThreadConnection,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadConnection {
    nodes: Vec<ReviewThreadNode>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadNode {
    id: String,
    #[serde(rename = "isResolved")]
    is_resolved: bool,
    #[serde(rename = "isOutdated")]
    is_outdated: bool,
    path: Option<String>,
    line: Option<u64>,
    comments: ReviewThreadCommentConnection,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadCommentConnection {
    nodes: Vec<ReviewThreadCommentNode>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadCommentNode {
    #[serde(rename = "databaseId")]
    database_id: Option<u64>,
    body: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    author: Option<ApiUser>,
    #[serde(rename = "replyTo")]
    reply_to: Option<ReviewThreadReplyTo>,
    path: Option<String>,
    line: Option<u64>,
    #[serde(rename = "diffSide")]
    diff_side: Option<String>,
    commit: Option<ReviewThreadCommit>,
    #[serde(rename = "originalCommit")]
    original_commit: Option<ReviewThreadCommit>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadReplyTo {
    #[serde(rename = "databaseId")]
    database_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadCommit {
    oid: Option<String>,
}

fn map_author(user: Option<ApiUser>) -> Option<PullRequestAuthor> {
    user.and_then(|value| {
        value.login.map(|login| PullRequestAuthor {
            login: login.to_string(),
        })
    })
}

fn map_issue_comment(value: IssueCommentResponse) -> PullRequestComment {
    PullRequestComment {
        id: Some(value.id),
        kind: PullRequestCommentKind::IssueComment,
        body: value.body,
        created_at: value.created_at,
        author: map_author(value.user),
        url: value.html_url.or(value.url),
        reply_to_comment_id: None,
        path: None,
        line: None,
        side: None,
        commit_id: None,
    }
}

fn map_review_comment(value: ReviewCommentResponse) -> PullRequestComment {
    PullRequestComment {
        id: Some(value.id),
        kind: PullRequestCommentKind::ReviewComment,
        body: value.body,
        created_at: value.created_at,
        author: map_author(value.user),
        url: value.html_url.or(value.url),
        reply_to_comment_id: value.in_reply_to_id,
        path: value.path,
        line: value.line,
        side: value.side,
        commit_id: value.commit_id,
    }
}

pub fn parse_pull_request_summaries(payload: &str) -> Result<Vec<PullRequestSummary>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse pull request list payload: {}", err),
            false,
        )
    })
}

pub fn parse_pull_request_created(payload: &str) -> Result<PullRequestCreated, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse pull request create payload: {}", err),
            false,
        )
    })
}

pub fn parse_pull_request_detail(payload: &str) -> Result<PullRequestDetail, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse pull request detail payload: {}", err),
            false,
        )
    })
}

pub fn parse_issue_comments(payload: &str) -> Result<Vec<PullRequestComment>, AppError> {
    let parsed: Vec<IssueCommentResponse> = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!(
                "failed to parse pull request issue comments payload: {}",
                err
            ),
            false,
        )
    })?;

    Ok(parsed.into_iter().map(map_issue_comment).collect())
}

pub fn parse_issue_comment(payload: &str) -> Result<PullRequestComment, AppError> {
    let parsed: IssueCommentResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!(
                "failed to parse pull request issue comment payload: {}",
                err
            ),
            false,
        )
    })?;

    Ok(map_issue_comment(parsed))
}

pub fn parse_review_comments(payload: &str) -> Result<Vec<PullRequestComment>, AppError> {
    let parsed: Vec<ReviewCommentResponse> = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!(
                "failed to parse pull request review comments payload: {}",
                err
            ),
            false,
        )
    })?;

    Ok(parsed.into_iter().map(map_review_comment).collect())
}

pub fn parse_review_comment(payload: &str) -> Result<PullRequestComment, AppError> {
    let parsed: ReviewCommentResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!(
                "failed to parse pull request review comment payload: {}",
                err
            ),
            false,
        )
    })?;

    Ok(map_review_comment(parsed))
}

pub fn parse_review_threads(payload: &str) -> Result<Vec<PullRequestReviewThread>, AppError> {
    let parsed: ReviewThreadsResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!(
                "failed to parse pull request review threads payload: {}",
                err
            ),
            false,
        )
    })?;

    Ok(parsed
        .data
        .repository
        .and_then(|repo| repo.pull_request)
        .map(|pr| {
            pr.review_threads
                .nodes
                .into_iter()
                .map(|thread| PullRequestReviewThread {
                    thread_id: thread.id,
                    is_resolved: thread.is_resolved,
                    is_outdated: thread.is_outdated,
                    path: thread.path,
                    line: thread.line,
                    comments: thread
                        .comments
                        .nodes
                        .into_iter()
                        .map(|comment| PullRequestComment {
                            id: comment.database_id,
                            kind: PullRequestCommentKind::ReviewComment,
                            body: comment.body,
                            created_at: comment.created_at,
                            author: map_author(comment.author),
                            url: None,
                            reply_to_comment_id: comment.reply_to.and_then(|v| v.database_id),
                            path: comment.path,
                            line: comment.line,
                            side: comment.diff_side,
                            commit_id: comment
                                .commit
                                .and_then(|value| value.oid)
                                .or(comment.original_commit.and_then(|value| value.oid)),
                        })
                        .collect(),
                })
                .collect()
        })
        .unwrap_or_default())
}

pub fn parse_pull_request_diff_files(payload: &str) -> Result<Vec<PullRequestDiffFile>, AppError> {
    serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse pull request diff files payload: {}", err),
            false,
        )
    })
}

pub fn parse_pull_request_raw_diff(payload: &str) -> PullRequestRawDiff {
    PullRequestRawDiff {
        text: payload.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pull_request_list_payload() {
        let json = r#"
        [
          {
            "number": 10,
            "title": "Add feature",
            "state": "OPEN",
            "url": "https://github.com/octocat/hello/pull/10",
            "isDraft": false,
            "author": {"login": "octocat"},
            "headRefName": "feature-1",
            "baseRefName": "main"
          }
        ]
        "#;

        let prs = parse_pull_request_summaries(json).expect("list payload should parse");
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 10);
        assert_eq!(prs[0].head_ref_name, "feature-1");
    }

    #[test]
    fn list_parse_returns_error_for_invalid_shape() {
        let json = r#"[{"number":"bad"}]"#;
        let err = parse_pull_request_summaries(json).expect_err("invalid list shape should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }

    #[test]
    fn parses_pull_request_create_payload() {
        let json = r#"
        {
          "number": 12,
          "html_url": "https://github.com/octocat/hello/pull/12",
          "state": "OPEN"
        }
        "#;

        let created = parse_pull_request_created(json).expect("create payload should parse");
        assert_eq!(created.number, 12);
        assert_eq!(created.state, "OPEN");
    }

    #[test]
    fn create_parse_returns_error_for_invalid_shape() {
        let json = r#"{"html_url":true}"#;
        let err = parse_pull_request_created(json).expect_err("invalid create shape should fail");
        assert_eq!(err.code, ErrorCode::UpstreamError);
    }

    #[test]
    fn parses_pull_request_detail_payload() {
        let json = r#"{
          "number": 3,
          "title": "Feature",
          "body": "body",
          "state": "OPEN",
          "url": "https://github.com/octocat/hello/pull/3",
          "isDraft": false,
          "author": {"login":"octocat"},
          "headRefName": "feature",
          "baseRefName": "main",
          "mergeStateStatus": "CLEAN",
          "reviewDecision": "APPROVED",
          "statusCheckRollup": {"state":"SUCCESS"},
          "additions": 10,
          "deletions": 2,
          "changedFiles": 1
        }"#;

        let detail = parse_pull_request_detail(json).expect("detail payload should parse");
        assert_eq!(detail.number, 3);
        assert_eq!(detail.changed_files, 1);
    }

    #[test]
    fn parses_issue_comments_payload() {
        let json = r#"[
          {
            "id": 100,
            "body": "hello",
            "created_at": "2026-03-01T00:00:00Z",
            "user": {"login":"octocat"},
            "html_url":"https://github.com/octocat/hello/pull/3#issuecomment-1"
          }
        ]"#;

        let comments = parse_issue_comments(json).expect("issue comments should parse");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].kind, PullRequestCommentKind::IssueComment);
        assert_eq!(comments[0].id, Some(100));
    }

    #[test]
    fn parses_review_comments_payload() {
        let json = r#"[
          {
            "id": 101,
            "body": "nit",
            "created_at": "2026-03-01T00:00:00Z",
            "user": {"login":"octocat"},
            "html_url":"https://github.com/octocat/hello/pull/3#discussion_r1",
            "in_reply_to_id": 50,
            "path":"src/lib.rs",
            "line": 10,
            "side":"RIGHT",
            "commit_id":"abc123"
          }
        ]"#;

        let comments = parse_review_comments(json).expect("review comments should parse");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].kind, PullRequestCommentKind::ReviewComment);
        assert_eq!(comments[0].reply_to_comment_id, Some(50));
    }

    #[test]
    fn parses_review_threads_payload() {
        let json = r#"{
          "data": {
            "repository": {
              "pullRequest": {
                "reviewThreads": {
                  "nodes": [
                    {
                      "id": "PRRT_1",
                      "isResolved": false,
                      "isOutdated": false,
                      "path": "src/lib.rs",
                      "line": 10,
                      "comments": {
                        "nodes": [
                          {
                            "databaseId": 101,
                            "body": "nit",
                            "createdAt": "2026-03-01T00:00:00Z",
                            "author": {"login":"octocat"},
                            "replyTo": {"databaseId": 100},
                            "path": "src/lib.rs",
                            "line": 10,
                            "diffSide": "RIGHT",
                            "commit": {"oid":"abc123"},
                            "originalCommit": null
                          }
                        ]
                      }
                    }
                  ]
                }
              }
            }
          }
        }"#;

        let threads = parse_review_threads(json).expect("review threads should parse");
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].thread_id, "PRRT_1");
        assert_eq!(threads[0].comments[0].id, Some(101));
    }

    #[test]
    fn parses_diff_files_payload() {
        let json = r#"[
          {
            "filename":"src/lib.rs",
            "status":"modified",
            "additions":3,
            "deletions":1,
            "changes":4,
            "blob_url":"https://github.com/octocat/hello/blob/main/src/lib.rs",
            "raw_url":"https://raw.githubusercontent.com/octocat/hello/main/src/lib.rs",
            "patch":"@@ -1 +1 @@"
          }
        ]"#;

        let files = parse_pull_request_diff_files(json).expect("diff files should parse");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "src/lib.rs");
    }

    #[test]
    fn wraps_raw_diff_payload() {
        let raw = parse_pull_request_raw_diff("diff --git a/a b/a\n");
        assert!(raw.text.starts_with("diff --git"));
    }
}
