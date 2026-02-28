use std::io;
use std::process::Command;
use std::time::Instant;

use crate::core::command_registry::{CommandRequest, CommandSafety};
use crate::core::error::AppError;
use crate::core::observability::{AuditEvent, TraceContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait Runner: Send + Sync {
    fn run(&self, program: &str, args: &[String]) -> io::Result<RawExecutionOutput>;
}

#[derive(Debug, Default)]
pub struct ProcessRunner;

impl Runner for ProcessRunner {
    fn run(&self, program: &str, args: &[String]) -> io::Result<RawExecutionOutput> {
        let output = Command::new(program).args(args).output()?;
        Ok(RawExecutionOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutput {
    pub command_id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub noop: bool,
}

pub struct CommandExecutor<R: Runner> {
    runner: R,
    safe_test_mode: bool,
}

impl<R: Runner> CommandExecutor<R> {
    pub fn new(runner: R, safe_test_mode: bool) -> Self {
        Self {
            runner,
            safe_test_mode,
        }
    }

    pub fn execute(
        &self,
        request: &CommandRequest,
        trace: &TraceContext,
    ) -> Result<(ExecutionOutput, AuditEvent), AppError> {
        let started = Instant::now();

        if self.safe_test_mode && request.safety == CommandSafety::Destructive {
            let output = ExecutionOutput {
                command_id: request.command_id.clone(),
                exit_code: 0,
                stdout: String::new(),
                stderr: "SAFE_TEST_MODE prevented destructive execution".to_string(),
                duration_ms: started.elapsed().as_millis(),
                noop: true,
            };

            let event = AuditEvent {
                trace_id: trace.trace_id.clone(),
                request_id: trace.request_id.clone(),
                command_id: request.command_id.clone(),
                duration_ms: output.duration_ms,
                exit_code: 0,
                noop: true,
            };

            return Ok((output, event));
        }

        let raw = self
            .runner
            .run(&request.program, &request.args)
            .map_err(|err| {
                AppError::execution(format!(
                    "failed to execute `{}`: {}",
                    request.command_id, err
                ))
            })?;

        let output = ExecutionOutput {
            command_id: request.command_id.clone(),
            exit_code: raw.exit_code,
            stdout: raw.stdout,
            stderr: raw.stderr,
            duration_ms: started.elapsed().as_millis(),
            noop: false,
        };

        let event = AuditEvent {
            trace_id: trace.trace_id.clone(),
            request_id: trace.request_id.clone(),
            command_id: request.command_id.clone(),
            duration_ms: output.duration_ms,
            exit_code: output.exit_code,
            noop: false,
        };

        if output.exit_code != 0 {
            return Err(AppError::execution(format!(
                "command `{}` failed with exit code {} (stderr: {})",
                request.command_id,
                output.exit_code,
                output.stderr.trim()
            )));
        }

        Ok((output, event))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct SuccessRunner;

    impl Runner for SuccessRunner {
        fn run(&self, _program: &str, _args: &[String]) -> io::Result<RawExecutionOutput> {
            Ok(RawExecutionOutput {
                exit_code: 0,
                stdout: "ok".into(),
                stderr: String::new(),
            })
        }
    }

    struct FailingRunner;

    impl Runner for FailingRunner {
        fn run(&self, _program: &str, _args: &[String]) -> io::Result<RawExecutionOutput> {
            Ok(RawExecutionOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: "boom".into(),
            })
        }
    }

    struct ErrorRunner;

    impl Runner for ErrorRunner {
        fn run(&self, _program: &str, _args: &[String]) -> io::Result<RawExecutionOutput> {
            Err(io::Error::other("spawn failure"))
        }
    }

    struct CountingRunner {
        calls: AtomicUsize,
    }

    impl CountingRunner {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl Runner for CountingRunner {
        fn run(&self, _program: &str, _args: &[String]) -> io::Result<RawExecutionOutput> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(RawExecutionOutput {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    fn trace() -> TraceContext {
        TraceContext::new("req-test")
    }

    fn destructive_request() -> CommandRequest {
        CommandRequest {
            command_id: "repo.delete".into(),
            program: "gh".into(),
            args: vec!["repo".into(), "delete".into()],
            safety: CommandSafety::Destructive,
        }
    }

    fn non_destructive_request() -> CommandRequest {
        CommandRequest {
            command_id: "repo.list".into(),
            program: "gh".into(),
            args: vec!["repo".into(), "list".into()],
            safety: CommandSafety::NonDestructive,
        }
    }

    #[test]
    fn safe_test_mode_skips_destructive_command() {
        let runner = CountingRunner::new();
        let executor = CommandExecutor::new(runner, true);

        let (output, audit) = executor
            .execute(&destructive_request(), &trace())
            .expect("safe test mode should return success");

        assert!(output.noop);
        assert_eq!(audit.command_id, "repo.delete");
        assert!(audit.noop);
    }

    #[test]
    fn non_destructive_command_executes_even_in_safe_mode() {
        let runner = CountingRunner::new();
        let executor = CommandExecutor::new(runner, true);

        let (output, _audit) = executor
            .execute(&non_destructive_request(), &trace())
            .expect("non destructive should execute");

        assert!(!output.noop);
        assert_eq!(executor.runner.call_count(), 1);
    }

    #[test]
    fn returns_error_on_non_zero_exit_code() {
        let executor = CommandExecutor::new(FailingRunner, false);
        let err = executor
            .execute(&non_destructive_request(), &trace())
            .expect_err("non-zero exit must fail");

        assert!(err.message.contains("exit code 1"));
    }

    #[test]
    fn returns_error_when_runner_fails() {
        let executor = CommandExecutor::new(ErrorRunner, false);
        let err = executor
            .execute(&non_destructive_request(), &trace())
            .expect_err("spawn failure must fail");

        assert!(err.message.contains("failed to execute"));
    }

    #[test]
    fn successful_execution_contains_audit_event() {
        let executor = CommandExecutor::new(SuccessRunner, false);
        let (output, audit) = executor
            .execute(&non_destructive_request(), &trace())
            .expect("success expected");

        assert_eq!(output.stdout, "ok");
        assert_eq!(audit.command_id, "repo.list");
        assert_eq!(audit.exit_code, 0);
    }
}
