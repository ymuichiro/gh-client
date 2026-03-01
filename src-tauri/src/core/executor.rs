use std::io;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::command_registry::{CommandRequest, CommandSafety};
use crate::core::error::{AppError, ErrorCode};
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutorOptions {
    pub max_retries: u8,
    pub initial_backoff_ms: u64,
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_backoff_ms: 150,
        }
    }
}

pub struct CommandExecutor<R: Runner> {
    runner: R,
    safe_test_mode: bool,
    options: ExecutorOptions,
}

impl<R: Runner> CommandExecutor<R> {
    pub fn new(runner: R, safe_test_mode: bool) -> Self {
        Self {
            runner,
            safe_test_mode,
            options: ExecutorOptions::default(),
        }
    }

    pub fn with_options(runner: R, safe_test_mode: bool, options: ExecutorOptions) -> Self {
        Self {
            runner,
            safe_test_mode,
            options,
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

        let mut last_error: Option<AppError> = None;

        for attempt in 0..=self.options.max_retries {
            match self.runner.run(&request.program, &request.args) {
                Ok(raw) => {
                    let output = ExecutionOutput {
                        command_id: request.command_id.clone(),
                        exit_code: raw.exit_code,
                        stdout: raw.stdout,
                        stderr: raw.stderr,
                        duration_ms: started.elapsed().as_millis(),
                        noop: false,
                    };

                    if output.exit_code == 0 {
                        let event = AuditEvent {
                            trace_id: trace.trace_id.clone(),
                            request_id: trace.request_id.clone(),
                            command_id: request.command_id.clone(),
                            duration_ms: output.duration_ms,
                            exit_code: output.exit_code,
                            noop: false,
                        };
                        return Ok((output, event));
                    }

                    let err = classify_process_error(
                        &request.command_id,
                        output.exit_code,
                        &output.stderr,
                        &output.stdout,
                    );

                    if should_retry(&err, attempt, self.options.max_retries) {
                        thread::sleep(backoff_delay(self.options.initial_backoff_ms, attempt));
                        last_error = Some(err);
                        continue;
                    }

                    return Err(err);
                }
                Err(err) => {
                    let app_err = classify_spawn_error(&request.command_id, &err);

                    if should_retry(&app_err, attempt, self.options.max_retries) {
                        thread::sleep(backoff_delay(self.options.initial_backoff_ms, attempt));
                        last_error = Some(app_err);
                        continue;
                    }

                    return Err(app_err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AppError::execution(format!(
                "command `{}` failed for an unknown reason",
                request.command_id
            ))
        }))
    }
}

fn should_retry(err: &AppError, attempt: u8, max_retries: u8) -> bool {
    err.retryable && attempt < max_retries
}

fn backoff_delay(initial_backoff_ms: u64, attempt: u8) -> Duration {
    let factor = 2u64.saturating_pow(attempt as u32);
    Duration::from_millis(initial_backoff_ms.saturating_mul(factor))
}

fn classify_spawn_error(command_id: &str, err: &io::Error) -> AppError {
    let message = format!("failed to execute `{}`: {}", command_id, err);
    AppError::new(ErrorCode::ExecutionError, message, true)
}

fn classify_process_error(
    command_id: &str,
    exit_code: i32,
    stderr: &str,
    stdout: &str,
) -> AppError {
    let stderr_trimmed = stderr.trim();
    let normalized = normalize_error_text(stderr, stdout);

    let message = format!(
        "command `{}` failed with exit code {} (stderr: {})",
        command_id, exit_code, stderr_trimmed
    );

    if is_rate_limited(&normalized) {
        return AppError::new(ErrorCode::RateLimited, message, true);
    }

    if is_auth_error(&normalized) {
        return AppError::new(ErrorCode::AuthRequired, message, false);
    }

    if is_permission_error(&normalized) {
        return AppError::new(ErrorCode::PermissionDenied, message, false);
    }

    if is_not_found_error(&normalized) {
        return AppError::new(ErrorCode::NotFound, message, false);
    }

    if is_network_error(&normalized) {
        return AppError::new(ErrorCode::NetworkError, message, true);
    }

    if is_upstream_error(&normalized) {
        return AppError::new(ErrorCode::UpstreamError, message, true);
    }

    AppError::new(ErrorCode::ExecutionError, message, false)
}

fn normalize_error_text(stderr: &str, stdout: &str) -> String {
    let mut joined = String::with_capacity(stderr.len() + stdout.len() + 1);
    joined.push_str(stderr);
    joined.push('\n');
    joined.push_str(stdout);
    joined.to_ascii_lowercase()
}

fn is_auth_error(text: &str) -> bool {
    [
        "authentication required",
        "requires authentication",
        "gh auth login",
        "bad credentials",
        "http 401",
        "must authenticate",
        "not logged into",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn is_permission_error(text: &str) -> bool {
    [
        "permission denied",
        "resource not accessible",
        "forbidden",
        "http 403",
        "must have admin rights",
        "required scopes",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn is_not_found_error(text: &str) -> bool {
    ["not found", "http 404", "unknown repository"]
        .iter()
        .any(|needle| text.contains(needle))
}

fn is_rate_limited(text: &str) -> bool {
    [
        "rate limit exceeded",
        "secondary rate limit",
        "api rate limit exceeded",
        "http 429",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn is_network_error(text: &str) -> bool {
    [
        "connection reset",
        "connection refused",
        "connection timed out",
        "tls handshake timeout",
        "i/o timeout",
        "temporary failure",
        "no such host",
        "dial tcp",
        "network is unreachable",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn is_upstream_error(text: &str) -> bool {
    [
        "http 500",
        "http 502",
        "http 503",
        "http 504",
        "service unavailable",
        "bad gateway",
        "gateway timeout",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    enum Step {
        Output(RawExecutionOutput),
        IoError(&'static str),
    }

    struct SequenceRunner {
        calls: AtomicUsize,
        steps: Mutex<VecDeque<Step>>,
    }

    impl SequenceRunner {
        fn new(steps: Vec<Step>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                steps: Mutex::new(VecDeque::from(steps)),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl Runner for SequenceRunner {
        fn run(&self, _program: &str, _args: &[String]) -> io::Result<RawExecutionOutput> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            let step = self
                .steps
                .lock()
                .expect("lock poisoned")
                .pop_front()
                .unwrap_or(Step::Output(RawExecutionOutput {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                }));

            match step {
                Step::Output(output) => Ok(output),
                Step::IoError(message) => Err(io::Error::other(message)),
            }
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
        let runner = SequenceRunner::new(vec![]);
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
        let runner = SequenceRunner::new(vec![Step::Output(RawExecutionOutput {
            exit_code: 0,
            stdout: String::from("ok"),
            stderr: String::new(),
        })]);
        let executor = CommandExecutor::new(runner, true);

        let (output, _audit) = executor
            .execute(&non_destructive_request(), &trace())
            .expect("non destructive should execute");

        assert!(!output.noop);
        assert_eq!(executor.runner.call_count(), 1);
    }

    #[test]
    fn classifies_rate_limit_errors() {
        let runner = SequenceRunner::new(vec![Step::Output(RawExecutionOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: String::from("API rate limit exceeded for user"),
        })]);

        let executor = CommandExecutor::with_options(
            runner,
            false,
            ExecutorOptions {
                max_retries: 0,
                initial_backoff_ms: 1,
            },
        );

        let err = executor
            .execute(&non_destructive_request(), &trace())
            .expect_err("rate limited command must fail");

        assert_eq!(err.code, ErrorCode::RateLimited);
        assert!(err.retryable);
    }

    #[test]
    fn classifies_auth_errors() {
        let runner = SequenceRunner::new(vec![Step::Output(RawExecutionOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: String::from("run gh auth login to authenticate"),
        })]);

        let executor = CommandExecutor::with_options(
            runner,
            false,
            ExecutorOptions {
                max_retries: 0,
                initial_backoff_ms: 1,
            },
        );

        let err = executor
            .execute(&non_destructive_request(), &trace())
            .expect_err("auth command must fail");

        assert_eq!(err.code, ErrorCode::AuthRequired);
        assert!(!err.retryable);
    }

    #[test]
    fn retries_on_network_error_then_succeeds() {
        let runner = SequenceRunner::new(vec![
            Step::Output(RawExecutionOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: String::from("connection reset by peer"),
            }),
            Step::Output(RawExecutionOutput {
                exit_code: 0,
                stdout: String::from("ok"),
                stderr: String::new(),
            }),
        ]);

        let executor = CommandExecutor::with_options(
            runner,
            false,
            ExecutorOptions {
                max_retries: 1,
                initial_backoff_ms: 1,
            },
        );

        let (output, _audit) = executor
            .execute(&non_destructive_request(), &trace())
            .expect("network retry should succeed");

        assert_eq!(output.stdout, "ok");
        assert_eq!(executor.runner.call_count(), 2);
    }

    #[test]
    fn does_not_retry_on_permission_denied() {
        let runner = SequenceRunner::new(vec![
            Step::Output(RawExecutionOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: String::from("HTTP 403 Forbidden"),
            }),
            Step::Output(RawExecutionOutput {
                exit_code: 0,
                stdout: String::from("ok"),
                stderr: String::new(),
            }),
        ]);

        let executor = CommandExecutor::with_options(
            runner,
            false,
            ExecutorOptions {
                max_retries: 2,
                initial_backoff_ms: 1,
            },
        );

        let err = executor
            .execute(&non_destructive_request(), &trace())
            .expect_err("permission denied should fail immediately");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(executor.runner.call_count(), 1);
    }

    #[test]
    fn retries_on_spawn_error_then_succeeds() {
        let runner = SequenceRunner::new(vec![
            Step::IoError("temporary spawn failure"),
            Step::Output(RawExecutionOutput {
                exit_code: 0,
                stdout: String::from("ok"),
                stderr: String::new(),
            }),
        ]);

        let executor = CommandExecutor::with_options(
            runner,
            false,
            ExecutorOptions {
                max_retries: 1,
                initial_backoff_ms: 1,
            },
        );

        let (output, _audit) = executor
            .execute(&non_destructive_request(), &trace())
            .expect("spawn retry should succeed");

        assert_eq!(output.stdout, "ok");
        assert_eq!(executor.runner.call_count(), 2);
    }

    #[test]
    fn returns_error_on_non_zero_exit_code() {
        let runner = SequenceRunner::new(vec![Step::Output(RawExecutionOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "boom".into(),
        })]);
        let executor = CommandExecutor::with_options(
            runner,
            false,
            ExecutorOptions {
                max_retries: 0,
                initial_backoff_ms: 1,
            },
        );

        let err = executor
            .execute(&non_destructive_request(), &trace())
            .expect_err("non-zero exit must fail");

        assert_eq!(err.code, ErrorCode::ExecutionError);
        assert!(err.message.contains("exit code 1"));
    }

    #[test]
    fn successful_execution_contains_audit_event() {
        let runner = SequenceRunner::new(vec![Step::Output(RawExecutionOutput {
            exit_code: 0,
            stdout: "ok".into(),
            stderr: String::new(),
        })]);
        let executor = CommandExecutor::new(runner, false);
        let (output, audit) = executor
            .execute(&non_destructive_request(), &trace())
            .expect("success expected");

        assert_eq!(output.stdout, "ok");
        assert_eq!(audit.command_id, "repo.list");
        assert_eq!(audit.exit_code, 0);
    }
}
