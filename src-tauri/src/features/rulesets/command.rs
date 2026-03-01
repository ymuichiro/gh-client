use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::RulesetSummary;
use super::service::{DeleteRulesetInput, RulesetsService, UpsertRulesetInput};

pub struct RulesetsCommandHandler<R: Runner> {
    service: RulesetsService<R>,
}

impl<R: Runner> RulesetsCommandHandler<R> {
    pub fn new(service: RulesetsService<R>) -> Self {
        Self { service }
    }

    pub fn list_rulesets(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<RulesetSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list(permission, owner, repo, &trace)
    }

    pub fn get_ruleset(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        ruleset_id: u64,
    ) -> Result<RulesetSummary, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .get(permission, owner, repo, ruleset_id, &trace)
    }

    pub fn create_ruleset(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &UpsertRulesetInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create(permission, input, &trace)
    }

    pub fn update_ruleset(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &UpsertRulesetInput,
        ruleset_id: u64,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.update(permission, input, ruleset_id, &trace)
    }

    pub fn delete_ruleset(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeleteRulesetInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete(permission, input, &trace)
    }
}
