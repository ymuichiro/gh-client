use crate::core::command_registry::CommandRegistry;
use crate::core::error::{AppError, ErrorCode};
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{RepoSummary, parse_repo_summaries};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRepositoryInput {
    pub owner: String,
    pub name: String,
    pub private: bool,
    pub description: Option<String>,
}

impl CreateRepositoryInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() {
            return Err(AppError::validation("owner is required"));
        }

        if self.name.trim().is_empty() {
            return Err(AppError::validation("name is required"));
        }

        if self.name.contains(' ') {
            return Err(AppError::validation(
                "repository name must not contain spaces",
            ));
        }

        Ok(())
    }
}

pub struct RepositoriesService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> RepositoriesService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list(
        &self,
        owner: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<RepoSummary>, AppError> {
        if owner.trim().is_empty() {
            return Err(AppError::validation("owner is required"));
        }

        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![owner.to_string(), "--limit".to_string(), limit.to_string()];
        let req = self.registry.build_request("repo.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_repo_summaries(&output.stdout)
    }

    pub fn create(
        &self,
        permission: RepoPermission,
        input: &CreateRepositoryInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "repo.create")?;
        input.validate()?;

        let target = format!("{}/{}", input.owner, input.name);
        let mut args = vec![target, "--confirm".to_string()];
        if input.private {
            args.push("--private".to_string());
        } else {
            args.push("--public".to_string());
        }

        if let Some(description) = input.description.as_ref() {
            if description.trim().is_empty() {
                return Err(AppError::new(
                    ErrorCode::ValidationError,
                    "description must not be empty when provided",
                    false,
                ));
            }

            args.push("--description".to_string());
            args.push(description.clone());
        }

        let req = self.registry.build_request("repo.create", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn delete(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "repo.delete")?;

        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let args = vec![format!("{}/{}", owner, repo), "--yes".to_string()];
        let req = self.registry.build_request("repo.delete", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        TraceContext::new("req-service")
    }

    #[test]
    fn list_executes_repo_list_and_parses_payload() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[
              {
                "name":"repo-a",
                "nameWithOwner":"octocat/repo-a",
                "description":"test",
                "url":"https://github.com/octocat/repo-a",
                "isPrivate":false,
                "viewerPermission":"ADMIN"
              }
            ]"#
            .to_string(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(output);
        let executor = CommandExecutor::new(runner, false);
        let service = RepositoriesService::new(CommandRegistry::with_defaults(), executor);

        let repos = service
            .list("octocat", 20, &trace())
            .expect("list should succeed");
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name_with_owner, "octocat/repo-a");

        let (program, args) = state.last_call().expect("command should be called");
        assert_eq!(program, "gh");
        assert!(args.contains(&"octocat".to_string()));
        assert!(args.contains(&"--limit".to_string()));
    }

    #[test]
    fn create_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let executor = CommandExecutor::new(runner, false);
        let service = RepositoriesService::new(CommandRegistry::with_defaults(), executor);

        let input = CreateRepositoryInput {
            owner: "octocat".into(),
            name: "repo-a".into(),
            private: true,
            description: None,
        };

        let err = service
            .create(RepoPermission::Viewer, &input, &trace())
            .expect_err("viewer should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn create_executes_when_permission_is_sufficient() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let executor = CommandExecutor::new(runner, false);
        let service = RepositoriesService::new(CommandRegistry::with_defaults(), executor);

        let input = CreateRepositoryInput {
            owner: "octocat".into(),
            name: "repo-b".into(),
            private: false,
            description: Some("desc".into()),
        };

        service
            .create(RepoPermission::Write, &input, &trace())
            .expect("write permission should allow create");

        assert_eq!(state.call_count(), 1);
        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--public".to_string()));
        assert!(args.contains(&"--description".to_string()));
    }

    #[test]
    fn delete_is_noop_under_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let executor = CommandExecutor::new(runner, true);
        let service = RepositoriesService::new(CommandRegistry::with_defaults(), executor);

        service
            .delete(RepoPermission::Admin, "octocat", "repo-z", &trace())
            .expect("delete should be skipped safely");

        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn delete_requires_admin_permission() {
        let (runner, _state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let executor = CommandExecutor::new(runner, true);
        let service = RepositoriesService::new(CommandRegistry::with_defaults(), executor);

        let err = service
            .delete(RepoPermission::Write, "octocat", "repo-z", &trace())
            .expect_err("write permission should not allow delete");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
    }
}
