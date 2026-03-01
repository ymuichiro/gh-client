use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract::FrontendCommandEnvelope;
use crate::core::error::{AppError, ErrorCode};
use crate::core::executor::Runner;
use crate::frontend::FrontendDispatcher;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontendInvokeError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub fingerprint: String,
    pub request_id: String,
    pub command_id: String,
}

impl FrontendInvokeError {
    pub fn from_app_error(err: AppError, request_id: String, command_id: String) -> Self {
        Self {
            code: error_code_string(err.code).to_string(),
            message: err.message,
            retryable: err.retryable,
            fingerprint: err.fingerprint,
            request_id,
            command_id,
        }
    }
}

pub fn execute_frontend_envelope<R: Runner + Clone>(
    dispatcher: &FrontendDispatcher<R>,
    envelope: FrontendCommandEnvelope,
) -> Result<Value, FrontendInvokeError> {
    let request_id = envelope.request_id.clone();
    let command_id = envelope.command_id.clone();

    dispatcher
        .execute_envelope(envelope)
        .map_err(|err| FrontendInvokeError::from_app_error(err, request_id, command_id))
}

pub fn error_code_string(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::AuthRequired => "auth_required",
        ErrorCode::PermissionDenied => "permission_denied",
        ErrorCode::NotFound => "not_found",
        ErrorCode::ValidationError => "validation_error",
        ErrorCode::RateLimited => "rate_limited",
        ErrorCode::NetworkError => "network_error",
        ErrorCode::UpstreamError => "upstream_error",
        ErrorCode::ExecutionError => "execution_error",
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
        responses: Mutex<VecDeque<RawExecutionOutput>>,
    }

    #[derive(Clone)]
    struct RecordingRunner {
        state: Arc<RecordingState>,
    }

    impl RecordingRunner {
        fn new(responses: Vec<RawExecutionOutput>) -> Self {
            Self {
                state: Arc::new(RecordingState {
                    responses: Mutex::new(VecDeque::from(responses)),
                }),
            }
        }
    }

    impl Runner for RecordingRunner {
        fn run(&self, _program: &str, _args: &[String]) -> io::Result<RawExecutionOutput> {
            let response = self
                .state
                .responses
                .lock()
                .expect("lock poisoned")
                .pop_front()
                .unwrap_or(RawExecutionOutput {
                    exit_code: 0,
                    stdout: "{}".to_string(),
                    stderr: String::new(),
                });
            Ok(response)
        }
    }

    fn envelope(command_id: &str, payload: Value) -> FrontendCommandEnvelope {
        FrontendCommandEnvelope {
            contract_version: crate::contract::PAYLOAD_CONTRACT_VERSION.to_string(),
            request_id: "req-ipc-1".to_string(),
            command_id: command_id.to_string(),
            permission: None,
            payload,
        }
    }

    #[test]
    fn execute_frontend_envelope_success() {
        let dispatcher = FrontendDispatcher::new(
            RecordingRunner::new(vec![RawExecutionOutput {
                exit_code: 0,
                stdout: "github.com\n  ✓ Logged in to github.com account octocat (keyring)\n  - Active account: true".into(),
                stderr: String::new(),
            }]),
            false,
        )
        .expect("dispatcher should initialize");

        let result = execute_frontend_envelope(&dispatcher, envelope("auth.status", Value::Null))
            .expect("execution should succeed");

        assert_eq!(result["logged_in"], Value::Bool(true));
    }

    #[test]
    fn execute_frontend_envelope_returns_validation_error() {
        let dispatcher = FrontendDispatcher::new(RecordingRunner::new(vec![]), false)
            .expect("dispatcher should initialize");

        let mut env = envelope("repo.list", serde_json::json!({"owner":"octocat"}));
        env.contract_version = "invalid-version".to_string();

        let err = execute_frontend_envelope(&dispatcher, env).expect_err("must fail");
        assert_eq!(err.code, "validation_error");
        assert_eq!(err.command_id, "repo.list");
    }

    #[test]
    fn execute_frontend_envelope_maps_auth_errors() {
        let dispatcher = FrontendDispatcher::new(
            RecordingRunner::new(vec![RawExecutionOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: "To get started with GitHub CLI, please run: gh auth login".to_string(),
            }]),
            false,
        )
        .expect("dispatcher should initialize");

        let err = execute_frontend_envelope(
            &dispatcher,
            envelope(
                "repo.list",
                serde_json::json!({"owner":"octocat","limit":20}),
            ),
        )
        .expect_err("must fail");

        assert_eq!(err.code, "auth_required");
        assert_eq!(err.request_id, "req-ipc-1");
    }

    #[test]
    fn execute_frontend_envelope_maps_permission_error() {
        let dispatcher = FrontendDispatcher::new(RecordingRunner::new(vec![]), false)
            .expect("dispatcher should initialize");

        let err = execute_frontend_envelope(
            &dispatcher,
            envelope(
                "settings.collaborators.list",
                serde_json::json!({"owner":"octocat","repo":"hello"}),
            ),
        )
        .expect_err("must fail");

        assert_eq!(err.code, "permission_denied");
    }
}
