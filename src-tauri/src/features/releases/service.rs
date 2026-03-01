use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{
    ReleaseCreated, ReleaseSummary, parse_release_created_output, parse_release_summaries,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateReleaseInput {
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub target: Option<String>,
}

impl CreateReleaseInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.tag.trim().is_empty() {
            return Err(AppError::validation("tag is required"));
        }

        if let Some(title) = self.title.as_ref() {
            if title.trim().is_empty() {
                return Err(AppError::validation(
                    "title must not be empty when provided",
                ));
            }
        }

        if let Some(notes) = self.notes.as_ref() {
            if notes.trim().is_empty() {
                return Err(AppError::validation(
                    "notes must not be empty when provided",
                ));
            }
        }

        if let Some(target) = self.target.as_ref() {
            if target.trim().is_empty() {
                return Err(AppError::validation(
                    "target must not be empty when provided",
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteReleaseInput {
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub cleanup_tag: bool,
}

impl DeleteReleaseInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.tag.trim().is_empty() {
            return Err(AppError::validation("tag is required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditReleaseInput {
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub draft: Option<bool>,
    pub prerelease: Option<bool>,
}

impl EditReleaseInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.tag.trim().is_empty() {
            return Err(AppError::validation("tag is required"));
        }

        if self.title.is_none()
            && self.notes.is_none()
            && self.draft.is_none()
            && self.prerelease.is_none()
        {
            return Err(AppError::validation(
                "at least one editable field must be provided",
            ));
        }

        if self
            .title
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "title must not be empty when provided",
            ));
        }

        if self
            .notes
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::validation(
                "notes must not be empty when provided",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadReleaseAssetInput {
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub file_path: String,
    pub clobber: bool,
}

impl UploadReleaseAssetInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.tag.trim().is_empty() {
            return Err(AppError::validation("tag is required"));
        }
        if self.file_path.trim().is_empty() {
            return Err(AppError::validation("file_path is required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteReleaseAssetInput {
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub asset_name: String,
}

impl DeleteReleaseAssetInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.tag.trim().is_empty() {
            return Err(AppError::validation("tag is required"));
        }
        if self.asset_name.trim().is_empty() {
            return Err(AppError::validation("asset_name is required"));
        }

        Ok(())
    }
}

pub struct ReleasesService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> ReleasesService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list(
        &self,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<ReleaseSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![
            "--repo".to_string(),
            format!("{}/{}", owner, repo),
            "--limit".to_string(),
            limit.to_string(),
        ];
        let req = self.registry.build_request("release.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_release_summaries(&output.stdout)
    }

    pub fn create(
        &self,
        permission: RepoPermission,
        input: &CreateReleaseInput,
        trace: &TraceContext,
    ) -> Result<ReleaseCreated, AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "release.create")?;
        input.validate()?;

        let mut args = vec![
            input.tag.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        if let Some(title) = input.title.as_ref() {
            args.push("--title".to_string());
            args.push(title.clone());
        }

        if let Some(notes) = input.notes.as_ref() {
            args.push("--notes".to_string());
            args.push(notes.clone());
        }

        if let Some(target) = input.target.as_ref() {
            args.push("--target".to_string());
            args.push(target.clone());
        }

        if input.draft {
            args.push("--draft".to_string());
        }

        if input.prerelease {
            args.push("--prerelease".to_string());
        }

        let req = self.registry.build_request("release.create", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_release_created_output(&input.tag, &output.stdout)
    }

    pub fn delete(
        &self,
        permission: RepoPermission,
        input: &DeleteReleaseInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "release.delete")?;
        input.validate()?;

        let mut args = vec![
            input.tag.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            "--yes".to_string(),
        ];

        if input.cleanup_tag {
            args.push("--cleanup-tag".to_string());
        }

        let req = self.registry.build_request("release.delete", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn edit(
        &self,
        permission: RepoPermission,
        input: &EditReleaseInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "release.edit")?;
        input.validate()?;

        let mut args = vec![
            input.tag.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];

        if let Some(title) = input.title.as_ref() {
            args.push("--title".to_string());
            args.push(title.clone());
        }

        if let Some(notes) = input.notes.as_ref() {
            args.push("--notes".to_string());
            args.push(notes.clone());
        }

        if let Some(draft) = input.draft {
            args.push(format!("--draft={}", draft));
        }

        if let Some(prerelease) = input.prerelease {
            args.push(format!("--prerelease={}", prerelease));
        }

        let req = self.registry.build_request("release.edit", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn upload_asset(
        &self,
        permission: RepoPermission,
        input: &UploadReleaseAssetInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "release.asset.upload")?;
        input.validate()?;

        let mut args = vec![
            input.tag.clone(),
            input.file_path.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
        ];
        if input.clobber {
            args.push("--clobber".to_string());
        }

        let req = self.registry.build_request("release.asset.upload", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn delete_asset(
        &self,
        permission: RepoPermission,
        input: &DeleteReleaseAssetInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Admin, permission, "release.asset.delete")?;
        input.validate()?;

        let args = vec![
            input.tag.clone(),
            input.asset_name.clone(),
            "--repo".to_string(),
            format!("{}/{}", input.owner, input.repo),
            "--yes".to_string(),
        ];

        let req = self.registry.build_request("release.asset.delete", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
    }

    struct RecordingRunner {
        state: Arc<RecordingState>,
        response: RawExecutionOutput,
    }

    impl RecordingRunner {
        fn new(response: RawExecutionOutput) -> (Self, Arc<RecordingState>) {
            let state = Arc::new(RecordingState::default());
            (
                Self {
                    state: Arc::clone(&state),
                    response,
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
            Ok(self.response.clone())
        }
    }

    fn trace() -> TraceContext {
        TraceContext::new("req-releases-service")
    }

    #[test]
    fn list_executes_release_list_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"[{"tagName":"v1.0.0","name":"v1","isDraft":false,"isPrerelease":false,"publishedAt":"2026-01-01T00:00:00Z","createdAt":"2026-01-01T00:00:00Z","isLatest":true}]"#.into(),
            stderr: String::new(),
        };
        let (runner, state) = RecordingRunner::new(output);

        let service = ReleasesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let releases = service
            .list("octocat", "hello", 20, &trace())
            .expect("list should succeed");
        assert_eq!(releases.len(), 1);

        let (program, args) = state.last_call().expect("command should be called");
        assert_eq!(program, "gh");
        assert!(args.contains(&"release".to_string()));
        assert!(args.contains(&"list".to_string()));
    }

    #[test]
    fn create_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ReleasesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreateReleaseInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            tag: "v1.0.0".into(),
            title: Some("v1".into()),
            notes: Some("notes".into()),
            draft: false,
            prerelease: false,
            target: None,
        };

        let err = service
            .create(RepoPermission::Viewer, &input, &trace())
            .expect_err("viewer should be denied");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn delete_is_noop_in_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ReleasesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeleteReleaseInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            tag: "v1.0.0".into(),
            cleanup_tag: false,
        };

        service
            .delete(RepoPermission::Admin, &input, &trace())
            .expect("delete should no-op in safe mode");

        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn delete_requires_admin_permission() {
        let (runner, _state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ReleasesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeleteReleaseInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            tag: "v1.0.0".into(),
            cleanup_tag: true,
        };

        let err = service
            .delete(RepoPermission::Write, &input, &trace())
            .expect_err("write should not delete release");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
    }

    #[test]
    fn edit_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ReleasesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = EditReleaseInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            tag: "v1.0.0".into(),
            title: Some("new title".into()),
            notes: None,
            draft: Some(false),
            prerelease: None,
        };

        service
            .edit(RepoPermission::Write, &input, &trace())
            .expect("edit should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--title".to_string()));
        assert!(args.contains(&"--draft=false".to_string()));
    }

    #[test]
    fn upload_asset_executes_command() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ReleasesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = UploadReleaseAssetInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            tag: "v1.0.0".into(),
            file_path: "/tmp/asset.zip".into(),
            clobber: true,
        };

        service
            .upload_asset(RepoPermission::Write, &input, &trace())
            .expect("upload should succeed");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"--clobber".to_string()));
    }

    #[test]
    fn delete_asset_is_noop_in_safe_test_mode() {
        let (runner, state) = RecordingRunner::new(RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        let service = ReleasesService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, true),
        );

        let input = DeleteReleaseAssetInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            tag: "v1.0.0".into(),
            asset_name: "asset.zip".into(),
        };

        service
            .delete_asset(RepoPermission::Admin, &input, &trace())
            .expect("delete asset should no-op");

        assert_eq!(state.call_count(), 0);
    }
}
