use std::env;
use std::time::Duration;

use serde_json::Value;
use tauri::Manager;

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

#[tauri::command]
pub fn close_splashscreen(app: tauri::AppHandle) -> Result<(), String> {
    reveal_main_window(&app)
}

fn reveal_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    main_window
        .show()
        .map_err(|error| format!("failed to show main window: {error}"))?;
    if let Err(error) = main_window.set_focus() {
        eprintln!("failed to focus main window: {error}");
    }

    if let Some(splash_window) = app.get_webview_window("splashscreen") {
        if let Err(error) = splash_window.close() {
            eprintln!("failed to close splashscreen window: {error}");
        }
    }

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .setup(|app| {
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(12));
                if let Some(splash_window) = app_handle.get_webview_window("splashscreen") {
                    if splash_window.is_visible().unwrap_or(false) {
                        if let Err(error) = reveal_main_window(&app_handle) {
                            eprintln!("startup fallback failed to reveal main window: {error}");
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_execute,
            close_splashscreen
        ])
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
        const TEST_KEY: &str = "TEST_BOOL_TRUE";
        unsafe {
            env::set_var(TEST_KEY, "1");
        }
        assert!(parse_env_bool(TEST_KEY));

        unsafe {
            env::set_var(TEST_KEY, "true");
        }
        assert!(parse_env_bool(TEST_KEY));

        unsafe {
            env::set_var(TEST_KEY, "yes");
        }
        assert!(parse_env_bool(TEST_KEY));
    }

    #[test]
    fn parse_env_bool_false_values() {
        const TEST_KEY: &str = "TEST_BOOL_FALSE";
        unsafe {
            env::set_var(TEST_KEY, "0");
        }
        assert!(!parse_env_bool(TEST_KEY));

        unsafe {
            env::remove_var(TEST_KEY);
        }
        assert!(!parse_env_bool(TEST_KEY));
    }
}
