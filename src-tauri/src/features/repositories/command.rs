use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{BranchSummary, CommitSummary, RepoSummary};
use super::service::{
    CreateBranchInput, CreateRepositoryInput, DeleteBranchInput, EditRepositoryInput,
    RepositoriesService,
};

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

    pub fn edit_repository(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &EditRepositoryInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.edit(permission, input, &trace)
    }

    pub fn list_branches(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<BranchSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_branches(owner, repo, limit, &trace)
    }

    pub fn list_commits(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
        limit: u16,
    ) -> Result<Vec<CommitSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_commits(owner, repo, branch, limit, &trace)
    }

    pub fn create_branch(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreateBranchInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create_branch(permission, input, &trace)
    }

    pub fn delete_branch(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeleteBranchInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete_branch(permission, input, &trace)
    }
}
