use serde::Deserialize;

use crate::core::command_registry::{CommandRequest, CommandSafety};
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

#[derive(Debug, Default, Deserialize)]
struct RepoPermissionsPayload {
    admin: Option<bool>,
    maintain: Option<bool>,
    push: Option<bool>,
    triage: Option<bool>,
    pull: Option<bool>,
}

pub struct RepoPermissionResolver<R: Runner> {
    executor: CommandExecutor<R>,
}

impl<R: Runner> RepoPermissionResolver<R> {
    pub fn new(executor: CommandExecutor<R>) -> Self {
        Self { executor }
    }

    pub fn resolve(
        &self,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<RepoPermission, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let request = CommandRequest {
            command_id: "internal.repo_permission.get".to_string(),
            program: "gh".to_string(),
            args: vec![
                "api".to_string(),
                format!("repos/{}/{}", owner, repo),
                "--jq".to_string(),
                ".permissions".to_string(),
            ],
            safety: CommandSafety::NonDestructive,
        };

        let (output, _audit) = self.executor.execute(&request, trace)?;
        let parsed: RepoPermissionsPayload = serde_json::from_str(output.stdout.trim()).unwrap_or_default();

        if parsed.admin.unwrap_or(false) || parsed.maintain.unwrap_or(false) {
            return Ok(RepoPermission::Admin);
        }

        if parsed.push.unwrap_or(false) || parsed.triage.unwrap_or(false) {
            return Ok(RepoPermission::Write);
        }

        if parsed.pull.unwrap_or(false) {
            return Ok(RepoPermission::Viewer);
        }

        Ok(RepoPermission::Viewer)
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
        responses: Mutex<VecDeque<RawExecutionOutput>>,
    }

    #[derive(Clone)]
    struct RecordingRunner {
        state: Arc<RecordingState>,
    }

    impl RecordingRunner {
        fn new(responses: Vec<RawExecutionOutput>) -> (Self, Arc<RecordingState>) {
            let state = Arc::new(RecordingState {
                calls: Mutex::new(Vec::new()),
                responses: Mutex::new(VecDeque::from(responses)),
            });
            (
                Self {
                    state: Arc::clone(&state),
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

            let output = self
                .state
                .responses
                .lock()
                .expect("lock poisoned")
                .pop_front()
                .unwrap_or(RawExecutionOutput {
                    exit_code: 0,
                    stdout: "{}".into(),
                    stderr: String::new(),
                });

            Ok(output)
        }
    }

    fn trace() -> TraceContext {
        TraceContext::new("req-permission")
    }

    #[test]
    fn resolves_admin_permission_from_repo_api() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"admin":true,"maintain":false,"push":true,"triage":true,"pull":true}"#.into(),
            stderr: String::new(),
        }]);
        let resolver = RepoPermissionResolver::new(CommandExecutor::new(runner, false));

        let permission = resolver
            .resolve("octocat", "hello", &trace())
            .expect("permission lookup should succeed");

        assert_eq!(permission, RepoPermission::Admin);
        let (_program, args) = state
            .calls
            .lock()
            .expect("lock poisoned")
            .last()
            .cloned()
            .expect("command should be recorded");
        assert!(args.iter().any(|arg| arg == "repos/octocat/hello"));
        assert!(args.iter().any(|arg| arg == "--jq"));
    }

    #[test]
    fn defaults_to_viewer_when_permissions_are_missing() {
        let (runner, _state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: "null".into(),
            stderr: String::new(),
        }]);
        let resolver = RepoPermissionResolver::new(CommandExecutor::new(runner, false));

        let permission = resolver
            .resolve("octocat", "hello", &trace())
            .expect("permission lookup should succeed");

        assert_eq!(permission, RepoPermission::Viewer);
    }
}
