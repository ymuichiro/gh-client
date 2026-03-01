use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{WikiInfo, parse_wiki_info};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateWikiInput {
    pub owner: String,
    pub repo: String,
    pub enabled: bool,
}

impl UpdateWikiInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        Ok(())
    }
}

pub struct WikiService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> WikiService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn get(&self, owner: &str, repo: &str, trace: &TraceContext) -> Result<WikiInfo, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let args = vec![format!("repos/{}/{}", owner, repo)];
        let req = self.registry.build_request("wiki.get", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_wiki_info(&output.stdout)
    }

    pub fn update(
        &self,
        permission: RepoPermission,
        input: &UpdateWikiInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "wiki.update")?;
        input.validate()?;

        let args = vec![
            format!("repos/{}/{}", input.owner, input.repo),
            "-F".to_string(),
            format!("has_wiki={}", input.enabled),
        ];

        let req = self.registry.build_request("wiki.update", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::core::error::ErrorCode;
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
        TraceContext::new("req-wiki-service")
    }

    #[test]
    fn get_executes_wiki_query() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"has_wiki":true,"html_url":"https://github.com/octocat/hello"}"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = WikiService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let info = service
            .get("octocat", "hello", &trace())
            .expect("get should succeed");
        assert!(info.has_wiki);

        let (_program, args) = state.last_call().expect("call should be recorded");
        assert_eq!(args[0], "api");
        assert_eq!(args[1], "repos/octocat/hello");
    }

    #[test]
    fn update_requires_admin_permission() {
        let (runner, state) = RecordingRunner::new(vec![]);
        let service = WikiService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = UpdateWikiInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            enabled: false,
        };

        let err = service
            .update(RepoPermission::Write, &input, &trace())
            .expect_err("write should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }
}
