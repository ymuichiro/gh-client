use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::contract::{ContractRepoPermission, FrontendCommandEnvelope};
use crate::core::command_registry::CommandRegistry;
use crate::core::error::{AppError, ErrorCode};
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::RepoPermission;
use crate::features::actions::service::{ActionsService, RunActionInput};
use crate::features::auth::service::AuthService;
use crate::features::discussions::service::{
    CloseDiscussionInput, CreateDiscussionInput, DiscussionsService, MarkAnswerInput,
};
use crate::features::insights::service::InsightsService;
use crate::features::issues::service::{
    CloseIssueInput, CloseReason, CommentIssueInput, CreateIssueInput, EditIssueInput,
    IssuesService, ReopenIssueInput,
};
use crate::features::pages::service::{ConfigurePagesInput, DeletePagesInput, PagesService};
use crate::features::projects::service::{AddProjectItemInput, ProjectsService};
use crate::features::pull_requests::service::{
    ClosePullRequestInput, CommentPullRequestInput, CreatePullRequestInput,
    CreateReviewCommentInput, EditPullRequestInput, MergeMethod, MergePullRequestInput,
    PullRequestsService, ReopenPullRequestInput, ReplyReviewCommentInput, ResolveReviewThreadInput,
    ReviewEvent, ReviewPullRequestInput,
};
use crate::features::releases::service::{
    CreateReleaseInput, DeleteReleaseAssetInput, DeleteReleaseInput, EditReleaseInput,
    ReleasesService, UploadReleaseAssetInput,
};
use crate::features::repositories::service::{
    CreateBranchInput, CreateRepositoryInput, DeleteBranchInput, EditRepositoryInput,
    RepositoriesService, RepositoryVisibility,
};
use crate::features::rulesets::service::{
    DeleteRulesetInput, RulesetField, RulesetsService, UpsertRulesetInput,
};
use crate::features::settings::service::{
    AddCollaboratorInput, AddDeployKeyInput, BranchProtectionTarget, CollaboratorPermission,
    CreateWebhookInput, DeleteDeployKeyInput, DeleteSecretInput, DeleteVariableInput,
    DeleteWebhookInput, PingWebhookInput, RemoveCollaboratorInput, SecretApp, SetSecretInput,
    SetVariableInput, SettingsService, UpdateBranchProtectionInput, WebhookContentType,
};
use crate::features::wiki::service::{UpdateWikiInput, WikiService};

pub const SUPPORTED_COMMAND_IDS: &[&str] = &[
    "auth.organizations.list",
    "auth.status",
    "repo.list",
    "repo.create",
    "repo.edit",
    "repo.topics.replace",
    "repo.branches.list",
    "repo.branch.ref.get",
    "repo.branch.create",
    "repo.branch.delete",
    "repo.commits.list",
    "repo.delete",
    "pr.list",
    "pr.create",
    "pr.view",
    "pr.review",
    "pr.edit",
    "pr.close",
    "pr.reopen",
    "pr.merge",
    "pr.comments.list",
    "pr.comments.create",
    "pr.review_comments.list",
    "pr.review_comments.create",
    "pr.review_comments.reply",
    "pr.review_threads.list",
    "pr.review_threads.resolve",
    "pr.review_threads.unresolve",
    "pr.diff.files.list",
    "pr.diff.raw.get",
    "issue.list",
    "issue.create",
    "issue.comment",
    "issue.edit",
    "issue.close",
    "issue.reopen",
    "workflow.list",
    "run.list",
    "run.rerun",
    "run.view",
    "run.logs",
    "run.cancel",
    "release.list",
    "release.create",
    "release.edit",
    "release.asset.upload",
    "release.asset.delete",
    "release.delete",
    "settings.collaborators.list",
    "settings.collaborators.add",
    "settings.collaborators.remove",
    "settings.secrets.list",
    "settings.secrets.set",
    "settings.secrets.delete",
    "settings.variables.list",
    "settings.variables.set",
    "settings.variables.delete",
    "settings.webhooks.list",
    "settings.webhooks.create",
    "settings.webhooks.ping",
    "settings.webhooks.delete",
    "settings.branch_protection.get",
    "settings.branch_protection.update",
    "settings.deploy_keys.list",
    "settings.deploy_keys.add",
    "settings.deploy_keys.delete",
    "settings.dependabot_alerts.list",
    "insights.views.get",
    "insights.clones.get",
    "projects.list",
    "projects.items.list",
    "projects.items.add",
    "discussions.categories.list",
    "discussions.list",
    "discussions.create",
    "discussions.close",
    "discussions.answer",
    "wiki.get",
    "wiki.update",
    "pages.get",
    "pages.create",
    "pages.update",
    "pages.delete",
    "rulesets.list",
    "rulesets.get",
    "rulesets.create",
    "rulesets.update",
    "rulesets.delete",
];

pub struct FrontendDispatcher<R: Runner + Clone> {
    registry: CommandRegistry,
    runner: R,
    safe_test_mode: bool,
}

impl<R: Runner + Clone> FrontendDispatcher<R> {
    pub fn new(runner: R, safe_test_mode: bool) -> Result<Self, AppError> {
        let registry = CommandRegistry::with_defaults();
        crate::contract::validate_registry_contract(&registry)?;

        Ok(Self {
            registry,
            runner,
            safe_test_mode,
        })
    }

    pub fn supported_command_ids() -> &'static [&'static str] {
        SUPPORTED_COMMAND_IDS
    }

    pub fn execute_envelope(&self, envelope: FrontendCommandEnvelope) -> Result<Value, AppError> {
        envelope.validate(&self.registry)?;

        if !Self::supported_command_ids().contains(&envelope.command_id.as_str()) {
            return Err(AppError::validation(format!(
                "command_id `{}` is not supported by frontend dispatcher",
                envelope.command_id
            )));
        }

        let FrontendCommandEnvelope {
            request_id,
            command_id,
            permission,
            payload,
            ..
        } = envelope;

        let permission = map_permission(permission);

        match command_id.as_str() {
            "auth.status" => {
                let status = self.auth_service().status(&TraceContext::new(request_id))?;
                to_json(status)
            }
            "auth.organizations.list" => {
                let _: serde_json::Map<String, Value> = parse_payload(payload)?;
                let organizations = self
                    .auth_service()
                    .list_organizations(&TraceContext::new(request_id))?;
                to_json(organizations)
            }

            "repo.list" => {
                let p: OwnerLimitPayload = parse_payload(payload)?;
                let result =
                    self.repositories_service()
                        .list(&p.owner, p.limit, &trace(&request_id))?;
                to_json(result)
            }
            "repo.create" => {
                let p: RepoCreatePayload = parse_payload(payload)?;
                let input = CreateRepositoryInput {
                    owner: p.owner,
                    name: p.name,
                    private: p.private,
                    description: p.description,
                };
                self.repositories_service()
                    .create(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "repo.edit" => {
                let p: RepoEditPayload = parse_payload(payload)?;
                let input = EditRepositoryInput {
                    owner: p.owner,
                    repo: p.repo,
                    description: p.description,
                    homepage: p.homepage,
                    default_branch: p.default_branch,
                    visibility: parse_repo_visibility(p.visibility)?,
                    add_topics: p.add_topics,
                    remove_topics: p.remove_topics,
                    replace_topics: p.replace_topics,
                };
                self.repositories_service()
                    .edit(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "repo.topics.replace" => {
                let p: RawArgsPayload = parse_payload(payload)?;
                self.execute_raw_command(&command_id, &request_id, p.args)
            }
            "repo.branches.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.repositories_service().list_branches(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "repo.branch.ref.get" => {
                let p: RawArgsPayload = parse_payload(payload)?;
                self.execute_raw_command(&command_id, &request_id, p.args)
            }
            "repo.branch.create" => {
                let p: RepoCreateBranchPayload = parse_payload(payload)?;
                let input = CreateBranchInput {
                    owner: p.owner,
                    repo: p.repo,
                    branch: p.branch,
                    from_branch: p.from_branch,
                };
                self.repositories_service().create_branch(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "repo.branch.delete" => {
                let p: RepoDeleteBranchPayload = parse_payload(payload)?;
                let input = DeleteBranchInput {
                    owner: p.owner,
                    repo: p.repo,
                    branch: p.branch,
                };
                self.repositories_service().delete_branch(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "repo.commits.list" => {
                let p: RepoCommitsPayload = parse_payload(payload)?;
                let result = self.repositories_service().list_commits(
                    &p.owner,
                    &p.repo,
                    p.branch.as_deref(),
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "repo.delete" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                self.repositories_service().delete(
                    permission,
                    &p.owner,
                    &p.repo,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }

            "pr.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.pull_requests_service().list(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.view" => {
                let p: PullRequestNumberPayload = parse_payload(payload)?;
                let result = self.pull_requests_service().view(
                    &p.owner,
                    &p.repo,
                    p.number,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.create" => {
                let p: PullRequestCreatePayload = parse_payload(payload)?;
                let input = CreatePullRequestInput {
                    owner: p.owner,
                    repo: p.repo,
                    title: p.title,
                    head: p.head,
                    base: p.base,
                    body: p.body,
                    draft: p.draft,
                };
                let created =
                    self.pull_requests_service()
                        .create(permission, &input, &trace(&request_id))?;
                to_json(created)
            }
            "pr.review" => {
                let p: PullRequestReviewPayload = parse_payload(payload)?;
                let input = ReviewPullRequestInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    event: parse_review_event(&p.event)?,
                    body: p.body,
                };
                self.pull_requests_service()
                    .review(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "pr.edit" => {
                let p: PullRequestEditPayload = parse_payload(payload)?;
                let input = EditPullRequestInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    title: p.title,
                    body: p.body,
                    base: p.base,
                };
                self.pull_requests_service()
                    .edit(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "pr.close" => {
                let p: PullRequestClosePayload = parse_payload(payload)?;
                let input = ClosePullRequestInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    comment: p.comment,
                    delete_branch: p.delete_branch,
                };
                self.pull_requests_service()
                    .close(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "pr.reopen" => {
                let p: PullRequestReopenPayload = parse_payload(payload)?;
                let input = ReopenPullRequestInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    comment: p.comment,
                };
                self.pull_requests_service()
                    .reopen(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "pr.merge" => {
                let p: PullRequestMergePayload = parse_payload(payload)?;
                let input = MergePullRequestInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    method: parse_merge_method(&p.method)?,
                    delete_branch: p.delete_branch,
                    auto: p.auto,
                };
                self.pull_requests_service()
                    .merge(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "pr.comments.list" => {
                let p: PullRequestNumberPayload = parse_payload(payload)?;
                let result = self.pull_requests_service().list_issue_comments(
                    &p.owner,
                    &p.repo,
                    p.number,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.comments.create" => {
                let p: PullRequestCommentPayload = parse_payload(payload)?;
                let input = CommentPullRequestInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    body: p.body,
                };
                let result = self.pull_requests_service().create_issue_comment(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.review_comments.list" => {
                let p: PullRequestNumberPayload = parse_payload(payload)?;
                let result = self.pull_requests_service().list_review_comments(
                    &p.owner,
                    &p.repo,
                    p.number,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.review_comments.create" => {
                let p: PullRequestReviewCommentCreatePayload = parse_payload(payload)?;
                let input = CreateReviewCommentInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    commit_id: p.commit_id,
                    path: p.path,
                    line: p.line,
                    body: p.body,
                    side: p.side.map(|v| v.to_ascii_uppercase()),
                };
                let result = self.pull_requests_service().create_review_comment(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.review_comments.reply" => {
                let p: PullRequestReviewCommentReplyPayload = parse_payload(payload)?;
                let input = ReplyReviewCommentInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    comment_id: p.comment_id,
                    body: p.body,
                };
                let result = self.pull_requests_service().reply_review_comment(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.review_threads.list" => {
                let p: PullRequestNumberPayload = parse_payload(payload)?;
                let result = self.pull_requests_service().list_review_threads(
                    &p.owner,
                    &p.repo,
                    p.number,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.review_threads.resolve" => {
                let p: ReviewThreadPayload = parse_payload(payload)?;
                let input = ResolveReviewThreadInput {
                    thread_id: p.thread_id,
                };
                self.pull_requests_service().resolve_review_thread(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "pr.review_threads.unresolve" => {
                let p: ReviewThreadPayload = parse_payload(payload)?;
                let input = ResolveReviewThreadInput {
                    thread_id: p.thread_id,
                };
                self.pull_requests_service().unresolve_review_thread(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "pr.diff.files.list" => {
                let p: PullRequestNumberPayload = parse_payload(payload)?;
                let result = self.pull_requests_service().list_diff_files(
                    &p.owner,
                    &p.repo,
                    p.number,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "pr.diff.raw.get" => {
                let p: PullRequestNumberPayload = parse_payload(payload)?;
                let result = self.pull_requests_service().get_raw_diff(
                    &p.owner,
                    &p.repo,
                    p.number,
                    &trace(&request_id),
                )?;
                to_json(result)
            }

            "issue.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result =
                    self.issues_service()
                        .list(&p.owner, &p.repo, p.limit, &trace(&request_id))?;
                to_json(result)
            }
            "issue.create" => {
                let p: IssueCreatePayload = parse_payload(payload)?;
                let input = CreateIssueInput {
                    owner: p.owner,
                    repo: p.repo,
                    title: p.title,
                    body: p.body,
                };
                let created =
                    self.issues_service()
                        .create(permission, &input, &trace(&request_id))?;
                Ok(json!({
                    "number": created.number,
                    "url": created.url,
                    "state": created.state,
                }))
            }
            "issue.comment" => {
                let p: IssueCommentPayload = parse_payload(payload)?;
                let input = CommentIssueInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    body: p.body,
                };
                self.issues_service()
                    .comment(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "issue.edit" => {
                let p: IssueEditPayload = parse_payload(payload)?;
                let input = EditIssueInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    title: p.title,
                    body: p.body,
                    add_assignees: p.add_assignees,
                    remove_assignees: p.remove_assignees,
                    add_labels: p.add_labels,
                    remove_labels: p.remove_labels,
                };
                self.issues_service()
                    .edit(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "issue.close" => {
                let p: IssueClosePayload = parse_payload(payload)?;
                let input = CloseIssueInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    comment: p.comment,
                    reason: parse_issue_close_reason(p.reason)?,
                };
                self.issues_service()
                    .close(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "issue.reopen" => {
                let p: IssueReopenPayload = parse_payload(payload)?;
                let input = ReopenIssueInput {
                    owner: p.owner,
                    repo: p.repo,
                    number: p.number,
                    comment: p.comment,
                };
                self.issues_service()
                    .reopen(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }

            "workflow.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.actions_service().list_workflows(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "run.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.actions_service().list_runs(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "run.rerun" => {
                let p: RunRerunPayload = parse_payload(payload)?;
                let input = RunActionInput {
                    owner: p.owner,
                    repo: p.repo,
                    run_id: p.run_id,
                };
                self.actions_service().rerun(
                    permission,
                    &input,
                    p.failed_only,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "run.view" => {
                let p: RunPayload = parse_payload(payload)?;
                let result = self.actions_service().view_run(
                    &p.owner,
                    &p.repo,
                    p.run_id,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "run.logs" => {
                let p: RunPayload = parse_payload(payload)?;
                let logs = self.actions_service().view_run_logs(
                    &p.owner,
                    &p.repo,
                    p.run_id,
                    &trace(&request_id),
                )?;
                Ok(json!({ "logs": logs }))
            }
            "run.cancel" => {
                let p: RunPayload = parse_payload(payload)?;
                let input = RunActionInput {
                    owner: p.owner,
                    repo: p.repo,
                    run_id: p.run_id,
                };
                self.actions_service()
                    .cancel(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }

            "release.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.releases_service().list(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "release.create" => {
                let p: ReleaseCreatePayload = parse_payload(payload)?;
                let input = CreateReleaseInput {
                    owner: p.owner,
                    repo: p.repo,
                    tag: p.tag,
                    title: p.title,
                    notes: p.notes,
                    draft: p.draft,
                    prerelease: p.prerelease,
                    target: p.target,
                };
                let created =
                    self.releases_service()
                        .create(permission, &input, &trace(&request_id))?;
                Ok(json!({
                    "tag_name": created.tag_name,
                    "url": created.url,
                }))
            }
            "release.edit" => {
                let p: ReleaseEditPayload = parse_payload(payload)?;
                let input = EditReleaseInput {
                    owner: p.owner,
                    repo: p.repo,
                    tag: p.tag,
                    title: p.title,
                    notes: p.notes,
                    draft: p.draft,
                    prerelease: p.prerelease,
                };
                self.releases_service()
                    .edit(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "release.asset.upload" => {
                let p: ReleaseAssetUploadPayload = parse_payload(payload)?;
                let input = UploadReleaseAssetInput {
                    owner: p.owner,
                    repo: p.repo,
                    tag: p.tag,
                    file_path: p.file_path,
                    clobber: p.clobber,
                };
                self.releases_service()
                    .upload_asset(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "release.asset.delete" => {
                let p: ReleaseAssetDeletePayload = parse_payload(payload)?;
                let input = DeleteReleaseAssetInput {
                    owner: p.owner,
                    repo: p.repo,
                    tag: p.tag,
                    asset_name: p.asset_name,
                };
                self.releases_service()
                    .delete_asset(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "release.delete" => {
                let p: ReleaseDeletePayload = parse_payload(payload)?;
                let input = DeleteReleaseInput {
                    owner: p.owner,
                    repo: p.repo,
                    tag: p.tag,
                    cleanup_tag: p.cleanup_tag,
                };
                self.releases_service()
                    .delete(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }

            "settings.collaborators.list" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result = self.settings_service().list_collaborators(
                    permission,
                    &p.owner,
                    &p.repo,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "settings.collaborators.add" => {
                let p: SettingsAddCollaboratorPayload = parse_payload(payload)?;
                let input = AddCollaboratorInput {
                    owner: p.owner,
                    repo: p.repo,
                    username: p.username,
                    permission: parse_collaborator_permission(&p.permission)?,
                };
                self.settings_service().add_collaborator(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "settings.collaborators.remove" => {
                let p: SettingsRemoveCollaboratorPayload = parse_payload(payload)?;
                let input = RemoveCollaboratorInput {
                    owner: p.owner,
                    repo: p.repo,
                    username: p.username,
                };
                self.settings_service().remove_collaborator(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "settings.secrets.list" => {
                let p: SettingsSecretsListPayload = parse_payload(payload)?;
                let result = self.settings_service().list_secrets(
                    permission,
                    &p.owner,
                    &p.repo,
                    parse_secret_app(p.app)?,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "settings.secrets.set" => {
                let p: SettingsSetSecretPayload = parse_payload(payload)?;
                let input = SetSecretInput {
                    owner: p.owner,
                    repo: p.repo,
                    name: p.name,
                    value: p.value,
                    app: parse_secret_app(p.app)?,
                };
                self.settings_service()
                    .set_secret(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.secrets.delete" => {
                let p: SettingsDeleteSecretPayload = parse_payload(payload)?;
                let input = DeleteSecretInput {
                    owner: p.owner,
                    repo: p.repo,
                    name: p.name,
                    app: parse_secret_app(p.app)?,
                };
                self.settings_service()
                    .delete_secret(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.variables.list" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result = self.settings_service().list_variables(
                    permission,
                    &p.owner,
                    &p.repo,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "settings.variables.set" => {
                let p: SettingsSetVariablePayload = parse_payload(payload)?;
                let input = SetVariableInput {
                    owner: p.owner,
                    repo: p.repo,
                    name: p.name,
                    value: p.value,
                };
                self.settings_service()
                    .set_variable(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.variables.delete" => {
                let p: SettingsDeleteVariablePayload = parse_payload(payload)?;
                let input = DeleteVariableInput {
                    owner: p.owner,
                    repo: p.repo,
                    name: p.name,
                };
                self.settings_service()
                    .delete_variable(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.webhooks.list" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result = self.settings_service().list_webhooks(
                    permission,
                    &p.owner,
                    &p.repo,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "settings.webhooks.create" => {
                let p: SettingsCreateWebhookPayload = parse_payload(payload)?;
                let input = CreateWebhookInput {
                    owner: p.owner,
                    repo: p.repo,
                    target_url: p.target_url,
                    events: p.events,
                    active: p.active,
                    content_type: parse_webhook_content_type(p.content_type)?,
                    secret: p.secret,
                };
                self.settings_service()
                    .create_webhook(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.webhooks.ping" => {
                let p: SettingsWebhookByIdPayload = parse_payload(payload)?;
                let input = PingWebhookInput {
                    owner: p.owner,
                    repo: p.repo,
                    hook_id: p.hook_id,
                };
                self.settings_service()
                    .ping_webhook(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.webhooks.delete" => {
                let p: SettingsWebhookByIdPayload = parse_payload(payload)?;
                let input = DeleteWebhookInput {
                    owner: p.owner,
                    repo: p.repo,
                    hook_id: p.hook_id,
                };
                self.settings_service()
                    .delete_webhook(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.branch_protection.get" => {
                let p: SettingsBranchProtectionGetPayload = parse_payload(payload)?;
                let target = BranchProtectionTarget {
                    owner: p.owner,
                    repo: p.repo,
                    branch: p.branch,
                };
                let result = self.settings_service().get_branch_protection(
                    permission,
                    &target,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "settings.branch_protection.update" => {
                let p: SettingsBranchProtectionUpdatePayload = parse_payload(payload)?;
                let input = UpdateBranchProtectionInput {
                    owner: p.owner,
                    repo: p.repo,
                    branch: p.branch,
                    enforce_admins: p.enforce_admins,
                    dismiss_stale_reviews: p.dismiss_stale_reviews,
                    require_code_owner_reviews: p.require_code_owner_reviews,
                    required_approving_review_count: p.required_approving_review_count,
                };
                self.settings_service().update_branch_protection(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "settings.deploy_keys.list" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result = self.settings_service().list_deploy_keys(
                    permission,
                    &p.owner,
                    &p.repo,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "settings.deploy_keys.add" => {
                let p: SettingsAddDeployKeyPayload = parse_payload(payload)?;
                let input = AddDeployKeyInput {
                    owner: p.owner,
                    repo: p.repo,
                    title: p.title,
                    key: p.key,
                    read_only: p.read_only,
                };
                self.settings_service()
                    .add_deploy_key(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "settings.deploy_keys.delete" => {
                let p: SettingsDeleteDeployKeyPayload = parse_payload(payload)?;
                let input = DeleteDeployKeyInput {
                    owner: p.owner,
                    repo: p.repo,
                    key_id: p.key_id,
                };
                self.settings_service().delete_deploy_key(
                    permission,
                    &input,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "settings.dependabot_alerts.list" => {
                let p: SettingsDependabotListPayload = parse_payload(payload)?;
                let result = self.settings_service().list_dependabot_alerts(
                    permission,
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }

            "insights.views.get" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result =
                    self.insights_service()
                        .get_views(&p.owner, &p.repo, &trace(&request_id))?;
                to_json(result)
            }
            "insights.clones.get" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result =
                    self.insights_service()
                        .get_clones(&p.owner, &p.repo, &trace(&request_id))?;
                to_json(result)
            }

            "projects.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.projects_service().list(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "projects.items.list" => {
                let p: ProjectsItemsListPayload = parse_payload(payload)?;
                let result = self.projects_service().list_items(
                    &p.owner,
                    &p.repo,
                    p.project_number,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "projects.items.add" => {
                let p: ProjectsAddItemPayload = parse_payload(payload)?;
                let input = AddProjectItemInput {
                    project_id: p.project_id,
                    content_id: p.content_id,
                };
                let result =
                    self.projects_service()
                        .add_item(permission, &input, &trace(&request_id))?;
                to_json(result)
            }

            "discussions.categories.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.discussions_service().list_categories(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "discussions.list" => {
                let p: RepoLimitPayload = parse_payload(payload)?;
                let result = self.discussions_service().list(
                    &p.owner,
                    &p.repo,
                    p.limit,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "discussions.create" => {
                let p: DiscussionsCreatePayload = parse_payload(payload)?;
                let input = CreateDiscussionInput {
                    owner: p.owner,
                    repo: p.repo,
                    category_slug: p.category_slug,
                    title: p.title,
                    body: p.body,
                };
                let result =
                    self.discussions_service()
                        .create(permission, &input, &trace(&request_id))?;
                to_json(result)
            }
            "discussions.close" => {
                let p: DiscussionsClosePayload = parse_payload(payload)?;
                let input = CloseDiscussionInput {
                    discussion_id: p.discussion_id,
                };
                self.discussions_service()
                    .close(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "discussions.answer" => {
                let p: DiscussionsAnswerPayload = parse_payload(payload)?;
                let input = MarkAnswerInput {
                    comment_id: p.comment_id,
                };
                self.discussions_service()
                    .mark_answer(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }

            "wiki.get" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result = self
                    .wiki_service()
                    .get(&p.owner, &p.repo, &trace(&request_id))?;
                to_json(result)
            }
            "wiki.update" => {
                let p: WikiUpdatePayload = parse_payload(payload)?;
                let input = UpdateWikiInput {
                    owner: p.owner,
                    repo: p.repo,
                    enabled: p.enabled,
                };
                self.wiki_service()
                    .update(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }

            "pages.get" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result = self
                    .pages_service()
                    .get(&p.owner, &p.repo, &trace(&request_id))?;
                to_json(result)
            }
            "pages.create" => {
                let p: PagesConfigurePayload = parse_payload(payload)?;
                let input = ConfigurePagesInput {
                    owner: p.owner,
                    repo: p.repo,
                    branch: p.branch,
                    path: p.path,
                    build_type: p.build_type,
                    cname: p.cname,
                };
                let result =
                    self.pages_service()
                        .create(permission, &input, &trace(&request_id))?;
                to_json(result)
            }
            "pages.update" => {
                let p: PagesConfigurePayload = parse_payload(payload)?;
                let input = ConfigurePagesInput {
                    owner: p.owner,
                    repo: p.repo,
                    branch: p.branch,
                    path: p.path,
                    build_type: p.build_type,
                    cname: p.cname,
                };
                let result =
                    self.pages_service()
                        .update(permission, &input, &trace(&request_id))?;
                to_json(result)
            }
            "pages.delete" => {
                let p: PagesDeletePayload = parse_payload(payload)?;
                let input = DeletePagesInput {
                    owner: p.owner,
                    repo: p.repo,
                };
                self.pages_service()
                    .delete(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }

            "rulesets.list" => {
                let p: RepoScopePayload = parse_payload(payload)?;
                let result = self.rulesets_service().list(
                    permission,
                    &p.owner,
                    &p.repo,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "rulesets.get" => {
                let p: RulesetGetPayload = parse_payload(payload)?;
                let result = self.rulesets_service().get(
                    permission,
                    &p.owner,
                    &p.repo,
                    p.ruleset_id,
                    &trace(&request_id),
                )?;
                to_json(result)
            }
            "rulesets.create" => {
                let p: RulesetUpsertPayload = parse_payload(payload)?;
                let input = UpsertRulesetInput {
                    owner: p.owner,
                    repo: p.repo,
                    fields: p
                        .fields
                        .into_iter()
                        .map(|field| RulesetField {
                            key: field.key,
                            value: field.value,
                        })
                        .collect(),
                };
                self.rulesets_service()
                    .create(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }
            "rulesets.update" => {
                let p: RulesetUpdatePayload = parse_payload(payload)?;
                let input = UpsertRulesetInput {
                    owner: p.owner,
                    repo: p.repo,
                    fields: p
                        .fields
                        .into_iter()
                        .map(|field| RulesetField {
                            key: field.key,
                            value: field.value,
                        })
                        .collect(),
                };
                self.rulesets_service().update(
                    permission,
                    &input,
                    p.ruleset_id,
                    &trace(&request_id),
                )?;
                Ok(ok_value())
            }
            "rulesets.delete" => {
                let p: RulesetDeletePayload = parse_payload(payload)?;
                let input = DeleteRulesetInput {
                    owner: p.owner,
                    repo: p.repo,
                    ruleset_id: p.ruleset_id,
                };
                self.rulesets_service()
                    .delete(permission, &input, &trace(&request_id))?;
                Ok(ok_value())
            }

            _ => Err(AppError::validation(format!(
                "command_id `{}` is not routed in frontend dispatcher",
                command_id
            ))),
        }
    }

    fn execute_raw_command(
        &self,
        command_id: &str,
        request_id: &str,
        args: Vec<String>,
    ) -> Result<Value, AppError> {
        let req = self.registry.build_request(command_id, &args)?;
        let (output, _audit) = self.executor().execute(&req, &trace(request_id))?;
        Ok(json!({
            "command_id": output.command_id,
            "exit_code": output.exit_code,
            "stdout": output.stdout,
            "stderr": output.stderr,
            "noop": output.noop,
        }))
    }

    fn executor(&self) -> CommandExecutor<R> {
        CommandExecutor::new(self.runner.clone(), self.safe_test_mode)
    }

    fn auth_service(&self) -> AuthService<R> {
        AuthService::new(self.registry.clone(), self.executor())
    }

    fn repositories_service(&self) -> RepositoriesService<R> {
        RepositoriesService::new(self.registry.clone(), self.executor())
    }

    fn pull_requests_service(&self) -> PullRequestsService<R> {
        PullRequestsService::new(self.registry.clone(), self.executor())
    }

    fn issues_service(&self) -> IssuesService<R> {
        IssuesService::new(self.registry.clone(), self.executor())
    }

    fn actions_service(&self) -> ActionsService<R> {
        ActionsService::new(self.registry.clone(), self.executor())
    }

    fn releases_service(&self) -> ReleasesService<R> {
        ReleasesService::new(self.registry.clone(), self.executor())
    }

    fn settings_service(&self) -> SettingsService<R> {
        SettingsService::new(self.registry.clone(), self.executor())
    }

    fn insights_service(&self) -> InsightsService<R> {
        InsightsService::new(self.registry.clone(), self.executor())
    }

    fn projects_service(&self) -> ProjectsService<R> {
        ProjectsService::new(self.registry.clone(), self.executor())
    }

    fn discussions_service(&self) -> DiscussionsService<R> {
        DiscussionsService::new(self.registry.clone(), self.executor())
    }

    fn wiki_service(&self) -> WikiService<R> {
        WikiService::new(self.registry.clone(), self.executor())
    }

    fn pages_service(&self) -> PagesService<R> {
        PagesService::new(self.registry.clone(), self.executor())
    }

    fn rulesets_service(&self) -> RulesetsService<R> {
        RulesetsService::new(self.registry.clone(), self.executor())
    }
}

fn to_json<T: Serialize>(value: T) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to serialize dispatcher response: {}", err),
            false,
        )
    })
}

fn parse_payload<T: DeserializeOwned>(payload: Value) -> Result<T, AppError> {
    serde_json::from_value(payload)
        .map_err(|err| AppError::validation(format!("invalid payload: {}", err)))
}

fn trace(request_id: &str) -> TraceContext {
    TraceContext::new(request_id)
}

fn ok_value() -> Value {
    json!({ "ok": true })
}

fn map_permission(permission: Option<ContractRepoPermission>) -> RepoPermission {
    match permission.unwrap_or(ContractRepoPermission::Viewer) {
        ContractRepoPermission::Viewer => RepoPermission::Viewer,
        ContractRepoPermission::Write => RepoPermission::Write,
        ContractRepoPermission::Admin => RepoPermission::Admin,
    }
}

fn parse_repo_visibility(value: Option<String>) -> Result<Option<RepositoryVisibility>, AppError> {
    match value {
        None => Ok(None),
        Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "public" => Ok(Some(RepositoryVisibility::Public)),
            "private" => Ok(Some(RepositoryVisibility::Private)),
            "internal" => Ok(Some(RepositoryVisibility::Internal)),
            _ => Err(AppError::validation(format!("invalid visibility: {}", raw))),
        },
    }
}

fn parse_review_event(value: &str) -> Result<ReviewEvent, AppError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "approve" => Ok(ReviewEvent::Approve),
        "request_changes" | "request-changes" | "request changes" => {
            Ok(ReviewEvent::RequestChanges)
        }
        "comment" => Ok(ReviewEvent::Comment),
        _ => Err(AppError::validation(format!(
            "invalid review event: {}",
            value
        ))),
    }
}

fn parse_merge_method(value: &str) -> Result<MergeMethod, AppError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "merge" => Ok(MergeMethod::Merge),
        "squash" => Ok(MergeMethod::Squash),
        "rebase" => Ok(MergeMethod::Rebase),
        _ => Err(AppError::validation(format!(
            "invalid merge method: {}",
            value
        ))),
    }
}

fn parse_issue_close_reason(value: Option<String>) -> Result<Option<CloseReason>, AppError> {
    match value {
        None => Ok(None),
        Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "completed" => Ok(Some(CloseReason::Completed)),
            "not planned" | "not_planned" | "not-planned" => Ok(Some(CloseReason::NotPlanned)),
            _ => Err(AppError::validation(format!(
                "invalid issue close reason: {}",
                raw
            ))),
        },
    }
}

fn parse_collaborator_permission(value: &str) -> Result<CollaboratorPermission, AppError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "pull" => Ok(CollaboratorPermission::Pull),
        "push" => Ok(CollaboratorPermission::Push),
        "admin" => Ok(CollaboratorPermission::Admin),
        "maintain" => Ok(CollaboratorPermission::Maintain),
        "triage" => Ok(CollaboratorPermission::Triage),
        _ => Err(AppError::validation(format!(
            "invalid collaborator permission: {}",
            value
        ))),
    }
}

fn parse_secret_app(value: Option<String>) -> Result<Option<SecretApp>, AppError> {
    match value {
        None => Ok(None),
        Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "actions" => Ok(Some(SecretApp::Actions)),
            "dependabot" => Ok(Some(SecretApp::Dependabot)),
            "codespaces" => Ok(Some(SecretApp::Codespaces)),
            _ => Err(AppError::validation(format!("invalid secret app: {}", raw))),
        },
    }
}

fn parse_webhook_content_type(value: Option<String>) -> Result<WebhookContentType, AppError> {
    match value
        .as_deref()
        .map(|raw| raw.trim().to_ascii_lowercase())
        .as_deref()
    {
        None => Ok(WebhookContentType::Json),
        Some("json") => Ok(WebhookContentType::Json),
        Some("form") => Ok(WebhookContentType::Form),
        Some(other) => Err(AppError::validation(format!(
            "invalid webhook content_type: {}",
            other
        ))),
    }
}

fn default_limit() -> u16 {
    20
}

#[derive(Debug, Deserialize)]
struct OwnerLimitPayload {
    owner: String,
    #[serde(default = "default_limit")]
    limit: u16,
}

#[derive(Debug, Deserialize)]
struct RepoScopePayload {
    owner: String,
    repo: String,
}

#[derive(Debug, Deserialize)]
struct RepoLimitPayload {
    owner: String,
    repo: String,
    #[serde(default = "default_limit")]
    limit: u16,
}

#[derive(Debug, Deserialize)]
struct RepoCreatePayload {
    owner: String,
    name: String,
    #[serde(default)]
    private: bool,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RepoEditPayload {
    owner: String,
    repo: String,
    description: Option<String>,
    homepage: Option<String>,
    default_branch: Option<String>,
    visibility: Option<String>,
    #[serde(default)]
    add_topics: Vec<String>,
    #[serde(default)]
    remove_topics: Vec<String>,
    replace_topics: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RepoCreateBranchPayload {
    owner: String,
    repo: String,
    branch: String,
    from_branch: String,
}

#[derive(Debug, Deserialize)]
struct RepoDeleteBranchPayload {
    owner: String,
    repo: String,
    branch: String,
}

#[derive(Debug, Deserialize)]
struct RepoCommitsPayload {
    owner: String,
    repo: String,
    branch: Option<String>,
    #[serde(default = "default_limit")]
    limit: u16,
}

#[derive(Debug, Deserialize)]
struct PullRequestNumberPayload {
    owner: String,
    repo: String,
    number: u64,
}

#[derive(Debug, Deserialize)]
struct PullRequestCreatePayload {
    owner: String,
    repo: String,
    title: String,
    head: String,
    base: String,
    body: Option<String>,
    #[serde(default)]
    draft: bool,
}

#[derive(Debug, Deserialize)]
struct PullRequestReviewPayload {
    owner: String,
    repo: String,
    number: u64,
    event: String,
    body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PullRequestEditPayload {
    owner: String,
    repo: String,
    number: u64,
    title: Option<String>,
    body: Option<String>,
    base: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PullRequestClosePayload {
    owner: String,
    repo: String,
    number: u64,
    comment: Option<String>,
    #[serde(default)]
    delete_branch: bool,
}

#[derive(Debug, Deserialize)]
struct PullRequestReopenPayload {
    owner: String,
    repo: String,
    number: u64,
    comment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PullRequestMergePayload {
    owner: String,
    repo: String,
    number: u64,
    #[serde(default = "default_merge_method")]
    method: String,
    #[serde(default)]
    delete_branch: bool,
    #[serde(default)]
    auto: bool,
}

fn default_merge_method() -> String {
    "merge".to_string()
}

#[derive(Debug, Deserialize)]
struct PullRequestCommentPayload {
    owner: String,
    repo: String,
    number: u64,
    body: String,
}

#[derive(Debug, Deserialize)]
struct PullRequestReviewCommentCreatePayload {
    owner: String,
    repo: String,
    number: u64,
    commit_id: String,
    path: String,
    line: u64,
    body: String,
    side: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PullRequestReviewCommentReplyPayload {
    owner: String,
    repo: String,
    number: u64,
    comment_id: u64,
    body: String,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadPayload {
    thread_id: String,
}

#[derive(Debug, Deserialize)]
struct IssueCreatePayload {
    owner: String,
    repo: String,
    title: String,
    body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IssueCommentPayload {
    owner: String,
    repo: String,
    number: u64,
    body: String,
}

#[derive(Debug, Deserialize)]
struct IssueEditPayload {
    owner: String,
    repo: String,
    number: u64,
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    add_assignees: Vec<String>,
    #[serde(default)]
    remove_assignees: Vec<String>,
    #[serde(default)]
    add_labels: Vec<String>,
    #[serde(default)]
    remove_labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct IssueClosePayload {
    owner: String,
    repo: String,
    number: u64,
    comment: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IssueReopenPayload {
    owner: String,
    repo: String,
    number: u64,
    comment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RunPayload {
    owner: String,
    repo: String,
    run_id: u64,
}

#[derive(Debug, Deserialize)]
struct RunRerunPayload {
    owner: String,
    repo: String,
    run_id: u64,
    #[serde(default)]
    failed_only: bool,
}

#[derive(Debug, Deserialize)]
struct ReleaseCreatePayload {
    owner: String,
    repo: String,
    tag: String,
    title: Option<String>,
    notes: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    target: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseDeletePayload {
    owner: String,
    repo: String,
    tag: String,
    #[serde(default)]
    cleanup_tag: bool,
}

#[derive(Debug, Deserialize)]
struct ReleaseEditPayload {
    owner: String,
    repo: String,
    tag: String,
    title: Option<String>,
    notes: Option<String>,
    draft: Option<bool>,
    prerelease: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAssetUploadPayload {
    owner: String,
    repo: String,
    tag: String,
    file_path: String,
    #[serde(default)]
    clobber: bool,
}

#[derive(Debug, Deserialize)]
struct ReleaseAssetDeletePayload {
    owner: String,
    repo: String,
    tag: String,
    asset_name: String,
}

#[derive(Debug, Deserialize)]
struct SettingsAddCollaboratorPayload {
    owner: String,
    repo: String,
    username: String,
    permission: String,
}

#[derive(Debug, Deserialize)]
struct SettingsRemoveCollaboratorPayload {
    owner: String,
    repo: String,
    username: String,
}

#[derive(Debug, Deserialize)]
struct SettingsSecretsListPayload {
    owner: String,
    repo: String,
    app: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SettingsSetSecretPayload {
    owner: String,
    repo: String,
    name: String,
    value: String,
    app: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SettingsDeleteSecretPayload {
    owner: String,
    repo: String,
    name: String,
    app: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SettingsSetVariablePayload {
    owner: String,
    repo: String,
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct SettingsDeleteVariablePayload {
    owner: String,
    repo: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SettingsCreateWebhookPayload {
    owner: String,
    repo: String,
    target_url: String,
    #[serde(default)]
    events: Vec<String>,
    #[serde(default)]
    active: bool,
    content_type: Option<String>,
    secret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SettingsWebhookByIdPayload {
    owner: String,
    repo: String,
    hook_id: u64,
}

#[derive(Debug, Deserialize)]
struct SettingsBranchProtectionGetPayload {
    owner: String,
    repo: String,
    branch: String,
}

#[derive(Debug, Deserialize)]
struct SettingsBranchProtectionUpdatePayload {
    owner: String,
    repo: String,
    branch: String,
    enforce_admins: Option<bool>,
    dismiss_stale_reviews: Option<bool>,
    require_code_owner_reviews: Option<bool>,
    required_approving_review_count: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct SettingsAddDeployKeyPayload {
    owner: String,
    repo: String,
    title: String,
    key: String,
    #[serde(default)]
    read_only: bool,
}

#[derive(Debug, Deserialize)]
struct SettingsDeleteDeployKeyPayload {
    owner: String,
    repo: String,
    key_id: u64,
}

#[derive(Debug, Deserialize)]
struct SettingsDependabotListPayload {
    owner: String,
    repo: String,
    #[serde(default = "default_limit")]
    limit: u16,
}

#[derive(Debug, Deserialize)]
struct ProjectsItemsListPayload {
    owner: String,
    repo: String,
    project_number: u32,
    #[serde(default = "default_limit")]
    limit: u16,
}

#[derive(Debug, Deserialize)]
struct ProjectsAddItemPayload {
    project_id: String,
    content_id: String,
}

#[derive(Debug, Deserialize)]
struct DiscussionsCreatePayload {
    owner: String,
    repo: String,
    category_slug: String,
    title: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct DiscussionsClosePayload {
    discussion_id: String,
}

#[derive(Debug, Deserialize)]
struct DiscussionsAnswerPayload {
    comment_id: String,
}

#[derive(Debug, Deserialize)]
struct WikiUpdatePayload {
    owner: String,
    repo: String,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct PagesConfigurePayload {
    owner: String,
    repo: String,
    branch: String,
    path: String,
    build_type: Option<String>,
    cname: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PagesDeletePayload {
    owner: String,
    repo: String,
}

#[derive(Debug, Deserialize)]
struct RulesetFieldPayload {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct RulesetUpsertPayload {
    owner: String,
    repo: String,
    fields: Vec<RulesetFieldPayload>,
}

#[derive(Debug, Deserialize)]
struct RulesetUpdatePayload {
    owner: String,
    repo: String,
    ruleset_id: u64,
    fields: Vec<RulesetFieldPayload>,
}

#[derive(Debug, Deserialize)]
struct RulesetGetPayload {
    owner: String,
    repo: String,
    ruleset_id: u64,
}

#[derive(Debug, Deserialize)]
struct RulesetDeletePayload {
    owner: String,
    repo: String,
    ruleset_id: u64,
}

#[derive(Debug, Deserialize)]
struct RawArgsPayload {
    #[serde(default)]
    args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::core::executor::RawExecutionOutput;

    #[derive(Default)]
    struct RecordingState {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        responses: Mutex<VecDeque<RawExecutionOutput>>,
    }

    impl RecordingState {
        fn call_count(&self) -> usize {
            self.calls.lock().expect("lock poisoned").len()
        }

        fn last_call(&self) -> Option<(String, Vec<String>)> {
            self.calls.lock().expect("lock poisoned").last().cloned()
        }
    }

    #[derive(Clone)]
    struct RecordingRunner {
        state: Arc<RecordingState>,
    }

    impl RecordingRunner {
        fn new(responses: Vec<RawExecutionOutput>) -> (Self, Arc<RecordingState>) {
            let state = Arc::new(RecordingState {
                calls: Mutex::new(Vec::new()),
                responses: Mutex::new(VecDeque::from(responses)),
            });
            (
                Self {
                    state: Arc::clone(&state),
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
                .state
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

    fn envelope(command_id: &str, payload: Value) -> FrontendCommandEnvelope {
        FrontendCommandEnvelope {
            contract_version: crate::contract::PAYLOAD_CONTRACT_VERSION.to_string(),
            request_id: "req-dispatch-1".to_string(),
            command_id: command_id.to_string(),
            permission: None,
            payload,
        }
    }

    #[test]
    fn supported_commands_match_stable_contract() {
        let mut dispatch_ids =
            FrontendDispatcher::<RecordingRunner>::supported_command_ids().to_vec();
        let mut stable_ids = crate::contract::STABLE_COMMAND_IDS.to_vec();
        dispatch_ids.sort_unstable();
        stable_ids.sort_unstable();
        assert_eq!(dispatch_ids, stable_ids);
    }

    #[test]
    fn dispatches_auth_status() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: "github.com\n  ✓ Logged in to github.com account octocat (keyring)\n  - Active account: true".into(),
            stderr: String::new(),
        }]);

        let dispatcher =
            FrontendDispatcher::new(runner, false).expect("dispatcher should initialize");

        let value = dispatcher
            .execute_envelope(envelope("auth.status", json!({})))
            .expect("dispatch should succeed");

        assert_eq!(value["logged_in"], json!(true));
        assert_eq!(value["account"], json!("octocat"));

        let (_program, args) = state.last_call().expect("command should be called");
        assert_eq!(args[0], "auth");
        assert_eq!(args[1], "status");
    }

    #[test]
    fn dispatches_auth_organizations_list() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"login":"octo-org","name":"Octo Org"}]"#.into(),
            stderr: String::new(),
        }]);

        let dispatcher =
            FrontendDispatcher::new(runner, false).expect("dispatcher should initialize");

        let value = dispatcher
            .execute_envelope(envelope("auth.organizations.list", json!({})))
            .expect("dispatch should succeed");

        assert_eq!(value[0]["login"], json!("octo-org"));

        let (_program, args) = state.last_call().expect("command should be called");
        assert_eq!(args[0], "api");
        assert_eq!(args[1], "user/orgs?per_page=100");
    }

    #[test]
    fn dispatches_pr_comments_list() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: "[]".into(),
            stderr: String::new(),
        }]);

        let dispatcher =
            FrontendDispatcher::new(runner, false).expect("dispatcher should initialize");

        let value = dispatcher
            .execute_envelope(envelope(
                "pr.comments.list",
                json!({"owner":"octocat","repo":"hello","number":1}),
            ))
            .expect("dispatch should succeed");

        assert_eq!(value, json!([]));

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"api".to_string()));
        assert!(
            args.iter()
                .any(|arg| arg.contains("repos/octocat/hello/issues/1/comments"))
        );
    }

    #[test]
    fn defaults_permission_to_viewer_and_denies_admin_action() {
        let (runner, state) = RecordingRunner::new(vec![]);
        let dispatcher =
            FrontendDispatcher::new(runner, false).expect("dispatcher should initialize");

        let err = dispatcher
            .execute_envelope(envelope(
                "settings.collaborators.list",
                json!({"owner":"octocat","repo":"hello"}),
            ))
            .expect_err("viewer should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn returns_validation_error_for_invalid_payload() {
        let (runner, _state) = RecordingRunner::new(vec![]);
        let dispatcher =
            FrontendDispatcher::new(runner, false).expect("dispatcher should initialize");

        let err = dispatcher
            .execute_envelope(envelope("repo.list", json!({"limit": 20})))
            .expect_err("payload without owner should fail");

        assert_eq!(err.code, ErrorCode::ValidationError);
        assert!(err.message.contains("invalid payload"));
    }

    #[test]
    fn executes_internal_raw_command() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: "{\"object\":{\"sha\":\"abc\"}}".into(),
            stderr: String::new(),
        }]);
        let dispatcher =
            FrontendDispatcher::new(runner, false).expect("dispatcher should initialize");

        let value = dispatcher
            .execute_envelope(envelope(
                "repo.branch.ref.get",
                json!({"args":["repos/octocat/hello/git/ref/heads/main"]}),
            ))
            .expect("raw command should succeed");

        assert_eq!(value["exit_code"], json!(0));
        assert_eq!(value["stdout"], json!("{\"object\":{\"sha\":\"abc\"}}"));

        let (_program, args) = state.last_call().expect("command should be called");
        assert_eq!(args[0], "api");
    }
}
