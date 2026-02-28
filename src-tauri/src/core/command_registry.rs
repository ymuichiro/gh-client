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
                "repo.delete",
                "gh",
                &["repo", "delete"],
                CommandSafety::Destructive,
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
        assert_eq!(registry.len(), 3);
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
