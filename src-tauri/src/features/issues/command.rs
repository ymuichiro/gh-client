use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{IssueCreated, IssueSummary};
use super::service::{
    CloseIssueInput, CommentIssueInput, CreateIssueInput, EditIssueInput, IssuesService,
    ReopenIssueInput,
};

pub struct IssuesCommandHandler<R: Runner> {
    service: IssuesService<R>,
}

impl<R: Runner> IssuesCommandHandler<R> {
    pub fn new(service: IssuesService<R>) -> Self {
        Self { service }
    }

    pub fn list_issues(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<IssueSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list(owner, repo, limit, &trace)
    }

    pub fn create_issue(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreateIssueInput,
    ) -> Result<IssueCreated, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create(permission, input, &trace)
    }

    pub fn comment_issue(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CommentIssueInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.comment(permission, input, &trace)
    }

    pub fn close_issue(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CloseIssueInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.close(permission, input, &trace)
    }

    pub fn edit_issue(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &EditIssueInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.edit(permission, input, &trace)
    }

    pub fn reopen_issue(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ReopenIssueInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.reopen(permission, input, &trace)
    }
}
