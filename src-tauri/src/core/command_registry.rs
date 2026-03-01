use std::collections::HashMap;

use crate::core::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSafety {
    NonDestructive,
    Destructive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub id: &'static str,
    pub program: &'static str,
    pub base_args: &'static [&'static str],
    pub safety: CommandSafety,
}

impl CommandSpec {
    pub const fn new(
        id: &'static str,
        program: &'static str,
        base_args: &'static [&'static str],
        safety: CommandSafety,
    ) -> Self {
        Self {
            id,
            program,
            base_args,
            safety,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRequest {
    pub command_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub safety: CommandSafety,
}

#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    specs: HashMap<&'static str, CommandSpec>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry
            .register(CommandSpec::new(
                "repo.list",
                "gh",
                &[
                    "repo",
                    "list",
                    "--json",
                    "name,nameWithOwner,description,url,isPrivate,viewerPermission",
                ],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.create",
                "gh",
                &["repo", "create"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.edit",
                "gh",
                &["repo", "edit"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.topics.replace",
                "gh",
                &["api", "--method", "PUT"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.branches.list",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.branch.ref.get",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.branch.create",
                "gh",
                &["api", "--method", "POST"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.branch.delete",
                "gh",
                &["api", "--method", "DELETE"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.commits.list",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "repo.delete",
                "gh",
                &["repo", "delete"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "pr.list",
                "gh",
                &[
                    "pr",
                    "list",
                    "--json",
                    "number,title,state,url,isDraft,author,headRefName,baseRefName",
                ],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "pr.create",
                "gh",
                &["api", "--method", "POST"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "pr.review",
                "gh",
                &["pr", "review"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "pr.edit",
                "gh",
                &["pr", "edit"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "pr.close",
                "gh",
                &["pr", "close"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "pr.reopen",
                "gh",
                &["pr", "reopen"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "pr.merge",
                "gh",
                &["pr", "merge"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "issue.list",
                "gh",
                &["issue", "list", "--json", "number,title,state,url,author"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "issue.create",
                "gh",
                &["issue", "create"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "issue.comment",
                "gh",
                &["issue", "comment"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "issue.edit",
                "gh",
                &["issue", "edit"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "issue.close",
                "gh",
                &["issue", "close"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "issue.reopen",
                "gh",
                &["issue", "reopen"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "workflow.list",
                "gh",
                &["workflow", "list", "--json", "id,name,path,state"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "run.list",
                "gh",
                &[
                    "run",
                    "list",
                    "--json",
                    "databaseId,workflowName,headBranch,status,conclusion,url,displayTitle",
                ],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "run.rerun",
                "gh",
                &["run", "rerun"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "run.view",
                "gh",
                &[
                    "run",
                    "view",
                    "--json",
                    "databaseId,status,conclusion,url,workflowName,jobs",
                ],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "run.logs",
                "gh",
                &["run", "view", "--log"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "run.cancel",
                "gh",
                &["run", "cancel"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "release.list",
                "gh",
                &[
                    "release",
                    "list",
                    "--json",
                    "tagName,name,isDraft,isPrerelease,publishedAt,createdAt,isLatest",
                ],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "release.create",
                "gh",
                &["release", "create"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "release.edit",
                "gh",
                &["release", "edit"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "release.asset.upload",
                "gh",
                &["release", "upload"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "release.asset.delete",
                "gh",
                &["release", "delete-asset"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "release.delete",
                "gh",
                &["release", "delete"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.collaborators.list",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.collaborators.add",
                "gh",
                &["api", "--method", "PUT"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.collaborators.remove",
                "gh",
                &["api", "--method", "DELETE"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.secrets.list",
                "gh",
                &["secret", "list", "--json", "name,updatedAt,visibility"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.secrets.set",
                "gh",
                &["secret", "set"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.secrets.delete",
                "gh",
                &["secret", "delete"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.variables.list",
                "gh",
                &[
                    "variable",
                    "list",
                    "--json",
                    "name,value,updatedAt,createdAt,visibility",
                ],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.variables.set",
                "gh",
                &["variable", "set"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.variables.delete",
                "gh",
                &["variable", "delete"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.webhooks.list",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.webhooks.create",
                "gh",
                &["api", "--method", "POST"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.webhooks.ping",
                "gh",
                &["api", "--method", "POST"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.webhooks.delete",
                "gh",
                &["api", "--method", "DELETE"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.branch_protection.get",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.branch_protection.update",
                "gh",
                &["api", "--method", "PUT"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.deploy_keys.list",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.deploy_keys.add",
                "gh",
                &["api", "--method", "POST"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.deploy_keys.delete",
                "gh",
                &["api", "--method", "DELETE"],
                CommandSafety::Destructive,
            ))
            .expect("default command should register");

        registry
            .register(CommandSpec::new(
                "settings.dependabot_alerts.list",
                "gh",
                &["api"],
                CommandSafety::NonDestructive,
            ))
            .expect("default command should register");

        registry
    }

    pub fn register(&mut self, spec: CommandSpec) -> Result<(), AppError> {
        if self.specs.contains_key(spec.id) {
            return Err(AppError::validation(format!(
                "command `{}` is already registered",
                spec.id
            )));
        }

        self.specs.insert(spec.id, spec);
        Ok(())
    }

    pub fn build_request(
        &self,
        command_id: &str,
        dynamic_args: &[String],
    ) -> Result<CommandRequest, AppError> {
        let spec = self.specs.get(command_id).ok_or_else(|| {
            AppError::validation(format!("command `{}` is not registered", command_id))
        })?;

        for arg in dynamic_args {
            if arg.contains('\0') {
                return Err(AppError::validation(format!(
                    "command `{}` contains a NUL byte in args",
                    command_id
                )));
            }
        }

        let mut args = spec
            .base_args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        args.extend(dynamic_args.iter().cloned());

        Ok(CommandRequest {
            command_id: command_id.to_string(),
            program: spec.program.to_string(),
            args,
            safety: spec.safety,
        })
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_contains_expected_commands() {
        let registry = CommandRegistry::with_defaults();
        assert_eq!(registry.len(), 54);
    }

    #[test]
    fn rejects_duplicate_registration() {
        let mut registry = CommandRegistry::new();
        registry
            .register(CommandSpec::new(
                "repo.list",
                "gh",
                &["repo", "list"],
                CommandSafety::NonDestructive,
            ))
            .expect("initial registration should succeed");

        let second = registry.register(CommandSpec::new(
            "repo.list",
            "gh",
            &["repo", "list"],
            CommandSafety::NonDestructive,
        ));

        assert!(second.is_err());
    }

    #[test]
    fn builds_request_with_static_and_dynamic_args() {
        let registry = CommandRegistry::with_defaults();
        let req = registry
            .build_request(
                "repo.list",
                &["octocat".into(), "--limit".into(), "10".into()],
            )
            .expect("request should build");

        assert_eq!(req.program, "gh");
        assert!(req.args.contains(&"repo".to_string()));
        assert!(req.args.contains(&"octocat".to_string()));
        assert_eq!(req.safety, CommandSafety::NonDestructive);
    }

    #[test]
    fn rejects_unknown_command() {
        let registry = CommandRegistry::with_defaults();
        let err = registry
            .build_request("unknown.command", &[])
            .expect_err("unknown command must fail");
        assert!(err.message.contains("not registered"));
    }

    #[test]
    fn rejects_nul_in_argument() {
        let registry = CommandRegistry::with_defaults();
        let err = registry
            .build_request("repo.list", &["bad\0arg".into()])
            .expect_err("nul byte should fail validation");
        assert!(err.message.contains("NUL"));
    }
}
