use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{PagesInfo, parse_pages_info};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurePagesInput {
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub path: String,
    pub build_type: Option<String>,
    pub cname: Option<String>,
}

impl ConfigurePagesInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.branch.trim().is_empty() {
            return Err(AppError::validation("branch is required"));
        }
        if self.path.trim().is_empty() {
            return Err(AppError::validation("path is required"));
        }

        if self
            .build_type
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "build_type must not be empty when provided",
            ));
        }

        if self
            .cname
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "cname must not be empty when provided",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePagesInput {
    pub owner: String,
    pub repo: String,
}

impl DeletePagesInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        Ok(())
    }
}

pub struct PagesService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> PagesService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn get(
        &self,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<PagesInfo, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let args = vec![format!("repos/{}/{}/pages", owner, repo)];
        let req = self.registry.build_request("pages.get", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_pages_info(&output.stdout)
    }

    pub fn create(
        &self,
        permission: RepoPermission,
        input: &ConfigurePagesInput,
        trace: &TraceContext,
    ) -> Result<PagesInfo, AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "pages.create")?;
        input.validate()?;

        let mut args = vec![
            format!("repos/{}/{}/pages", input.owner, input.repo),
            "-F".to_string(),
            format!("source[branch]={}", input.branch),
            "-F".to_string(),
            format!("source[path]={}", input.path),
        ];

        if let Some(build_type) = input.build_type.as_ref() {
            args.push("-F".to_string());
            args.push(format!("build_type={}", build_type));
        }

        if let Some(cname) = input.cname.as_ref() {
            args.push("-F".to_string());
            args.push(format!("cname={}", cname));
        }

        let req = self.registry.build_request("pages.create", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_pages_info(&output.stdout)
    }

    pub fn update(
        &self,
        permission: RepoPermission,
        input: &ConfigurePagesInput,
        trace: &TraceContext,
    ) -> Result<PagesInfo, AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "pages.update")?;
        input.validate()?;

        let mut args = vec![
            format!("repos/{}/{}/pages", input.owner, input.repo),
            "-F".to_string(),
            format!("source[branch]={}", input.branch),
            "-F".to_string(),
            format!("source[path]={}", input.path),
        ];

        if let Some(build_type) = input.build_type.as_ref() {
            args.push("-F".to_string());
            args.push(format!("build_type={}", build_type));
        }

        if let Some(cname) = input.cname.as_ref() {
            args.push("-F".to_string());
            args.push(format!("cname={}", cname));
        }

        let req = self.registry.build_request("pages.update", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_pages_info(&output.stdout)
    }

    pub fn delete(
        &self,
        permission: RepoPermission,
        input: &DeletePagesInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "pages.delete")?;
        input.validate()?;

        let args = vec![format!("repos/{}/{}/pages", input.owner, input.repo)];
        let req = self.registry.build_request("pages.delete", &args)?;
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
        TraceContext::new("req-pages-service")
    }

    #[test]
    fn get_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"url":"https://api.github.com/repos/octocat/hello/pages","status":"built","html_url":"https://octocat.github.io/hello/","source":{"branch":"main","path":"/"}}"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);

        let service = PagesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let page = service
            .get("octocat", "hello", &trace())
            .expect("get should succeed");
        assert_eq!(page.status.as_deref(), Some("built"));

        let (_program, args) = state.last_call().expect("call should be recorded");
        assert_eq!(args[0], "api");
    }

    #[test]
    fn create_requires_admin_permission() {
        let (runner, state) = RecordingRunner::new(vec![]);
        let service = PagesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = ConfigurePagesInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            branch: "main".into(),
            path: "/".into(),
            build_type: None,
            cname: None,
        };

        let err = service
            .create(RepoPermission::Write, &input, &trace())
            .expect_err("write should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn delete_is_noop_in_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(vec![]);
        let service = PagesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeletePagesInput {
            owner: "octocat".into(),
            repo: "hello".into(),
        };

        service
            .delete(RepoPermission::Admin, &input, &trace())
            .expect("delete should no-op");

        assert_eq!(state.call_count(), 0);
    }
}
