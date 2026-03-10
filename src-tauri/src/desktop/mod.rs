use std::env;

use serde_json::Value;

use crate::app_ipc::{FrontendInvokeError, execute_frontend_envelope};
use crate::contract::FrontendCommandEnvelope;
use crate::core::executor::ProcessRunner;
use crate::frontend::FrontendDispatcher;

pub struct AppState {
    dispatcher: FrontendDispatcher<ProcessRunner>,
}

impl AppState {
    fn new() -> Self {
        let safe_test_mode = parse_env_bool("SAFE_TEST_MODE");
        let dispatcher = FrontendDispatcher::new(ProcessRunner, safe_test_mode)
            .expect("failed to initialize frontend dispatcher");
        Self { dispatcher }
    }
}

#[tauri::command]
pub fn frontend_execute(
    state: tauri::State<'_, AppState>,
    envelope: FrontendCommandEnvelope,
) -> Result<Value, FrontendInvokeError> {
    execute_frontend_envelope(&state.dispatcher, envelope)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![frontend_execute])
        .run(tauri::generate_context!())
        .expect("error while running gh-client desktop");
}

fn parse_env_bool(key: &str) -> bool {
    match env::var(key) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        ),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_bool_true_values() {
        unsafe {
            env::set_var("TEST_BOOL", "1");
        }
        assert!(parse_env_bool("TEST_BOOL"));

        unsafe {
            env::set_var("TEST_BOOL", "true");
        }
        assert!(parse_env_bool("TEST_BOOL"));

        unsafe {
            env::set_var("TEST_BOOL", "yes");
        }
        assert!(parse_env_bool("TEST_BOOL"));
    }

    #[test]
    fn parse_env_bool_false_values() {
        unsafe {
            env::set_var("TEST_BOOL", "0");
        }
        assert!(!parse_env_bool("TEST_BOOL"));

        unsafe {
            env::remove_var("TEST_BOOL");
        }
        assert!(!parse_env_bool("TEST_BOOL"));
    }
}
