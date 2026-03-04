use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{
    PullRequestComment, PullRequestCreated, PullRequestDetail, PullRequestDiffFile,
    PullRequestRawDiff, PullRequestReviewThread, PullRequestSummary,
};
use super::service::{
    ClosePullRequestInput, CommentPullRequestInput, CreatePullRequestInput,
    CreateReviewCommentInput, EditPullRequestInput, MergePullRequestInput, PullRequestsService,
    ReopenPullRequestInput, ReplyReviewCommentInput, ResolveReviewThreadInput,
    ReviewPullRequestInput,
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

    pub fn view_pull_request(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<PullRequestDetail, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.view(owner, repo, number, &trace)
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

    pub fn edit_pull_request(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &EditPullRequestInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.edit(permission, input, &trace)
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

    pub fn close_pull_request(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ClosePullRequestInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.close(permission, input, &trace)
    }

    pub fn reopen_pull_request(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ReopenPullRequestInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.reopen(permission, input, &trace)
    }

    pub fn list_pull_request_comments(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<Vec<PullRequestComment>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_issue_comments(owner, repo, number, &trace)
    }

    pub fn create_pull_request_comment(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CommentPullRequestInput,
    ) -> Result<PullRequestComment, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create_issue_comment(permission, input, &trace)
    }

    pub fn list_pull_request_review_comments(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<Vec<PullRequestComment>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_review_comments(owner, repo, number, &trace)
    }

    pub fn create_pull_request_review_comment(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreateReviewCommentInput,
    ) -> Result<PullRequestComment, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .create_review_comment(permission, input, &trace)
    }

    pub fn reply_pull_request_review_comment(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ReplyReviewCommentInput,
    ) -> Result<PullRequestComment, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.reply_review_comment(permission, input, &trace)
    }

    pub fn list_pull_request_review_threads(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<Vec<PullRequestReviewThread>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_review_threads(owner, repo, number, &trace)
    }

    pub fn resolve_pull_request_review_thread(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ResolveReviewThreadInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .resolve_review_thread(permission, input, &trace)
    }

    pub fn unresolve_pull_request_review_thread(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &ResolveReviewThreadInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .unresolve_review_thread(permission, input, &trace)
    }

    pub fn list_pull_request_diff_files(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<Vec<PullRequestDiffFile>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_diff_files(owner, repo, number, &trace)
    }

    pub fn get_pull_request_raw_diff(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<PullRequestRawDiff, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.get_raw_diff(owner, repo, number, &trace)
    }
}
