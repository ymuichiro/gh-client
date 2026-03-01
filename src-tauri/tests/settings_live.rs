use gh_client_backend::core::command_registry::CommandRegistry;
use gh_client_backend::core::executor::{CommandExecutor, ProcessRunner};
use gh_client_backend::core::observability::TraceContext;
use gh_client_backend::core::policy_guard::RepoPermission;
use gh_client_backend::features::settings::service::{SecretApp, SettingsService};

#[test]
fn live_list_settings_resources_against_real_gh() {
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
    let service = SettingsService::new(registry, executor);

    let trace_collaborators = TraceContext::new("live-test-settings-collaborators-list");
    let result =
        service.list_collaborators(RepoPermission::Admin, &owner, &repo, &trace_collaborators);

    assert!(
        result.is_ok(),
        "live collaborators list should succeed: {:?}",
        result.err()
    );

    let trace_secrets = TraceContext::new("live-test-settings-secrets-list");
    let secrets = service.list_secrets(
        RepoPermission::Admin,
        &owner,
        &repo,
        Some(SecretApp::Actions),
        &trace_secrets,
    );
    assert!(
        secrets.is_ok(),
        "live secrets list should succeed: {:?}",
        secrets.err()
    );

    let trace_variables = TraceContext::new("live-test-settings-variables-list");
    let variables = service.list_variables(RepoPermission::Admin, &owner, &repo, &trace_variables);
    assert!(
        variables.is_ok(),
        "live variables list should succeed: {:?}",
        variables.err()
    );

    let trace_webhooks = TraceContext::new("live-test-settings-webhooks-list");
    let webhooks = service.list_webhooks(RepoPermission::Admin, &owner, &repo, &trace_webhooks);
    assert!(
        webhooks.is_ok(),
        "live webhooks list should succeed: {:?}",
        webhooks.err()
    );

    let trace_deploy_keys = TraceContext::new("live-test-settings-deploy-keys-list");
    let deploy_keys =
        service.list_deploy_keys(RepoPermission::Admin, &owner, &repo, &trace_deploy_keys);
    assert!(
        deploy_keys.is_ok(),
        "live deploy keys list should succeed: {:?}",
        deploy_keys.err()
    );
}
