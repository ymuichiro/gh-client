use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{RunSummary, WorkflowSummary, parse_run_summaries, parse_workflow_summaries};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunActionInput {
    pub owner: String,
    pub repo: String,
    pub run_id: u64,
}

impl RunActionInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.run_id == 0 {
            return Err(AppError::validation("run id must be greater than 0"));
        }

        Ok(())
    }
}

pub struct ActionsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> ActionsService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list_workflows(
        &self,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<WorkflowSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![
            "--repo".to_string(),
            format!("{}/{}", owner, repo),
            "--limit".to_string(),
            limit.to_string(),
            "--all".to_string(),
        ];
        let req = self.registry.build_request("workflow.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_workflow_summaries(&output.stdout)
    }

    pub fn list_runs(
        &self,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<RunSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![
            "--repo".to_string(),
            format!("{}/{}", owner, repo),
            "--limit".to_string(),
            limit.to_string(),
            "--all".to_string(),
        ];
        let req = self.registry.build_request("run.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_run_summaries(&output.stdout)
    }

    pub fn rerun(
        &self,
        permission: RepoPermission,
        input: &RunActionInput,
        failed_only: bool,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "run.rerun")?;
        input.validate()?;

        let mut args = vec![
            input.run_id.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        if failed_only {
            args.push("--failed".to_string());
        }

        let req = self.registry.build_request("run.rerun", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn cancel(
        &self,
        permission: RepoPermission,
        input: &RunActionInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "run.cancel")?;
        input.validate()?;

        let args = vec![
            input.run_id.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];
        let req = self.registry.build_request("run.cancel", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        response: RawExecutionOutput,
    }

    impl RecordingRunner {
        fn new(response: RawExecutionOutput) -> (Self, Arc<RecordingState>) {
            let state = Arc::new(RecordingState::default());
            (
                Self {
                    state: Arc::clone(&state),
                    response,
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
            Ok(self.response.clone())
        }
    }

    fn trace() -> TraceContext {
        TraceContext::new("req-actions-service")
    }

    #[test]
    fn list_workflows_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"id":1,"name":"CI","path":".github/workflows/ci.yml","state":"active"}]"#
                .into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(output);
        let service = ActionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_workflows("octocat", "hello", 50, &trace())
            .expect("list workflows should succeed");
        assert_eq!(items.len(), 1);

        let (program, args) = state.last_call().expect("command should be called");
        assert_eq!(program, "gh");
        assert!(args.contains(&"workflow".to_string()));
    }

    #[test]
    fn list_runs_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"databaseId":99,"workflowName":"CI","headBranch":"main","status":"completed","conclusion":"success","url":"https://example/run/99","displayTitle":"build"}]"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(output);
        let service = ActionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_runs("octocat", "hello", 20, &trace())
            .expect("list runs should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"run".to_string()));
        assert!(args.contains(&"list".to_string()));
    }

    #[test]
    fn rerun_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ActionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = RunActionInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            run_id: 10,
        };

        let err = service
            .rerun(RepoPermission::Viewer, &input, false, &trace())
            .expect_err("viewer should be denied");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn cancel_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ActionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = RunActionInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            run_id: 11,
        };

        service
            .cancel(RepoPermission::Write, &input, &trace())
            .expect("cancel should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"cancel".to_string()));
    }
}
