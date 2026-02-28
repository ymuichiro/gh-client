use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::RepoSummary;
use super::service::{CreateRepositoryInput, RepositoriesService};

pub struct RepositoriesCommandHandler<R: Runner> {
    service: RepositoriesService<R>,
}

impl<R: Runner> RepositoriesCommandHandler<R> {
    pub fn new(service: RepositoriesService<R>) -> Self {
        Self { service }
    }

    pub fn list_repositories(
        &self,
        request_id: &str,
        owner: &str,
        limit: u16,
    ) -> Result<Vec<RepoSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list(owner, limit, &trace)
    }

    pub fn create_repository(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreateRepositoryInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create(permission, input, &trace)
    }

    pub fn delete_repository(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete(permission, owner, repo, &trace)
    }
}
