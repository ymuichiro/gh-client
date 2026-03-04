use crate::core::command_registry::CommandRegistry;
use crate::core::error::ErrorCode;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;

use super::dto::{GhAuthStatus, GhOrganization, parse_gh_auth_status, parse_gh_organizations};

pub struct AuthService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
}

impl<R: Runner> AuthService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self { registry, executor }
    }

    pub fn status(
        &self,
        trace: &TraceContext,
    ) -> Result<GhAuthStatus, crate::core::error::AppError> {
        let req = self.registry.build_request("auth.status", &[])?;

        match self.executor.execute(&req, trace) {
            Ok((output, _audit)) => {
                parse_gh_auth_status(&format!("{}\n{}", output.stdout, output.stderr))
            }
            Err(err) if err.code == ErrorCode::AuthRequired => {
                Ok(GhAuthStatus::logged_out_default("github.com"))
            }
            Err(err) => Err(err),
        }
    }

    pub fn list_organizations(
        &self,
        trace: &TraceContext,
    ) -> Result<Vec<GhOrganization>, crate::core::error::AppError> {
        let req = self
            .registry
            .build_request("auth.organizations.list", &[])?;

        match self.executor.execute(&req, trace) {
            Ok((output, _audit)) => parse_gh_organizations(&output.stdout),
            Err(err) if err.code == ErrorCode::AuthRequired => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::core::executor::RawExecutionOutput;

    #[derive(Default)]
    struct RecordingState {
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl RecordingState {
        fn call_count(&self) -> usize {
            self.calls.lock().expect("lock poisoned").len()
        }

        fn last_call(&self) -> Option<(String, Vec<String>)> {
            self.calls.lock().expect("lock poisoned").last().cloned()
        }
    }

    struct RecordingRunner {
        state: Arc<RecordingState>,
        responses: Mutex<VecDeque<RawExecutionOutput>>,
    }

    impl RecordingRunner {
        fn new(responses: Vec<RawExecutionOutput>) -> (Self, Arc<RecordingState>) {
            let state = Arc::new(RecordingState::default());
            (
                Self {
                    state: Arc::clone(&state),
                    responses: Mutex::new(VecDeque::from(responses)),
                },
                state,
            )
        }
    }

    impl Runner for RecordingRunner {
        fn run(&self, program: &str, args: &[String]) -> io::Result<RawExecutionOutput> {
            self.state
                .calls
                .lock()
                .expect("lock poisoned")
                .push((program.to_string(), args.to_vec()));

            let response = self
                .responses
                .lock()
                .expect("lock poisoned")
                .pop_front()
                .unwrap_or(RawExecutionOutput {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                });

            Ok(response)
        }
    }

    fn trace() -> TraceContext {
        TraceContext::new("req-auth-service")
    }

    #[test]
    fn status_executes_gh_auth_status() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: "github.com\n  ✓ Logged in to github.com account octocat (keyring)\n  - Active account: true\n  - Token scopes: 'repo', 'workflow'".into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = AuthService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let status = service.status(&trace()).expect("status should succeed");
        assert!(status.logged_in);
        assert_eq!(status.account.as_deref(), Some("octocat"));

        let (_program, args) = state.last_call().expect("command should be called");
        assert_eq!(args[0], "auth");
        assert_eq!(args[1], "status");
    }

    #[test]
    fn status_returns_logged_out_when_not_authenticated() {
        let output = RawExecutionOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "run gh auth login to authenticate".into(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = AuthService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let status = service
            .status(&trace())
            .expect("status should map to logged out");
        assert!(!status.logged_in);
        assert_eq!(state.call_count(), 1);
    }

    #[test]
    fn list_organizations_executes_gh_api_and_parses_payload() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"login":"octo-org","name":"Octo Org"}]"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = AuthService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let organizations = service
            .list_organizations(&trace())
            .expect("organizations should parse");
        assert_eq!(organizations.len(), 1);
        assert_eq!(organizations[0].login, "octo-org");

        let (_program, args) = state.last_call().expect("command should be called");
        assert_eq!(args[0], "api");
        assert_eq!(args[1], "user/orgs?per_page=100");
    }

    #[test]
    fn list_organizations_returns_empty_when_not_authenticated() {
        let output = RawExecutionOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "run gh auth login to authenticate".into(),
        };

        let (runner, _state) = RecordingRunner::new(vec![output]);
        let service = AuthService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let organizations = service
            .list_organizations(&trace())
            .expect("status should map to empty organizations");
        assert!(organizations.is_empty());
    }
}
