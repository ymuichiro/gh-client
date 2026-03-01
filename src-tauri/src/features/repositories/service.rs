use serde_json::Value;

use crate::core::command_registry::CommandRegistry;
use crate::core::error::{AppError, ErrorCode};
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{
    BranchSummary, CommitSummary, RepoSummary, parse_branch_summaries, parse_commit_summaries,
    parse_repo_summaries,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryVisibility {
    Public,
    Private,
    Internal,
}

impl RepositoryVisibility {
    fn as_flag_value(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditRepositoryInput {
    pub owner: String,
    pub repo: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub default_branch: Option<String>,
    pub visibility: Option<RepositoryVisibility>,
    pub add_topics: Vec<String>,
    pub remove_topics: Vec<String>,
    pub replace_topics: Option<Vec<String>>,
}

impl EditRepositoryInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        if self
            .description
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "description must not be empty when provided",
            ));
        }

        if self
            .homepage
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "homepage must not be empty when provided",
            ));
        }

        if self
            .default_branch
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "default branch must not be empty when provided",
            ));
        }

        for topic in &self.add_topics {
            if topic.trim().is_empty() {
                return Err(AppError::validation(
                    "add_topics must not contain empty values",
                ));
            }
        }

        for topic in &self.remove_topics {
            if topic.trim().is_empty() {
                return Err(AppError::validation(
                    "remove_topics must not contain empty values",
                ));
            }
        }

        if let Some(topics) = self.replace_topics.as_ref() {
            for topic in topics {
                if topic.trim().is_empty() {
                    return Err(AppError::validation(
                        "replace_topics must not contain empty values",
                    ));
                }
            }
        }

        let has_changes = self.description.is_some()
            || self.homepage.is_some()
            || self.default_branch.is_some()
            || self.visibility.is_some()
            || !self.add_topics.is_empty()
            || !self.remove_topics.is_empty()
            || self.replace_topics.is_some();

        if !has_changes {
            return Err(AppError::validation(
                "at least one editable field must be provided",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateBranchInput {
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub from_branch: String,
}

impl CreateBranchInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.branch.trim().is_empty() || self.from_branch.trim().is_empty() {
            return Err(AppError::validation("branch and from_branch are required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteBranchInput {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

impl DeleteBranchInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.branch.trim().is_empty() {
            return Err(AppError::validation("branch is required"));
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

    pub fn edit(
        &self,
        permission: RepoPermission,
        input: &EditRepositoryInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "repo.edit")?;
        input.validate()?;

        let mut args = vec![format!("{}/{}", input.owner, input.repo)];

        if let Some(description) = input.description.as_ref() {
            args.push("--description".to_string());
            args.push(description.clone());
        }

        if let Some(homepage) = input.homepage.as_ref() {
            args.push("--homepage".to_string());
            args.push(homepage.clone());
        }

        if let Some(default_branch) = input.default_branch.as_ref() {
            args.push("--default-branch".to_string());
            args.push(default_branch.clone());
        }

        if let Some(visibility) = input.visibility {
            args.push("--visibility".to_string());
            args.push(visibility.as_flag_value().to_string());
            args.push("--accept-visibility-change-consequences".to_string());
        }

        for topic in &input.add_topics {
            args.push("--add-topic".to_string());
            args.push(topic.clone());
        }

        for topic in &input.remove_topics {
            args.push("--remove-topic".to_string());
            args.push(topic.clone());
        }

        if args.len() > 1 {
            let req = self.registry.build_request("repo.edit", &args)?;
            let _ = self.executor.execute(&req, trace)?;
        }

        if let Some(topics) = input.replace_topics.as_ref() {
            let mut topic_args = vec![format!("repos/{}/{}/topics", input.owner, input.repo)];
            for topic in topics {
                topic_args.push("-f".to_string());
                topic_args.push(format!("names[]={}", topic));
            }

            let req = self
                .registry
                .build_request("repo.topics.replace", &topic_args)?;
            let _ = self.executor.execute(&req, trace)?;
        }

        Ok(())
    }

    pub fn list_branches(
        &self,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<BranchSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![format!(
            "repos/{}/{}/branches?per_page={}",
            owner, repo, limit
        )];
        let req = self.registry.build_request("repo.branches.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_branch_summaries(&output.stdout)
    }

    pub fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<CommitSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let mut endpoint = format!("repos/{}/{}/commits?per_page={}", owner, repo, limit);
        if let Some(branch) = branch {
            if branch.trim().is_empty() {
                return Err(AppError::validation(
                    "branch must not be empty when provided",
                ));
            }
            endpoint.push_str(&format!("&sha={}", branch));
        }

        let args = vec![endpoint];
        let req = self.registry.build_request("repo.commits.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_commit_summaries(&output.stdout)
    }

    pub fn create_branch(
        &self,
        permission: RepoPermission,
        input: &CreateBranchInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "repo.branch.create")?;
        input.validate()?;

        let get_ref_args = vec![format!(
            "repos/{}/{}/git/ref/heads/{}",
            input.owner, input.repo, input.from_branch
        )];
        let get_ref_req = self
            .registry
            .build_request("repo.branch.ref.get", &get_ref_args)?;
        let (get_ref_output, _) = self.executor.execute(&get_ref_req, trace)?;

        let value: Value = serde_json::from_str(&get_ref_output.stdout).map_err(|err| {
            AppError::new(
                ErrorCode::UpstreamError,
                format!("failed to parse reference payload: {}", err),
                false,
            )
        })?;
        let sha = value
            .get("object")
            .and_then(|object| object.get("sha"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::UpstreamError,
                    "reference payload did not contain object.sha",
                    false,
                )
            })?;

        let create_args = vec![
            format!("repos/{}/{}/git/refs", input.owner, input.repo),
            "-f".to_string(),
            format!("ref=refs/heads/{}", input.branch),
            "-f".to_string(),
            format!("sha={}", sha),
        ];
        let create_req = self
            .registry
            .build_request("repo.branch.create", &create_args)?;
        let _ = self.executor.execute(&create_req, trace)?;
        Ok(())
    }

    pub fn delete_branch(
        &self,
        permission: RepoPermission,
        input: &DeleteBranchInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "repo.branch.delete")?;
        input.validate()?;

        let args = vec![format!(
            "repos/{}/{}/git/refs/heads/{}",
            input.owner, input.repo, input.branch
        )];
        let req = self.registry.build_request("repo.branch.delete", &args)?;
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
        let (runner, state) = RecordingRunner::new(vec![output]);
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
    fn list_branches_executes_branch_list_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"name":"main","protected":true,"commit":{"sha":"abc123"}}]"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = RepositoriesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let branches = service
            .list_branches("octocat", "hello", 20, &trace())
            .expect("branch list should succeed");
        assert_eq!(branches.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert_eq!(args[0], "api");
        assert!(args[1].contains("/branches"));
    }

    #[test]
    fn list_commits_executes_commit_list_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"sha":"abc123","commit":{"message":"init","author":{"name":"octocat","email":"a@b.com","date":"2026-01-01T00:00:00Z"}}}]"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = RepositoriesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let commits = service
            .list_commits("octocat", "hello", Some("main"), 10, &trace())
            .expect("commit list should succeed");
        assert_eq!(commits.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args[1].contains("/commits"));
        assert!(args[1].contains("sha=main"));
    }

    #[test]
    fn create_branch_executes_get_ref_and_create() {
        let get_ref = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"object":{"sha":"deadbeef"}}"#.into(),
            stderr: String::new(),
        };
        let create = RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![get_ref, create]);
        let service = RepositoriesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreateBranchInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            branch: "feature-a".into(),
            from_branch: "main".into(),
        };

        service
            .create_branch(RepoPermission::Write, &input, &trace())
            .expect("create branch should succeed");

        assert_eq!(state.call_count(), 2);
        let last = state.last_call().expect("command should be called");
        assert!(last.1.contains(&"sha=deadbeef".to_string()));
    }

    #[test]
    fn edit_requires_admin_permission() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = RepositoriesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = EditRepositoryInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            description: Some("desc".into()),
            homepage: None,
            default_branch: None,
            visibility: None,
            add_topics: vec![],
            remove_topics: vec![],
            replace_topics: None,
        };

        let err = service
            .edit(RepoPermission::Write, &input, &trace())
            .expect_err("write should be denied");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn create_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
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
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
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
    fn delete_branch_is_noop_under_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = RepositoriesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeleteBranchInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            branch: "feature-a".into(),
        };

        service
            .delete_branch(RepoPermission::Write, &input, &trace())
            .expect("delete branch should be skipped safely");

        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn delete_is_noop_under_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let executor = CommandExecutor::new(runner, true);
        let service = RepositoriesService::new(CommandRegistry::with_defaults(), executor);

        service
            .delete(RepoPermission::Admin, "octocat", "repo-z", &trace())
            .expect("delete should be skipped safely");

        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn delete_requires_admin_permission() {
        let (runner, _state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let executor = CommandExecutor::new(runner, true);
        let service = RepositoriesService::new(CommandRegistry::with_defaults(), executor);

        let err = service
            .delete(RepoPermission::Write, "octocat", "repo-z", &trace())
            .expect_err("write permission should not allow delete");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
    }
}
