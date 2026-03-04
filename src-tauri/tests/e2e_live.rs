use gh_client_backend::core::command_registry::CommandRegistry;
use gh_client_backend::core::executor::{CommandExecutor, ProcessRunner};
use gh_client_backend::core::observability::TraceContext;
use gh_client_backend::core::policy_guard::RepoPermission;
use gh_client_backend::features::actions::service::ActionsService;
use gh_client_backend::features::issues::service::IssuesService;
use gh_client_backend::features::pull_requests::service::PullRequestsService;
use gh_client_backend::features::releases::service::ReleasesService;
use gh_client_backend::features::repositories::service::RepositoriesService;
use gh_client_backend::features::settings::service::{SecretApp, SettingsService};

#[test]
fn live_cross_feature_read_only_flow() {
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

    let repo_service = RepositoriesService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let pr_service = PullRequestsService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let issue_service = IssuesService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let actions_service = ActionsService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let release_service = ReleasesService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let settings_service = SettingsService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );

    let repos = repo_service.list(&owner, 20, &TraceContext::new("e2e-repo-list"));
    assert!(
        repos.is_ok(),
        "e2e: repository list failed: {:?}",
        repos.err()
    );

    let branches =
        repo_service.list_branches(&owner, &repo, 20, &TraceContext::new("e2e-branch-list"));
    assert!(
        branches.is_ok(),
        "e2e: branch list failed: {:?}",
        branches.err()
    );

    let commits = repo_service.list_commits(
        &owner,
        &repo,
        None,
        20,
        &TraceContext::new("e2e-commit-list"),
    );
    assert!(
        commits.is_ok(),
        "e2e: commit list failed: {:?}",
        commits.err()
    );

    let prs = pr_service.list(&owner, &repo, 20, &TraceContext::new("e2e-pr-list"));
    assert!(
        prs.is_ok(),
        "e2e: pull request list failed: {:?}",
        prs.as_ref().err()
    );

    if let Ok(pr_items) = prs {
        if let Some(first_pr) = pr_items.first() {
            let pr_detail = pr_service.view(
                &owner,
                &repo,
                first_pr.number,
                &TraceContext::new("e2e-pr-view"),
            );
            assert!(
                pr_detail.is_ok(),
                "e2e: pull request detail failed: {:?}",
                pr_detail.err()
            );

            let issue_comments = pr_service.list_issue_comments(
                &owner,
                &repo,
                first_pr.number,
                &TraceContext::new("e2e-pr-issue-comments"),
            );
            assert!(
                issue_comments.is_ok(),
                "e2e: pull request issue comments failed: {:?}",
                issue_comments.err()
            );

            let review_comments = pr_service.list_review_comments(
                &owner,
                &repo,
                first_pr.number,
                &TraceContext::new("e2e-pr-review-comments"),
            );
            assert!(
                review_comments.is_ok(),
                "e2e: pull request review comments failed: {:?}",
                review_comments.err()
            );

            let review_threads = pr_service.list_review_threads(
                &owner,
                &repo,
                first_pr.number,
                &TraceContext::new("e2e-pr-review-threads"),
            );
            assert!(
                review_threads.is_ok(),
                "e2e: pull request review threads failed: {:?}",
                review_threads.err()
            );

            let diff_files = pr_service.list_diff_files(
                &owner,
                &repo,
                first_pr.number,
                &TraceContext::new("e2e-pr-diff-files"),
            );
            assert!(
                diff_files.is_ok(),
                "e2e: pull request diff files failed: {:?}",
                diff_files.err()
            );

            let raw_diff = pr_service.get_raw_diff(
                &owner,
                &repo,
                first_pr.number,
                &TraceContext::new("e2e-pr-raw-diff"),
            );
            assert!(
                raw_diff.is_ok(),
                "e2e: pull request raw diff failed: {:?}",
                raw_diff.err()
            );
        }
    }

    let issues = issue_service.list(&owner, &repo, 20, &TraceContext::new("e2e-issue-list"));
    assert!(issues.is_ok(), "e2e: issue list failed: {:?}", issues.err());

    let workflows =
        actions_service.list_workflows(&owner, &repo, 20, &TraceContext::new("e2e-workflow-list"));
    assert!(
        workflows.is_ok(),
        "e2e: workflow list failed: {:?}",
        workflows.err()
    );

    let runs = actions_service.list_runs(&owner, &repo, 20, &TraceContext::new("e2e-run-list"));
    assert!(runs.is_ok(), "e2e: run list failed: {:?}", runs.err());

    if let Ok(run_items) = runs {
        if let Some(first_run) = run_items.first() {
            let run_detail = actions_service.view_run(
                &owner,
                &repo,
                first_run.database_id,
                &TraceContext::new("e2e-run-detail"),
            );
            assert!(
                run_detail.is_ok(),
                "e2e: run detail failed: {:?}",
                run_detail.err()
            );

            let run_logs = actions_service.view_run_logs(
                &owner,
                &repo,
                first_run.database_id,
                &TraceContext::new("e2e-run-logs"),
            );
            assert!(
                run_logs.is_ok(),
                "e2e: run logs failed: {:?}",
                run_logs.err()
            );
        }
    }

    let releases = release_service.list(&owner, &repo, 20, &TraceContext::new("e2e-release-list"));
    assert!(
        releases.is_ok(),
        "e2e: release list failed: {:?}",
        releases.err()
    );

    let collaborators = settings_service.list_collaborators(
        RepoPermission::Admin,
        &owner,
        &repo,
        &TraceContext::new("e2e-settings-collaborators-list"),
    );
    assert!(
        collaborators.is_ok(),
        "e2e: collaborators list failed: {:?}",
        collaborators.err()
    );

    let secrets = settings_service.list_secrets(
        RepoPermission::Admin,
        &owner,
        &repo,
        Some(SecretApp::Actions),
        &TraceContext::new("e2e-settings-secrets-list"),
    );
    assert!(
        secrets.is_ok(),
        "e2e: secrets list failed: {:?}",
        secrets.err()
    );

    let variables = settings_service.list_variables(
        RepoPermission::Admin,
        &owner,
        &repo,
        &TraceContext::new("e2e-settings-variables-list"),
    );
    assert!(
        variables.is_ok(),
        "e2e: variables list failed: {:?}",
        variables.err()
    );

    let webhooks = settings_service.list_webhooks(
        RepoPermission::Admin,
        &owner,
        &repo,
        &TraceContext::new("e2e-settings-webhooks-list"),
    );
    assert!(
        webhooks.is_ok(),
        "e2e: webhooks list failed: {:?}",
        webhooks.err()
    );

    let deploy_keys = settings_service.list_deploy_keys(
        RepoPermission::Admin,
        &owner,
        &repo,
        &TraceContext::new("e2e-settings-deploy-keys-list"),
    );
    assert!(
        deploy_keys.is_ok(),
        "e2e: deploy keys list failed: {:?}",
        deploy_keys.err()
    );
}
