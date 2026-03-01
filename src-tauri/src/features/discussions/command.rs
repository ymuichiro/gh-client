use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{DiscussionCategory, DiscussionCreated, DiscussionSummary};
use super::service::{
    CloseDiscussionInput, CreateDiscussionInput, DiscussionsService, MarkAnswerInput,
};

pub struct DiscussionsCommandHandler<R: Runner> {
    service: DiscussionsService<R>,
}

impl<R: Runner> DiscussionsCommandHandler<R> {
    pub fn new(service: DiscussionsService<R>) -> Self {
        Self { service }
    }

    pub fn list_categories(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<DiscussionCategory>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_categories(owner, repo, limit, &trace)
    }

    pub fn list_discussions(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<DiscussionSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list(owner, repo, limit, &trace)
    }

    pub fn create_discussion(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreateDiscussionInput,
    ) -> Result<DiscussionCreated, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create(permission, input, &trace)
    }

    pub fn close_discussion(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CloseDiscussionInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.close(permission, input, &trace)
    }

    pub fn mark_discussion_answer(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &MarkAnswerInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.mark_answer(permission, input, &trace)
    }
}
