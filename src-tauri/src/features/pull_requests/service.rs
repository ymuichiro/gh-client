use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{
    PullRequestCreated, PullRequestSummary, parse_pull_request_created,
    parse_pull_request_summaries,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePullRequestInput {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub head: String,
    pub base: String,
    pub body: Option<String>,
    pub draft: bool,
}

impl CreatePullRequestInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        if self.title.trim().is_empty() {
            return Err(AppError::validation("title is required"));
        }

        if self.head.trim().is_empty() || self.base.trim().is_empty() {
            return Err(AppError::validation("head and base are required"));
        }

        if let Some(body) = self.body.as_ref() {
            if body.trim().is_empty() {
                return Err(AppError::validation("body must not be empty when provided"));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewEvent {
    Approve,
    RequestChanges,
    Comment,
}

impl ReviewEvent {
    fn as_flag(self) -> &'static str {
        match self {
            Self::Approve => "--approve",
            Self::RequestChanges => "--request-changes",
            Self::Comment => "--comment",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewPullRequestInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub event: ReviewEvent,
    pub body: Option<String>,
}

impl ReviewPullRequestInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        if self.number == 0 {
            return Err(AppError::validation(
                "pull request number must be greater than 0",
            ));
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
pub struct EditPullRequestInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: Option<String>,
    pub body: Option<String>,
    pub base: Option<String>,
}

impl EditPullRequestInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.number == 0 {
            return Err(AppError::validation(
                "pull request number must be greater than 0",
            ));
        }

        if self.title.is_none() && self.body.is_none() && self.base.is_none() {
            return Err(AppError::validation(
                "at least one of title/body/base must be provided",
            ));
        }

        if self
            .title
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "title must not be empty when provided",
            ));
        }
        if self
            .body
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation("body must not be empty when provided"));
        }
        if self
            .base
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation("base must not be empty when provided"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl MergeMethod {
    fn as_flag(self) -> &'static str {
        match self {
            Self::Merge => "--merge",
            Self::Squash => "--squash",
            Self::Rebase => "--rebase",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergePullRequestInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub method: MergeMethod,
    pub delete_branch: bool,
    pub auto: bool,
}

impl MergePullRequestInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        if self.number == 0 {
            return Err(AppError::validation(
                "pull request number must be greater than 0",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosePullRequestInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub comment: Option<String>,
    pub delete_branch: bool,
}

impl ClosePullRequestInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.number == 0 {
            return Err(AppError::validation(
                "pull request number must be greater than 0",
            ));
        }
        if self
            .comment
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "comment must not be empty when provided",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReopenPullRequestInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub comment: Option<String>,
}

impl ReopenPullRequestInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.number == 0 {
            return Err(AppError::validation(
                "pull request number must be greater than 0",
            ));
        }
        if self
            .comment
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "comment must not be empty when provided",
            ));
        }

        Ok(())
    }
}

pub struct PullRequestsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> PullRequestsService<R> {
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
    ) -> Result<Vec<PullRequestSummary>, AppError> {
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
        ];
        let req = self.registry.build_request("pr.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_pull_request_summaries(&output.stdout)
    }

    pub fn create(
        &self,
        permission: RepoPermission,
        input: &CreatePullRequestInput,
        trace: &TraceContext,
    ) -> Result<PullRequestCreated, AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "pr.create")?;
        input.validate()?;

        let mut args = vec![
            format!("repos/{}/{}/pulls", input.owner, input.repo),
            "-F".to_string(),
            format!("title={}", input.title),
            "-F".to_string(),
            format!("head={}", input.head),
            "-F".to_string(),
            format!("base={}", input.base),
        ];

        if let Some(body) = input.body.as_ref() {
            args.push("-F".to_string());
            args.push(format!("body={}", body));
        }

        if input.draft {
            args.push("-F".to_string());
            args.push("draft=true".to_string());
        }

        let req = self.registry.build_request("pr.create", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_pull_request_created(&output.stdout)
    }

    pub fn review(
        &self,
        permission: RepoPermission,
        input: &ReviewPullRequestInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "pr.review")?;
        input.validate()?;

        let mut args = vec![
            input.number.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            input.event.as_flag().to_string(),
        ];

        if let Some(body) = input.body.as_ref() {
            args.push("--body".to_string());
            args.push(body.clone());
        }

        let req = self.registry.build_request("pr.review", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn edit(
        &self,
        permission: RepoPermission,
        input: &EditPullRequestInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "pr.edit")?;
        input.validate()?;

        let mut args = vec![
            input.number.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        if let Some(title) = input.title.as_ref() {
            args.push("--title".to_string());
            args.push(title.clone());
        }

        if let Some(body) = input.body.as_ref() {
            args.push("--body".to_string());
            args.push(body.clone());
        }

        if let Some(base) = input.base.as_ref() {
            args.push("--base".to_string());
            args.push(base.clone());
        }

        let req = self.registry.build_request("pr.edit", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn merge(
        &self,
        permission: RepoPermission,
        input: &MergePullRequestInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "pr.merge")?;
        input.validate()?;

        let mut args = vec![
            input.number.to_string(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            input.method.as_flag().to_string(),
        ];

        if input.delete_branch {
            args.push("--delete-branch".to_string());
        }

        if input.auto {
            args.push("--auto".to_string());
        }

        let req = self.registry.build_request("pr.merge", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn close(
        &self,
        permission: RepoPermission,
        input: &ClosePullRequestInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "pr.close")?;
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

        if input.delete_branch {
            args.push("--delete-branch".to_string());
        }

        let req = self.registry.build_request("pr.close", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn reopen(
        &self,
        permission: RepoPermission,
        input: &ReopenPullRequestInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "pr.reopen")?;
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

        let req = self.registry.build_request("pr.reopen", &args)?;
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
        TraceContext::new("req-pr-service")
    }

    #[test]
    fn list_executes_pr_list_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[
              {
                "number": 1,
                "title": "hello",
                "state": "OPEN",
                "url": "https://github.com/octocat/hello/pull/1",
                "isDraft": false,
                "author": {"login": "octocat"},
                "headRefName": "feature-a",
                "baseRefName": "main"
              }
            ]"#
            .to_string(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(output);
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let prs = service
            .list("octocat", "hello", 20, &trace())
            .expect("list should succeed");

        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 1);

        let (program, args) = state.last_call().expect("command should be called");
        assert_eq!(program, "gh");
        assert!(args.contains(&"pr".to_string()));
        assert!(args.contains(&"--repo".to_string()));
        assert!(args.contains(&"octocat/hello".to_string()));
    }

    #[test]
    fn create_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreatePullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            title: "Add feature".into(),
            head: "feature-a".into(),
            base: "main".into(),
            body: Some("body".into()),
            draft: false,
        };

        let err = service
            .create(RepoPermission::Viewer, &input, &trace())
            .expect_err("viewer should be denied");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn create_executes_and_parses_response() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"number":2,"html_url":"https://example/pull/2","state":"OPEN"}"#.into(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreatePullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            title: "Add feature".into(),
            head: "feature-a".into(),
            base: "main".into(),
            body: Some("body".into()),
            draft: true,
        };

        let created = service
            .create(RepoPermission::Write, &input, &trace())
            .expect("create should succeed");

        assert_eq!(created.number, 2);
        assert_eq!(state.call_count(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"repos/octocat/hello/pulls".to_string()));
        assert!(args.contains(&"draft=true".to_string()));
    }

    #[test]
    fn review_executes_command_with_event_flag() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = ReviewPullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 3,
            event: ReviewEvent::Approve,
            body: Some("LGTM".into()),
        };

        service
            .review(RepoPermission::Write, &input, &trace())
            .expect("review should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--approve".to_string()));
        assert!(args.contains(&"--body".to_string()));
    }

    #[test]
    fn merge_executes_command_with_merge_method() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = MergePullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 4,
            method: MergeMethod::Squash,
            delete_branch: true,
            auto: false,
        };

        service
            .merge(RepoPermission::Write, &input, &trace())
            .expect("merge should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--squash".to_string()));
        assert!(args.contains(&"--delete-branch".to_string()));
    }

    #[test]
    fn merge_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = MergePullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 5,
            method: MergeMethod::Merge,
            delete_branch: false,
            auto: true,
        };

        let err = service
            .merge(RepoPermission::Viewer, &input, &trace())
            .expect_err("viewer should not merge");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn edit_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = EditPullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 7,
            title: Some("new title".into()),
            body: None,
            base: Some("develop".into()),
        };

        service
            .edit(RepoPermission::Write, &input, &trace())
            .expect("edit should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--title".to_string()));
        assert!(args.contains(&"--base".to_string()));
    }

    #[test]
    fn close_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = ClosePullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 8,
            comment: Some("closing".into()),
            delete_branch: true,
        };

        service
            .close(RepoPermission::Write, &input, &trace())
            .expect("close should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--delete-branch".to_string()));
        assert!(args.contains(&"--comment".to_string()));
    }

    #[test]
    fn reopen_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = PullRequestsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = ReopenPullRequestInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            number: 8,
            comment: Some("reopen".into()),
        };

        service
            .reopen(RepoPermission::Write, &input, &trace())
            .expect("reopen should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--comment".to_string()));
    }
}
