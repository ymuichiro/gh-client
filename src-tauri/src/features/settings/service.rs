use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{Collaborator, parse_collaborators};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollaboratorPermission {
    Pull,
    Push,
    Admin,
    Maintain,
    Triage,
}

impl CollaboratorPermission {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::Pull => "pull",
            Self::Push => "push",
            Self::Admin => "admin",
            Self::Maintain => "maintain",
            Self::Triage => "triage",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddCollaboratorInput {
    pub owner: String,
    pub repo: String,
    pub username: String,
    pub permission: CollaboratorPermission,
}

impl AddCollaboratorInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.username.trim().is_empty() {
            return Err(AppError::validation("username is required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveCollaboratorInput {
    pub owner: String,
    pub repo: String,
    pub username: String,
}

impl RemoveCollaboratorInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.username.trim().is_empty() {
            return Err(AppError::validation("username is required"));
        }

        Ok(())
    }
}

pub struct SettingsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> SettingsService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list_collaborators(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<Vec<Collaborator>, AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.collaborators.list",
        )?;

        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let args = vec![format!("repos/{}/{}/collaborators", owner, repo)];
        let req = self
            .registry
            .build_request("settings.collaborators.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_collaborators(&output.stdout)
    }

    pub fn add_collaborator(
        &self,
        permission: RepoPermission,
        input: &AddCollaboratorInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.collaborators.add",
        )?;
        input.validate()?;

        let args = vec![
            format!(
                "repos/{}/{}/collaborators/{}",
                input.owner, input.repo, input.username
            ),
            "-f".to_string(),
            format!("permission={}", input.permission.as_api_value()),
        ];

        let req = self
            .registry
            .build_request("settings.collaborators.add", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn remove_collaborator(
        &self,
        permission: RepoPermission,
        input: &RemoveCollaboratorInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.collaborators.remove",
        )?;
        input.validate()?;

        let args = vec![format!(
            "repos/{}/{}/collaborators/{}",
            input.owner, input.repo, input.username
        )];

        let req = self
            .registry
            .build_request("settings.collaborators.remove", &args)?;
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
        TraceContext::new("req-settings-service")
    }

    #[test]
    fn list_collaborators_requires_admin_permission() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let err = service
            .list_collaborators(RepoPermission::Write, "octocat", "hello", &trace())
            .expect_err("write permission should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn list_collaborators_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"login":"octocat","permissions":{"admin":true,"push":true,"pull":true}}]"#
                .into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(output);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_collaborators(RepoPermission::Admin, "octocat", "hello", &trace())
            .expect("list should succeed");
        assert_eq!(items.len(), 1);

        let (program, args) = state.last_call().expect("command should be called");
        assert_eq!(program, "gh");
        assert_eq!(args[0], "api");
    }

    #[test]
    fn add_collaborator_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = AddCollaboratorInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            username: "hubot".into(),
            permission: CollaboratorPermission::Push,
        };

        service
            .add_collaborator(RepoPermission::Admin, &input, &trace())
            .expect("add collaborator should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"permission=push".to_string()));
    }

    #[test]
    fn remove_collaborator_is_noop_in_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = RemoveCollaboratorInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            username: "hubot".into(),
        };

        service
            .remove_collaborator(RepoPermission::Admin, &input, &trace())
            .expect("remove collaborator should no-op");

        assert_eq!(state.call_count(), 0);
    }
}
