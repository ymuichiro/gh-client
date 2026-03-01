use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{AddedProjectItem, ProjectItemSummary, ProjectSummary};
use super::service::{AddProjectItemInput, ProjectsService};

pub struct ProjectsCommandHandler<R: Runner> {
    service: ProjectsService<R>,
}

impl<R: Runner> ProjectsCommandHandler<R> {
    pub fn new(service: ProjectsService<R>) -> Self {
        Self { service }
    }

    pub fn list_projects(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<ProjectSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list(owner, repo, limit, &trace)
    }

    pub fn list_project_items(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        project_number: u32,
        limit: u16,
    ) -> Result<Vec<ProjectItemSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_items(owner, repo, project_number, limit, &trace)
    }

    pub fn add_project_item(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &AddProjectItemInput,
    ) -> Result<AddedProjectItem, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.add_item(permission, input, &trace)
    }
}
