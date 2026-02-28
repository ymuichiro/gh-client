use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{IssueCreated, IssueSummary, parse_issue_created_output, parse_issue_summaries};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateIssueInput {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: Option<String>,
}

impl CreateIssueInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.title.trim().is_empty() {
            return Err(AppError::validation("title is required"));
        }

        if let Some(body) = self.body.as_ref() {
            if body.trim().is_empty() {
                return Err(AppError::validation("body must not be empty when provided"));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentIssueInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub body: String,
}

impl CommentIssueInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.number == 0 {
            return Err(AppError::validation("issue number must be greater than 0"));
        }
        if self.body.trim().is_empty() {
            return Err(AppError::validation("comment body is required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    Completed,
    NotPlanned,
}

impl CloseReason {
    fn as_flag_value(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::NotPlanned => "not planned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseIssueInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub comment: Option<String>,
    pub reason: Option<CloseReason>,
}

impl CloseIssueInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.number == 0 {
            return Err(AppError::validation("issue number must be greater than 0"));
        }

        if let Some(comment) = self.comment.as_ref() {
            if comment.trim().is_empty() {
                return Err(AppError::validation(
                    "comment must not be empty when provided",
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReopenIssueInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub comment: Option<String>,
}

impl ReopenIssueInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.number == 0 {
            return Err(AppError::validation("issue number must be greater than 0"));
        }

        if let Some(comment) = self.comment.as_ref() {
            if comment.trim().is_empty() {
                return Err(AppError::validation(
                    "comment must not be empty when provided",
                ));
            }
        }

        Ok(())
    }
}

pub struct IssuesService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> IssuesService<R> {
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
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<IssueSummary>, AppError> {
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
            "--state".to_string(),
            "all".to_string(),
        ];
        let req = self.registry.build_request("issue.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_issue_summaries(&output.stdout)
    }

    pub fn create(
        &self,
        permission: RepoPermission,
        input: &CreateIssueInput,
        trace: &TraceContext,
    ) -> Result<IssueCreated, AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "issue.create")?;
        input.validate()?;

        let mut args = vec![
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            "--title".to_string(),
            input.title.clone(),
        ];

        if let Some(body) = input.body.as_ref() {
            args.push("--body".to_string());
            args.push(body.clone());
        }

        let req = self.registry.build_request("issue.create", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_issue_created_output(&output.stdout)
    }

    pub fn comment(
        &self,
        permission: RepoPermission,
        input: &CommentIssueInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "issue.comment")?;
        input.validate()?;

        let args = vec![
            input.number.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            "--body".to_string(),
            input.body.clone(),
        ];

        let req = self.registry.build_request("issue.comment", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn close(
        &self,
        permission: RepoPermission,
        input: &CloseIssueInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "issue.close")?;
        input.validate()?;

        let mut args = vec![
            input.number.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        if let Some(comment) = input.comment.as_ref() {
            args.push("--comment".to_string());
            args.push(comment.clone());
        }

        if let Some(reason) = input.reason {
            args.push("--reason".to_string());
            args.push(reason.as_flag_value().to_string());
        }

        let req = self.registry.build_request("issue.close", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn reopen(
        &self,
        permission: RepoPermission,
        input: &ReopenIssueInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "issue.reopen")?;
        input.validate()?;

        let mut args = vec![
            input.number.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        if let Some(comment) = input.comment.as_ref() {
            args.push("--comment".to_string());
            args.push(comment.clone());
        }

        let req = self.registry.build_request("issue.reopen", &args)?;
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
        TraceContext::new("req-issues-service")
    }

    #[test]
    fn list_executes_issue_list_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[
              {
                "number": 11,
                "title": "Bug",
                "state": "OPEN",
                "url": "https://github.com/octocat/hello/issues/11",
                "author": {"login": "octocat"}
              }
            ]"#
            .to_string(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(output);
        let service = IssuesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let list = service
            .list("octocat", "hello", 20, &trace())
            .expect("list should succeed");
        assert_eq!(list.len(), 1);

        let (program, args) = state.last_call().expect("command should be called");
        assert_eq!(program, "gh");
        assert!(args.contains(&"issue".to_string()));
        assert!(args.contains(&"--repo".to_string()));
    }

    #[test]
    fn create_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = IssuesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreateIssueInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            title: "Bug".into(),
            body: Some("desc".into()),
        };

        let err = service
            .create(RepoPermission::Viewer, &input, &trace())
            .expect_err("viewer should be denied");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn create_executes_and_parses_url_output() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: "https://github.com/octocat/hello/issues/12\n".into(),
            stderr: String::new(),
        });
        let service = IssuesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreateIssueInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            title: "Bug".into(),
            body: Some("desc".into()),
        };

        let created = service
            .create(RepoPermission::Write, &input, &trace())
            .expect("create should succeed");

        assert_eq!(created.number, 12);
        assert_eq!(state.call_count(), 1);
    }

    #[test]
    fn close_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = IssuesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CloseIssueInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 12,
            comment: Some("done".into()),
            reason: Some(CloseReason::Completed),
        };

        service
            .close(RepoPermission::Write, &input, &trace())
            .expect("close should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--reason".to_string()));
        assert!(args.contains(&"completed".to_string()));
    }

    #[test]
    fn reopen_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = IssuesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = ReopenIssueInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 12,
            comment: Some("retry".into()),
        };

        service
            .reopen(RepoPermission::Write, &input, &trace())
            .expect("reopen should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--comment".to_string()));
    }
}
