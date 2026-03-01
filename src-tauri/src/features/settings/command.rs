use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;

use super::dto::{
    BranchProtection, Collaborator, DependabotAlert, DeployKeySummary, SecretSummary,
    VariableSummary, WebhookSummary,
};
use super::service::{
    AddCollaboratorInput, AddDeployKeyInput, BranchProtectionTarget, CreateWebhookInput,
    DeleteDeployKeyInput, DeleteSecretInput, DeleteVariableInput, DeleteWebhookInput,
    PingWebhookInput, RemoveCollaboratorInput, SecretApp, SetSecretInput, SetVariableInput,
    SettingsService, UpdateBranchProtectionInput,
};

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

    pub fn list_secrets(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        app: Option<SecretApp>,
    ) -> Result<Vec<SecretSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_secrets(permission, owner, repo, app, &trace)
    }

    pub fn set_secret(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &SetSecretInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.set_secret(permission, input, &trace)
    }

    pub fn delete_secret(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeleteSecretInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete_secret(permission, input, &trace)
    }

    pub fn list_variables(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<VariableSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_variables(permission, owner, repo, &trace)
    }

    pub fn set_variable(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &SetVariableInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.set_variable(permission, input, &trace)
    }

    pub fn delete_variable(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeleteVariableInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete_variable(permission, input, &trace)
    }

    pub fn list_webhooks(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<WebhookSummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_webhooks(permission, owner, repo, &trace)
    }

    pub fn create_webhook(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &CreateWebhookInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.create_webhook(permission, input, &trace)
    }

    pub fn ping_webhook(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &PingWebhookInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.ping_webhook(permission, input, &trace)
    }

    pub fn delete_webhook(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeleteWebhookInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete_webhook(permission, input, &trace)
    }

    pub fn get_branch_protection(
        &self,
        request_id: &str,
        permission: RepoPermission,
        target: &BranchProtectionTarget,
    ) -> Result<BranchProtection, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .get_branch_protection(permission, target, &trace)
    }

    pub fn update_branch_protection(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &UpdateBranchProtectionInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .update_branch_protection(permission, input, &trace)
    }

    pub fn list_deploy_keys(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<DeployKeySummary>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_deploy_keys(permission, owner, repo, &trace)
    }

    pub fn add_deploy_key(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &AddDeployKeyInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.add_deploy_key(permission, input, &trace)
    }

    pub fn delete_deploy_key(
        &self,
        request_id: &str,
        permission: RepoPermission,
        input: &DeleteDeployKeyInput,
    ) -> Result<(), AppError> {
        let trace = TraceContext::new(request_id);
        self.service.delete_deploy_key(permission, input, &trace)
    }

    pub fn list_dependabot_alerts(
        &self,
        request_id: &str,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        limit: u16,
    ) -> Result<Vec<DependabotAlert>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service
            .list_dependabot_alerts(permission, owner, repo, limit, &trace)
    }
}
