use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::PagesInfo;
use super::service::{ConfigurePagesInput, DeletePagesInput, PagesService};

pub struct PagesCommandHandler<R: Runner> {
    service: PagesService<R>,
}

impl<R: Runner> PagesCommandHandler<R> {
    pub fn new(service: PagesService<R>) -> Self {
        Self { service }
    }

    pub fn get_pages(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
    ) -> Result<PagesInfo, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.get(owner, repo, &trace)
    }

    pub fn create_pages(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ConfigurePagesInput,
    ) -> Result<PagesInfo, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create(permission, input, &trace)
    }

    pub fn update_pages(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ConfigurePagesInput,
    ) -> Result<PagesInfo, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.update(permission, input, &trace)
    }

    pub fn delete_pages(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeletePagesInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete(permission, input, &trace)
    }
}
