use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{PullRequestCreated, PullRequestSummary};
use super::service::{
    CreatePullRequestInput, MergePullRequestInput, PullRequestsService, ReviewPullRequestInput,
};

pub struct PullRequestsCommandHandler<R: Runner> {
    service: PullRequestsService<R>,
}

impl<R: Runner> PullRequestsCommandHandler<R> {
    pub fn new(service: PullRequestsService<R>) -> Self {
        Self { service }
    }

    pub fn list_pull_requests(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<PullRequestSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list(owner, repo, limit, &trace)
    }

    pub fn create_pull_request(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreatePullRequestInput,
    ) -> Result<PullRequestCreated, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create(permission, input, &trace)
    }

    pub fn review_pull_request(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ReviewPullRequestInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.review(permission, input, &trace)
    }

    pub fn merge_pull_request(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &MergePullRequestInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.merge(permission, input, &trace)
    }
}
