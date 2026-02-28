use gh_client_backend::core::command_registry::CommandRegistry;
use gh_client_backend::core::executor::{CommandExecutor, ProcessRunner};
use gh_client_backend::core::observability::TraceContext;
use gh_client_backend::features::pull_requests::service::PullRequestsService;

#[test]
fn live_list_pull_requests_against_real_gh() {
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

    let trace = TraceContext::new("live-test-pr-list");
    let result = service.list(&owner, &repo, 20, &trace);

    assert!(
        result.is_ok(),
        "live pull request list should succeed: {:?}",
        result.err()
    );
}
