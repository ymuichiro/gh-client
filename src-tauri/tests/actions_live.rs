use gh_client_backend::core::command_registry::CommandRegistry;
use gh_client_backend::core::executor::{CommandExecutor, ProcessRunner};
use gh_client_backend::core::observability::TraceContext;
use gh_client_backend::features::actions::service::ActionsService;

#[test]
fn live_list_workflows_and_runs_against_real_gh() {
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
    let service = ActionsService::new(registry, executor);

    let trace_wf = TraceContext::new("live-test-workflows-list");
    let workflows_result = service.list_workflows(&owner, &repo, 20, &trace_wf);
    assert!(
        workflows_result.is_ok(),
        "live workflow list should succeed: {:?}",
        workflows_result.err()
    );

    let trace_runs = TraceContext::new("live-test-runs-list");
    let runs_result = service.list_runs(&owner, &repo, 20, &trace_runs);
    assert!(
        runs_result.is_ok(),
        "live runs list should succeed: {:?}",
        runs_result.err()
    );
}
