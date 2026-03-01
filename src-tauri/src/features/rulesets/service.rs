use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{RulesetSummary, parse_ruleset, parse_ruleset_summaries};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetField {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertRulesetInput {
    pub owner: String,
    pub repo: String,
    pub fields: Vec<RulesetField>,
}

impl UpsertRulesetInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.fields.is_empty() {
            return Err(AppError::validation("fields must not be empty"));
        }

        for field in &self.fields {
            if field.key.trim().is_empty() || field.value.trim().is_empty() {
                return Err(AppError::validation(
                    "ruleset fields must have non-empty key and value",
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteRulesetInput {
    pub owner: String,
    pub repo: String,
    pub ruleset_id: u64,
}

impl DeleteRulesetInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.ruleset_id == 0 {
            return Err(AppError::validation("ruleset_id must be greater than 0"));
        }

        Ok(())
    }
}

pub struct RulesetsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> RulesetsService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<Vec<RulesetSummary>, AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "rulesets.list")?;

        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let args = vec![format!("repos/{}/{}/rulesets", owner, repo)];
        let req = self.registry.build_request("rulesets.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_ruleset_summaries(&output.stdout)
    }

    pub fn get(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        ruleset_id: u64,
        trace: &TraceContext,
    ) -> Result<RulesetSummary, AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "rulesets.get")?;

        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if ruleset_id == 0 {
            return Err(AppError::validation("ruleset_id must be greater than 0"));
        }

        let args = vec![format!("repos/{}/{}/rulesets/{}", owner, repo, ruleset_id)];
        let req = self.registry.build_request("rulesets.get", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_ruleset(&output.stdout)
    }

    pub fn create(
        &self,
        permission: RepoPermission,
        input: &UpsertRulesetInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "rulesets.create")?;
        input.validate()?;

        let mut args = vec![format!("repos/{}/{}/rulesets", input.owner, input.repo)];
        for field in &input.fields {
            args.push("-f".to_string());
            args.push(format!("{}={}", field.key, field.value));
        }

        let req = self.registry.build_request("rulesets.create", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn update(
        &self,
        permission: RepoPermission,
        input: &UpsertRulesetInput,
        ruleset_id: u64,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "rulesets.update")?;
        input.validate()?;
        if ruleset_id == 0 {
            return Err(AppError::validation("ruleset_id must be greater than 0"));
        }

        let mut args = vec![format!(
            "repos/{}/{}/rulesets/{}",
            input.owner, input.repo, ruleset_id
        )];
        for field in &input.fields {
            args.push("-f".to_string());
            args.push(format!("{}={}", field.key, field.value));
        }

        let req = self.registry.build_request("rulesets.update", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn delete(
        &self,
        permission: RepoPermission,
        input: &DeleteRulesetInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "rulesets.delete")?;
        input.validate()?;

        let args = vec![format!(
            "repos/{}/{}/rulesets/{}",
            input.owner, input.repo, input.ruleset_id
        )];

        let req = self.registry.build_request("rulesets.delete", &args)?;
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
        TraceContext::new("req-rulesets-service")
    }

    #[test]
    fn list_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"id":1,"name":"Protect main","target":"branch","enforcement":"active"}]"#
                .into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = RulesetsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list(RepoPermission::Admin, "octocat", "hello", &trace())
            .expect("list should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("call should be recorded");
        assert_eq!(args[0], "api");
        assert_eq!(args[1], "repos/octocat/hello/rulesets");
    }

    #[test]
    fn create_requires_admin_permission() {
        let (runner, state) = RecordingRunner::new(vec![]);
        let service = RulesetsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = UpsertRulesetInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            fields: vec![RulesetField {
                key: "name".into(),
                value: "Protect main".into(),
            }],
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
        let service = RulesetsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeleteRulesetInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            ruleset_id: 1,
        };

        service
            .delete(RepoPermission::Admin, &input, &trace())
            .expect("delete should no-op");

        assert_eq!(state.call_count(), 0);
    }
}
