use gh_client_backend::core::command_registry::CommandRegistry;
use gh_client_backend::core::executor::{CommandExecutor, ProcessRunner};
use gh_client_backend::core::observability::TraceContext;
use gh_client_backend::features::repositories::service::RepositoriesService;

#[test]
fn live_list_repositories_against_real_gh() {
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

    let registry = CommandRegistry::with_defaults();
    let executor = CommandExecutor::new(ProcessRunner, false);
    let service = RepositoriesService::new(registry, executor);

    let trace = TraceContext::new("live-test-list");
    let result = service.list(&owner, 20, &trace);

    assert!(
        result.is_ok(),
        "live list should succeed: {:?}",
        result.err()
    );
}
