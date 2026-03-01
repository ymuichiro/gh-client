use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{RunDetail, RunSummary, WorkflowSummary};
use super::service::{ActionsService, RunActionInput};

pub struct ActionsCommandHandler<R: Runner> {
    service: ActionsService<R>,
}

impl<R: Runner> ActionsCommandHandler<R> {
    pub fn new(service: ActionsService<R>) -> Self {
        Self { service }
    }

    pub fn list_workflows(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<WorkflowSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_workflows(owner, repo, limit, &trace)
    }

    pub fn list_runs(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<RunSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_runs(owner, repo, limit, &trace)
    }

    pub fn rerun(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &RunActionInput,
        failed_only: bool,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.rerun(permission, input, failed_only, &trace)
    }

    pub fn cancel(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &RunActionInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.cancel(permission, input, &trace)
    }

    pub fn view_run(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> Result<RunDetail, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.view_run(owner, repo, run_id, &trace)
    }

    pub fn view_run_logs(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> Result<String, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.view_run_logs(owner, repo, run_id, &trace)
    }
}
