use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{ReleaseCreated, ReleaseSummary};
use super::service::{CreateReleaseInput, DeleteReleaseInput, ReleasesService};

pub struct ReleasesCommandHandler<R: Runner> {
    service: ReleasesService<R>,
}

impl<R: Runner> ReleasesCommandHandler<R> {
    pub fn new(service: ReleasesService<R>) -> Self {
        Self { service }
    }

    pub fn list_releases(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<ReleaseSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list(owner, repo, limit, &trace)
    }

    pub fn create_release(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreateReleaseInput,
    ) -> Result<ReleaseCreated, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create(permission, input, &trace)
    }

    pub fn delete_release(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeleteReleaseInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete(permission, input, &trace)
    }
}
