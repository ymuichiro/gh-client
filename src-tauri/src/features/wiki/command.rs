use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::WikiInfo;
use super::service::{UpdateWikiInput, WikiService};

pub struct WikiCommandHandler<R: Runner> {
    service: WikiService<R>,
}

impl<R: Runner> WikiCommandHandler<R> {
    pub fn new(service: WikiService<R>) -> Self {
        Self { service }
    }

    pub fn get_wiki(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
    ) -> Result<WikiInfo, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.get(owner, repo, &trace)
    }

    pub fn update_wiki(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &UpdateWikiInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.update(permission, input, &trace)
    }
}
