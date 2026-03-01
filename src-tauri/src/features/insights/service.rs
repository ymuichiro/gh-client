use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;

use super::dto::{TrafficOverview, parse_traffic_overview};

pub struct InsightsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
}

impl<R: Runner> InsightsService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self { registry, executor }
    }

    pub fn get_views(
        &self,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<TrafficOverview, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let args = vec![format!("repos/{}/{}/traffic/views", owner, repo)];
        let req = self.registry.build_request("insights.views.get", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_traffic_overview(&output.stdout)
    }

    pub fn get_clones(
        &self,
        owner: &str,
        repo: &str,
        trace: &TraceContext,
    ) -> Result<TrafficOverview, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }

        let args = vec![format!("repos/{}/{}/traffic/clones", owner, repo)];
        let req = self.registry.build_request("insights.clones.get", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_traffic_overview(&output.stdout)
    }
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
    }

    impl RecordingState {
        fn last_call(&self) -> Option<(String, Vec<String>)> {
            self.calls.lock().expect("lock poisoned").last().cloned()
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
        TraceContext::new("req-insights-service")
    }

    #[test]
    fn get_views_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"count":10,"uniques":5,"views":[{"timestamp":"2026-03-01T00:00:00Z","count":2,"uniques":1}]}"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = InsightsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let result = service
            .get_views("octocat", "hello", &trace())
            .expect("views should succeed");
        assert_eq!(result.count, 10);

        let (_program, args) = state.last_call().expect("call should be recorded");
        assert_eq!(args[1], "repos/octocat/hello/traffic/views");
    }

    #[test]
    fn get_clones_executes_command() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"count":3,"uniques":2,"clones":[{"timestamp":"2026-03-01T00:00:00Z","count":1,"uniques":1}]}"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = InsightsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let result = service
            .get_clones("octocat", "hello", &trace())
            .expect("clones should succeed");
        assert_eq!(result.count, 3);

        let (_program, args) = state.last_call().expect("call should be recorded");
        assert_eq!(args[1], "repos/octocat/hello/traffic/clones");
    }
}
