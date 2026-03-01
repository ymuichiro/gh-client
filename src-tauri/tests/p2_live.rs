use gh_client_backend::core::command_registry::CommandRegistry;
use gh_client_backend::core::error::ErrorCode;
use gh_client_backend::core::executor::{CommandExecutor, ProcessRunner};
use gh_client_backend::core::observability::TraceContext;
use gh_client_backend::core::policy_guard::RepoPermission;
use gh_client_backend::features::discussions::service::DiscussionsService;
use gh_client_backend::features::insights::service::InsightsService;
use gh_client_backend::features::pages::service::PagesService;
use gh_client_backend::features::projects::service::ProjectsService;
use gh_client_backend::features::rulesets::service::RulesetsService;
use gh_client_backend::features::wiki::service::WikiService;

fn should_skip_feature_error(code: ErrorCode, message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();

    matches!(code, ErrorCode::NotFound | ErrorCode::PermissionDenied)
        || normalized.contains("disabled")
        || normalized.contains("not enabled")
        || normalized.contains("must have admin rights")
        || normalized.contains("required scopes")
}

#[test]
fn live_p2_feature_read_flows_against_real_gh() {
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

    let projects_service = ProjectsService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let discussions_service = DiscussionsService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let wiki_service = WikiService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let insights_service = InsightsService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let pages_service = PagesService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );
    let rulesets_service = RulesetsService::new(
        CommandRegistry::with_defaults(),
        CommandExecutor::new(ProcessRunner, false),
    );

    let projects = projects_service.list(
        &owner,
        &repo,
        20,
        &TraceContext::new("live-p2-projects-list"),
    );
    if let Err(err) = &projects {
        if should_skip_feature_error(err.code, &err.message) {
            eprintln!(
                "skip projects check for this repository/token: {}",
                err.message
            );
        } else {
            panic!("live projects list should succeed: {:?}", err);
        }
    }

    let discussions = discussions_service.list(
        &owner,
        &repo,
        20,
        &TraceContext::new("live-p2-discussions-list"),
    );
    if let Err(err) = &discussions {
        if should_skip_feature_error(err.code, &err.message) {
            eprintln!(
                "skip discussions check for this repository: {}",
                err.message
            );
        } else {
            panic!("live discussions list should succeed: {:?}", err);
        }
    }

    let wiki = wiki_service.get(&owner, &repo, &TraceContext::new("live-p2-wiki-get"));
    assert!(
        wiki.is_ok(),
        "live wiki get should succeed: {:?}",
        wiki.err()
    );

    let pages = pages_service.get(&owner, &repo, &TraceContext::new("live-p2-pages-get"));
    if let Err(err) = &pages {
        if should_skip_feature_error(err.code, &err.message) {
            eprintln!("skip pages check for this repository: {}", err.message);
        } else {
            panic!("live pages get should succeed: {:?}", err);
        }
    }

    let rulesets = rulesets_service.list(
        RepoPermission::Admin,
        &owner,
        &repo,
        &TraceContext::new("live-p2-rulesets-list"),
    );
    if let Err(err) = &rulesets {
        if should_skip_feature_error(err.code, &err.message) {
            eprintln!("skip rulesets check for this repository: {}", err.message);
        } else {
            panic!("live rulesets list should succeed: {:?}", err);
        }
    }

    let views =
        insights_service.get_views(&owner, &repo, &TraceContext::new("live-p2-insights-views"));
    if let Err(err) = &views {
        if should_skip_feature_error(err.code, &err.message) {
            eprintln!(
                "skip insights views check for this repository: {}",
                err.message
            );
        } else {
            panic!("live insights views should succeed: {:?}", err);
        }
    }

    let clones =
        insights_service.get_clones(&owner, &repo, &TraceContext::new("live-p2-insights-clones"));
    if let Err(err) = &clones {
        if should_skip_feature_error(err.code, &err.message) {
            eprintln!(
                "skip insights clones check for this repository: {}",
                err.message
            );
        } else {
            panic!("live insights clones should succeed: {:?}", err);
        }
    }
}
