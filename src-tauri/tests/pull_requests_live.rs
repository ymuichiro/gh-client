use gh_client_backend::core::command_registry::CommandRegistry;
use gh_client_backend::core::executor::{CommandExecutor, ProcessRunner};
use gh_client_backend::core::observability::TraceContext;
use gh_client_backend::core::policy_guard::RepoPermission;
use gh_client_backend::features::pull_requests::service::{
    CommentPullRequestInput, PullRequestsService, ReplyReviewCommentInput, ResolveReviewThreadInput,
};

#[test]
fn live_pull_requests_feature_against_real_gh() {
    if std::env::var("GH_CLIENT_LIVE_TEST").ok().as_deref() != Some("1") {
        eprintln!("skip live test: set GH_CLIENT_LIVE_TEST=1");
        return;
    }

    let owner = match std::env::var("GH_TEST_OWNER") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            eprintln!("skip live test: set GH_TEST_OWNER");
            return;
        }
    };

    let repo = match std::env::var("GH_TEST_REPO") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            eprintln!("skip live test: set GH_TEST_REPO");
            return;
        }
    };

    let registry = CommandRegistry::with_defaults();
    let executor = CommandExecutor::new(ProcessRunner, false);
    let service = PullRequestsService::new(registry, executor);

    let list_trace = TraceContext::new("live-test-pr-list");
    let list_result = service.list(&owner, &repo, 20, &list_trace);

    assert!(
        list_result.is_ok(),
        "live pull request list should succeed: {:?}",
        list_result.err()
    );

    let prs = list_result.expect("pr list should be ok");
    let pr_number = if let Ok(env_pr_number) = std::env::var("GH_TEST_PR_NUMBER") {
        match env_pr_number.parse::<u64>() {
            Ok(value) if value > 0 => value,
            _ => {
                eprintln!("skip pr deep tests: GH_TEST_PR_NUMBER is invalid");
                return;
            }
        }
    } else if let Some(first) = prs.first() {
        first.number
    } else {
        eprintln!("skip pr deep tests: no pull requests found and GH_TEST_PR_NUMBER not set");
        return;
    };

    let detail = service.view(
        &owner,
        &repo,
        pr_number,
        &TraceContext::new("live-test-pr-view"),
    );
    assert!(
        detail.is_ok(),
        "live pr view should succeed: {:?}",
        detail.err()
    );

    let issue_comments = service.list_issue_comments(
        &owner,
        &repo,
        pr_number,
        &TraceContext::new("live-test-pr-issue-comments"),
    );
    assert!(
        issue_comments.is_ok(),
        "live pr issue comments list should succeed: {:?}",
        issue_comments.err()
    );

    let review_comments = service.list_review_comments(
        &owner,
        &repo,
        pr_number,
        &TraceContext::new("live-test-pr-review-comments"),
    );
    assert!(
        review_comments.is_ok(),
        "live pr review comments list should succeed: {:?}",
        review_comments.err()
    );

    let review_threads_result = service.list_review_threads(
        &owner,
        &repo,
        pr_number,
        &TraceContext::new("live-test-pr-review-threads"),
    );
    assert!(
        review_threads_result.is_ok(),
        "live pr review threads list should succeed: {:?}",
        review_threads_result.as_ref().err()
    );

    let diff_files = service.list_diff_files(
        &owner,
        &repo,
        pr_number,
        &TraceContext::new("live-test-pr-diff-files"),
    );
    assert!(
        diff_files.is_ok(),
        "live pr diff files list should succeed: {:?}",
        diff_files.err()
    );

    let raw_diff = service.get_raw_diff(
        &owner,
        &repo,
        pr_number,
        &TraceContext::new("live-test-pr-raw-diff"),
    );
    assert!(
        raw_diff.is_ok(),
        "live pr raw diff should succeed: {:?}",
        raw_diff.err()
    );

    if std::env::var("GH_CLIENT_LIVE_WRITE_TEST").ok().as_deref() != Some("1") {
        eprintln!("skip write live tests: set GH_CLIENT_LIVE_WRITE_TEST=1");
        return;
    }

    let create_comment_input = CommentPullRequestInput {
        owner: owner.clone(),
        repo: repo.clone(),
        number: pr_number,
        body: format!(
            "gh-client live test comment ({})",
            TraceContext::new("live-test-pr-write-comment").request_id
        ),
    };
    let create_comment_result = service.create_issue_comment(
        RepoPermission::Write,
        &create_comment_input,
        &TraceContext::new("live-test-pr-write-comment"),
    );
    assert!(
        create_comment_result.is_ok(),
        "live pr issue comment create should succeed: {:?}",
        create_comment_result.err()
    );

    let reply_comment_id = match std::env::var("GH_TEST_REVIEW_COMMENT_ID") {
        Ok(v) => match v.parse::<u64>() {
            Ok(parsed) if parsed > 0 => Some(parsed),
            _ => {
                eprintln!("skip review comment reply live test: GH_TEST_REVIEW_COMMENT_ID invalid");
                None
            }
        },
        Err(_) => {
            eprintln!("skip review comment reply live test: set GH_TEST_REVIEW_COMMENT_ID");
            None
        }
    };

    if let Some(comment_id) = reply_comment_id {
        let reply_input = ReplyReviewCommentInput {
            owner: owner.clone(),
            repo: repo.clone(),
            number: pr_number,
            comment_id,
            body: "gh-client live test reply".to_string(),
        };

        let reply_result = service.reply_review_comment(
            RepoPermission::Write,
            &reply_input,
            &TraceContext::new("live-test-pr-write-reply"),
        );

        assert!(
            reply_result.is_ok(),
            "live pr review comment reply should succeed: {:?}",
            reply_result.err()
        );
    }

    let thread_id = match std::env::var("GH_TEST_REVIEW_THREAD_ID") {
        Ok(v) if !v.trim().is_empty() => Some(v),
        _ => review_threads_result.ok().and_then(|threads| {
            threads
                .iter()
                .find(|thread| !thread.thread_id.is_empty())
                .map(|thread| thread.thread_id.clone())
        }),
    };

    if let Some(thread_id) = thread_id {
        let thread_input = ResolveReviewThreadInput { thread_id };

        let resolve_result = service.resolve_review_thread(
            RepoPermission::Write,
            &thread_input,
            &TraceContext::new("live-test-pr-thread-resolve"),
        );
        assert!(
            resolve_result.is_ok(),
            "live pr review thread resolve should succeed: {:?}",
            resolve_result.err()
        );

        let unresolve_result = service.unresolve_review_thread(
            RepoPermission::Write,
            &thread_input,
            &TraceContext::new("live-test-pr-thread-unresolve"),
        );
        assert!(
            unresolve_result.is_ok(),
            "live pr review thread unresolve should succeed: {:?}",
            unresolve_result.err()
        );
    } else {
        eprintln!("skip review thread resolve live test: no thread id available");
    }
}
