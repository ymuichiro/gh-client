use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::Collaborator;
use super::service::{AddCollaboratorInput, RemoveCollaboratorInput, SettingsService};

pub struct SettingsCommandHandler<R: Runner> {
    service: SettingsService<R>,
}

impl<R: Runner> SettingsCommandHandler<R> {
    pub fn new(service: SettingsService<R>) -> Self {
        Self { service }
    }

    pub fn list_collaborators(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Collaborator>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_collaborators(permission, owner, repo, &trace)
    }

    pub fn add_collaborator(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &AddCollaboratorInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.add_collaborator(permission, input, &trace)
    }

    pub fn remove_collaborator(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &RemoveCollaboratorInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.remove_collaborator(permission, input, &trace)
    }
}
