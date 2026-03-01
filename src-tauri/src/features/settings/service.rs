use std::collections::HashMap;

use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{
    BranchProtection, Collaborator, DependabotAlert, DeployKeySummary, SecretSummary,
    VariableSummary, WebhookSummary, parse_branch_protection, parse_collaborators,
    parse_dependabot_alerts, parse_deploy_keys, parse_secret_summaries, parse_variable_summaries,
    parse_webhook_summaries,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollaboratorPermission {
    Pull,
    Push,
    Admin,
    Maintain,
    Triage,
}

impl CollaboratorPermission {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::Pull => "pull",
            Self::Push => "push",
            Self::Admin => "admin",
            Self::Maintain => "maintain",
            Self::Triage => "triage",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretApp {
    Actions,
    Dependabot,
    Codespaces,
}

impl SecretApp {
    fn as_flag_value(self) -> &'static str {
        match self {
            Self::Actions => "actions",
            Self::Dependabot => "dependabot",
            Self::Codespaces => "codespaces",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookContentType {
    Json,
    Form,
}

impl WebhookContentType {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Form => "form",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddCollaboratorInput {
    pub owner: String,
    pub repo: String,
    pub username: String,
    pub permission: CollaboratorPermission,
}

impl AddCollaboratorInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("username", &self.username)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveCollaboratorInput {
    pub owner: String,
    pub repo: String,
    pub username: String,
}

impl RemoveCollaboratorInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("username", &self.username)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSecretInput {
    pub owner: String,
    pub repo: String,
    pub name: String,
    pub value: String,
    pub app: Option<SecretApp>,
}

impl SetSecretInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("name", &self.name)?;
        validate_required_field("value", &self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteSecretInput {
    pub owner: String,
    pub repo: String,
    pub name: String,
    pub app: Option<SecretApp>,
}

impl DeleteSecretInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("name", &self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetVariableInput {
    pub owner: String,
    pub repo: String,
    pub name: String,
    pub value: String,
}

impl SetVariableInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("name", &self.name)?;
        validate_required_field("value", &self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteVariableInput {
    pub owner: String,
    pub repo: String,
    pub name: String,
}

impl DeleteVariableInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("name", &self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateWebhookInput {
    pub owner: String,
    pub repo: String,
    pub target_url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub content_type: WebhookContentType,
    pub secret: Option<String>,
}

impl CreateWebhookInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("target_url", &self.target_url)?;

        if self.events.is_empty() {
            return Err(AppError::validation(
                "events must contain at least one event",
            ));
        }

        for event in &self.events {
            if event.trim().is_empty() {
                return Err(AppError::validation("events must not contain empty values"));
            }
        }

        if self
            .secret
            .as_ref()
            .is_some_and(|secret| secret.trim().is_empty())
        {
            return Err(AppError::validation(
                "secret must not be empty when provided",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingWebhookInput {
    pub owner: String,
    pub repo: String,
    pub hook_id: u64,
}

impl PingWebhookInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        if self.hook_id == 0 {
            return Err(AppError::validation("hook_id must be greater than 0"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteWebhookInput {
    pub owner: String,
    pub repo: String,
    pub hook_id: u64,
}

impl DeleteWebhookInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        if self.hook_id == 0 {
            return Err(AppError::validation("hook_id must be greater than 0"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchProtectionTarget {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

impl BranchProtectionTarget {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("branch", &self.branch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateBranchProtectionInput {
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub enforce_admins: Option<bool>,
    pub dismiss_stale_reviews: Option<bool>,
    pub require_code_owner_reviews: Option<bool>,
    pub required_approving_review_count: Option<u8>,
}

impl UpdateBranchProtectionInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("branch", &self.branch)?;

        let has_any_change = self.enforce_admins.is_some()
            || self.dismiss_stale_reviews.is_some()
            || self.require_code_owner_reviews.is_some()
            || self.required_approving_review_count.is_some();

        if !has_any_change {
            return Err(AppError::validation(
                "at least one branch protection field must be provided",
            ));
        }

        if self
            .required_approving_review_count
            .is_some_and(|count| count > 6)
        {
            return Err(AppError::validation(
                "required_approving_review_count must be between 0 and 6",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddDeployKeyInput {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub key: String,
    pub read_only: bool,
}

impl AddDeployKeyInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        validate_required_field("title", &self.title)?;
        validate_required_field("key", &self.key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteDeployKeyInput {
    pub owner: String,
    pub repo: String,
    pub key_id: u64,
}

impl DeleteDeployKeyInput {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_owner_repo(&self.owner, &self.repo)?;
        if self.key_id == 0 {
            return Err(AppError::validation("key_id must be greater than 0"));
        }

        Ok(())
    }
}

fn validate_owner_repo(owner: &str, repo: &str) -> Result<(), AppError> {
    if owner.trim().is_empty() || repo.trim().is_empty() {
        return Err(AppError::validation("owner and repo are required"));
    }

    Ok(())
}

fn validate_required_field(name: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::validation(format!("{} is required", name)));
    }

    Ok(())
}

pub struct SettingsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> SettingsService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list_collaborators(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<Vec<Collaborator>, AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.collaborators.list",
        )?;
        validate_owner_repo(owner, repo)?;

        let args = vec![format!("repos/{}/{}/collaborators", owner, repo)];
        let req = self
            .registry
            .build_request("settings.collaborators.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_collaborators(&output.stdout)
    }

    pub fn add_collaborator(
        &self,
        permission: RepoPermission,
        input: &AddCollaboratorInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.collaborators.add",
        )?;
        input.validate()?;

        let args = vec![
            format!(
                "repos/{}/{}/collaborators/{}",
                input.owner, input.repo, input.username
            ),
            "-f".to_string(),
            format!("permission={}", input.permission.as_api_value()),
        ];

        let req = self
            .registry
            .build_request("settings.collaborators.add", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn remove_collaborator(
        &self,
        permission: RepoPermission,
        input: &RemoveCollaboratorInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.collaborators.remove",
        )?;
        input.validate()?;

        let args = vec![format!(
            "repos/{}/{}/collaborators/{}",
            input.owner, input.repo, input.username
        )];

        let req = self
            .registry
            .build_request("settings.collaborators.remove", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn list_secrets(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        app: Option<SecretApp>,
        trace: &TraceContext,
    ) -> Result<Vec<SecretSummary>, AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "settings.secrets.list")?;
        validate_owner_repo(owner, repo)?;

        let mut args = vec!["--repo".to_string(), format!("{}/{}", owner, repo)];
        if let Some(app) = app {
            args.push("--app".to_string());
            args.push(app.as_flag_value().to_string());
        }

        let req = self
            .registry
            .build_request("settings.secrets.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_secret_summaries(&output.stdout)
    }

    pub fn set_secret(
        &self,
        permission: RepoPermission,
        input: &SetSecretInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "settings.secrets.set")?;
        input.validate()?;

        let mut args = vec![
            input.name.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            "--body".to_string(),
            input.value.clone(),
        ];

        if let Some(app) = input.app {
            args.push("--app".to_string());
            args.push(app.as_flag_value().to_string());
        }

        let req = self.registry.build_request("settings.secrets.set", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn delete_secret(
        &self,
        permission: RepoPermission,
        input: &DeleteSecretInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "settings.secrets.delete")?;
        input.validate()?;

        let mut args = vec![
            input.name.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        if let Some(app) = input.app {
            args.push("--app".to_string());
            args.push(app.as_flag_value().to_string());
        }

        let req = self
            .registry
            .build_request("settings.secrets.delete", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn list_variables(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<Vec<VariableSummary>, AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "settings.variables.list")?;
        validate_owner_repo(owner, repo)?;

        let args = vec!["--repo".to_string(), format!("{}/{}", owner, repo)];
        let req = self
            .registry
            .build_request("settings.variables.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_variable_summaries(&output.stdout)
    }

    pub fn set_variable(
        &self,
        permission: RepoPermission,
        input: &SetVariableInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "settings.variables.set")?;
        input.validate()?;

        let args = vec![
            input.name.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            "--body".to_string(),
            input.value.clone(),
        ];

        let req = self
            .registry
            .build_request("settings.variables.set", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn delete_variable(
        &self,
        permission: RepoPermission,
        input: &DeleteVariableInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.variables.delete",
        )?;
        input.validate()?;

        let args = vec![
            input.name.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        let req = self
            .registry
            .build_request("settings.variables.delete", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn list_webhooks(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<Vec<WebhookSummary>, AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "settings.webhooks.list")?;
        validate_owner_repo(owner, repo)?;

        let args = vec![format!("repos/{}/{}/hooks", owner, repo)];
        let req = self
            .registry
            .build_request("settings.webhooks.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_webhook_summaries(&output.stdout)
    }

    pub fn create_webhook(
        &self,
        permission: RepoPermission,
        input: &CreateWebhookInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.webhooks.create",
        )?;
        input.validate()?;

        let mut args = vec![
            format!("repos/{}/{}/hooks", input.owner, input.repo),
            "-f".to_string(),
            "name=web".to_string(),
            "-f".to_string(),
            format!("config[url]={}", input.target_url),
            "-f".to_string(),
            format!("config[content_type]={}", input.content_type.as_api_value()),
            "-F".to_string(),
            format!("active={}", input.active),
        ];

        if let Some(secret) = input.secret.as_ref() {
            args.push("-f".to_string());
            args.push(format!("config[secret]={}", secret));
        }

        for event in &input.events {
            args.push("-f".to_string());
            args.push(format!("events[]={}", event));
        }

        let req = self
            .registry
            .build_request("settings.webhooks.create", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn ping_webhook(
        &self,
        permission: RepoPermission,
        input: &PingWebhookInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "settings.webhooks.ping")?;
        input.validate()?;

        let args = vec![format!(
            "repos/{}/{}/hooks/{}/pings",
            input.owner, input.repo, input.hook_id
        )];
        let req = self
            .registry
            .build_request("settings.webhooks.ping", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn delete_webhook(
        &self,
        permission: RepoPermission,
        input: &DeleteWebhookInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.webhooks.delete",
        )?;
        input.validate()?;

        let args = vec![format!(
            "repos/{}/{}/hooks/{}",
            input.owner, input.repo, input.hook_id
        )];
        let req = self
            .registry
            .build_request("settings.webhooks.delete", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn get_branch_protection(
        &self,
        permission: RepoPermission,
        target: &BranchProtectionTarget,
        trace: &TraceContext,
    ) -> Result<BranchProtection, AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.branch_protection.get",
        )?;
        target.validate()?;

        let args = vec![format!(
            "repos/{}/{}/branches/{}/protection",
            target.owner, target.repo, target.branch
        )];
        let req = self
            .registry
            .build_request("settings.branch_protection.get", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_branch_protection(&output.stdout)
    }

    pub fn update_branch_protection(
        &self,
        permission: RepoPermission,
        input: &UpdateBranchProtectionInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.branch_protection.update",
        )?;
        input.validate()?;

        let target = BranchProtectionTarget {
            owner: input.owner.clone(),
            repo: input.repo.clone(),
            branch: input.branch.clone(),
        };
        let current = self.get_branch_protection(permission, &target, trace)?;

        let current_reviews = current.required_pull_request_reviews;
        let dismiss_stale_reviews = input.dismiss_stale_reviews.or_else(|| {
            current_reviews
                .as_ref()
                .and_then(|value| value.dismiss_stale_reviews)
        });
        let require_code_owner_reviews = input.require_code_owner_reviews.or_else(|| {
            current_reviews
                .as_ref()
                .and_then(|value| value.require_code_owner_reviews)
        });
        let required_approving_review_count = input.required_approving_review_count.or_else(|| {
            current_reviews
                .as_ref()
                .and_then(|value| value.required_approving_review_count)
        });

        let enforce_admins = input.enforce_admins.or_else(|| {
            current
                .enforce_admins
                .as_ref()
                .and_then(|value| value.enabled)
        });

        let mut fields = HashMap::new();
        fields.insert(
            "enforce_admins".to_string(),
            enforce_admins.unwrap_or(false).to_string(),
        );
        fields.insert(
            "required_pull_request_reviews[dismiss_stale_reviews]".to_string(),
            dismiss_stale_reviews.unwrap_or(false).to_string(),
        );
        fields.insert(
            "required_pull_request_reviews[require_code_owner_reviews]".to_string(),
            require_code_owner_reviews.unwrap_or(false).to_string(),
        );
        fields.insert(
            "required_pull_request_reviews[required_approving_review_count]".to_string(),
            required_approving_review_count.unwrap_or(0).to_string(),
        );

        let mut args = vec![format!(
            "repos/{}/{}/branches/{}/protection",
            input.owner, input.repo, input.branch
        )];

        args.push("-F".to_string());
        args.push("required_status_checks=null".to_string());
        args.push("-F".to_string());
        args.push("restrictions=null".to_string());

        for key in [
            "enforce_admins",
            "required_pull_request_reviews[dismiss_stale_reviews]",
            "required_pull_request_reviews[require_code_owner_reviews]",
            "required_pull_request_reviews[required_approving_review_count]",
        ] {
            args.push("-F".to_string());
            args.push(format!(
                "{}={}",
                key,
                fields.get(key).expect("branch protection field must exist")
            ));
        }

        let req = self
            .registry
            .build_request("settings.branch_protection.update", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn list_deploy_keys(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<Vec<DeployKeySummary>, AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.deploy_keys.list",
        )?;
        validate_owner_repo(owner, repo)?;

        let args = vec![format!("repos/{}/{}/keys", owner, repo)];
        let req = self
            .registry
            .build_request("settings.deploy_keys.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_deploy_keys(&output.stdout)
    }

    pub fn add_deploy_key(
        &self,
        permission: RepoPermission,
        input: &AddDeployKeyInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.deploy_keys.add",
        )?;
        input.validate()?;

        let args = vec![
            format!("repos/{}/{}/keys", input.owner, input.repo),
            "-f".to_string(),
            format!("title={}", input.title),
            "-f".to_string(),
            format!("key={}", input.key),
            "-F".to_string(),
            format!("read_only={}", input.read_only),
        ];

        let req = self
            .registry
            .build_request("settings.deploy_keys.add", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn delete_deploy_key(
        &self,
        permission: RepoPermission,
        input: &DeleteDeployKeyInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.deploy_keys.delete",
        )?;
        input.validate()?;

        let args = vec![format!(
            "repos/{}/{}/keys/{}",
            input.owner, input.repo, input.key_id
        )];

        let req = self
            .registry
            .build_request("settings.deploy_keys.delete", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn list_dependabot_alerts(
        &self,
        permission: RepoPermission,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<DependabotAlert>, AppError> {
        self.policy_guard.require(
            RepoPermission::Admin,
            permission,
            "settings.dependabot_alerts.list",
        )?;
        validate_owner_repo(owner, repo)?;

        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![format!(
            "repos/{}/{}/dependabot/alerts?per_page={}",
            owner, repo, limit
        )];
        let req = self
            .registry
            .build_request("settings.dependabot_alerts.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_dependabot_alerts(&output.stdout)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::core::error::ErrorCode;
    use crate::core::executor::RawExecutionOutput;

    #[derive(Default)]
    struct RecordingState {
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl RecordingState {
        fn call_count(&self) -> usize {
            self.calls.lock().expect("lock poisoned").len()
        }

        fn last_call(&self) -> Option<(String, Vec<String>)> {
            self.calls.lock().expect("lock poisoned").last().cloned()
        }

        fn call_at(&self, index: usize) -> Option<(String, Vec<String>)> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .get(index)
                .cloned()
        }
    }

    struct RecordingRunner {
        state: Arc<RecordingState>,
        responses: Mutex<VecDeque<RawExecutionOutput>>,
    }

    impl RecordingRunner {
        fn new(responses: Vec<RawExecutionOutput>) -> (Self, Arc<RecordingState>) {
            let state = Arc::new(RecordingState::default());
            (
                Self {
                    state: Arc::clone(&state),
                    responses: Mutex::new(VecDeque::from(responses)),
                },
                state,
            )
        }
    }

    impl Runner for RecordingRunner {
        fn run(&self, program: &str, args: &[String]) -> io::Result<RawExecutionOutput> {
            self.state
                .calls
                .lock()
                .expect("lock poisoned")
                .push((program.to_string(), args.to_vec()));

            let response = self
                .responses
                .lock()
                .expect("lock poisoned")
                .pop_front()
                .unwrap_or(RawExecutionOutput {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                });

            Ok(response)
        }
    }

    fn trace() -> TraceContext {
        TraceContext::new("req-settings-service")
    }

    #[test]
    fn list_collaborators_requires_admin_permission() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let err = service
            .list_collaborators(RepoPermission::Write, "octocat", "hello", &trace())
            .expect_err("write permission should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn list_collaborators_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"login":"octocat","permissions":{"admin":true,"push":true,"pull":true}}]"#
                .into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_collaborators(RepoPermission::Admin, "octocat", "hello", &trace())
            .expect("list should succeed");
        assert_eq!(items.len(), 1);

        let (program, args) = state.last_call().expect("command should be called");
        assert_eq!(program, "gh");
        assert_eq!(args[0], "api");
    }

    #[test]
    fn add_collaborator_executes_command() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = AddCollaboratorInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            username: "hubot".into(),
            permission: CollaboratorPermission::Push,
        };

        service
            .add_collaborator(RepoPermission::Admin, &input, &trace())
            .expect("add collaborator should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"permission=push".to_string()));
    }

    #[test]
    fn remove_collaborator_is_noop_in_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = RemoveCollaboratorInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            username: "hubot".into(),
        };

        service
            .remove_collaborator(RepoPermission::Admin, &input, &trace())
            .expect("remove collaborator should no-op");

        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn list_secrets_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout:
                r#"[{"name":"TOKEN","updatedAt":"2026-01-01T00:00:00Z","visibility":"private"}]"#
                    .into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_secrets(
                RepoPermission::Admin,
                "octocat",
                "hello",
                Some(SecretApp::Actions),
                &trace(),
            )
            .expect("list secrets should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"secret".to_string()));
        assert!(args.contains(&"--app".to_string()));
    }

    #[test]
    fn set_secret_executes_command() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = SetSecretInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            name: "API_TOKEN".into(),
            value: "secret-value".into(),
            app: Some(SecretApp::Actions),
        };

        service
            .set_secret(RepoPermission::Admin, &input, &trace())
            .expect("set secret should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"API_TOKEN".to_string()));
        assert!(args.contains(&"--body".to_string()));
    }

    #[test]
    fn delete_secret_is_noop_in_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeleteSecretInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            name: "API_TOKEN".into(),
            app: None,
        };

        service
            .delete_secret(RepoPermission::Admin, &input, &trace())
            .expect("delete secret should no-op");

        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn list_variables_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"name":"ENV","value":"prod","updatedAt":"2026-01-01T00:00:00Z","createdAt":"2026-01-01T00:00:00Z","visibility":"private"}]"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_variables(RepoPermission::Admin, "octocat", "hello", &trace())
            .expect("list variables should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"variable".to_string()));
    }

    #[test]
    fn set_variable_executes_command() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = SetVariableInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            name: "APP_ENV".into(),
            value: "staging".into(),
        };

        service
            .set_variable(RepoPermission::Admin, &input, &trace())
            .expect("set variable should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"APP_ENV".to_string()));
        assert!(args.contains(&"--body".to_string()));
    }

    #[test]
    fn list_webhooks_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"id":1,"name":"web","active":true,"events":["push"],"config":{"url":"https://example.com","content_type":"json"}}]"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_webhooks(RepoPermission::Admin, "octocat", "hello", &trace())
            .expect("list webhooks should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert_eq!(args[0], "api");
        assert!(args[1].contains("/hooks"));
    }

    #[test]
    fn create_webhook_executes_command() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreateWebhookInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            target_url: "https://example.com/hook".into(),
            events: vec!["push".into(), "pull_request".into()],
            active: true,
            content_type: WebhookContentType::Json,
            secret: Some("abc123".into()),
        };

        service
            .create_webhook(RepoPermission::Admin, &input, &trace())
            .expect("create webhook should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"name=web".to_string()));
        assert!(args.contains(&"events[]=push".to_string()));
        assert!(args.contains(&"events[]=pull_request".to_string()));
    }

    #[test]
    fn get_branch_protection_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"url":"https://example.com","required_pull_request_reviews":{"required_approving_review_count":1,"dismiss_stale_reviews":true,"require_code_owner_reviews":false},"enforce_admins":{"enabled":true}}"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let target = BranchProtectionTarget {
            owner: "octocat".into(),
            repo: "hello".into(),
            branch: "main".into(),
        };

        let protection = service
            .get_branch_protection(RepoPermission::Admin, &target, &trace())
            .expect("get branch protection should succeed");
        assert_eq!(
            protection.enforce_admins.and_then(|value| value.enabled),
            Some(true)
        );

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args[1].contains("/protection"));
    }

    #[test]
    fn update_branch_protection_executes_get_then_put() {
        let get_output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"url":"https://example.com","required_pull_request_reviews":{"required_approving_review_count":1,"dismiss_stale_reviews":false,"require_code_owner_reviews":false},"enforce_admins":{"enabled":false}}"#.into(),
            stderr: String::new(),
        };
        let put_output = RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![get_output, put_output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = UpdateBranchProtectionInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            branch: "main".into(),
            enforce_admins: Some(true),
            dismiss_stale_reviews: Some(true),
            require_code_owner_reviews: Some(true),
            required_approving_review_count: Some(2),
        };

        service
            .update_branch_protection(RepoPermission::Admin, &input, &trace())
            .expect("update branch protection should succeed");

        assert_eq!(state.call_count(), 2);
        let second_call = state.call_at(1).expect("update call should exist");
        assert!(second_call.1.contains(&"--method".to_string()));
        assert!(second_call.1.contains(&"PUT".to_string()));
        assert!(second_call.1.contains(&"enforce_admins=true".to_string()));
        assert!(second_call.1.contains(
            &"required_pull_request_reviews[required_approving_review_count]=2".to_string()
        ));
    }

    #[test]
    fn list_deploy_keys_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"id":1,"title":"ci","key":"ssh-rsa AAA","read_only":true}]"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_deploy_keys(RepoPermission::Admin, "octocat", "hello", &trace())
            .expect("list deploy keys should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args[1].contains("/keys"));
    }

    #[test]
    fn add_deploy_key_executes_command() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = AddDeployKeyInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            title: "ci".into(),
            key: "ssh-rsa AAA".into(),
            read_only: true,
        };

        service
            .add_deploy_key(RepoPermission::Admin, &input, &trace())
            .expect("add deploy key should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"title=ci".to_string()));
        assert!(args.contains(&"read_only=true".to_string()));
    }

    #[test]
    fn delete_deploy_key_is_noop_in_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeleteDeployKeyInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            key_id: 1,
        };

        service
            .delete_deploy_key(RepoPermission::Admin, &input, &trace())
            .expect("delete deploy key should no-op");

        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn list_dependabot_alerts_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"number":1,"state":"open","dependency":{"package":{"name":"openssl","ecosystem":"npm"}}}]"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = SettingsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list_dependabot_alerts(RepoPermission::Admin, "octocat", "hello", 50, &trace())
            .expect("list dependabot alerts should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args[1].contains("dependabot/alerts"));
        assert!(args[1].contains("per_page=50"));
    }
}
