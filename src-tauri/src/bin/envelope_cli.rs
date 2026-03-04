use std::io::{self, Read};

use gh_client_backend::app_ipc::execute_frontend_envelope;
use gh_client_backend::contract::FrontendCommandEnvelope;
use gh_client_backend::core::executor::ProcessRunner;
use gh_client_backend::frontend::FrontendDispatcher;
use serde_json::{Value, json};

fn main() {
    if let Err(err) = run() {
        let payload = json!({
            "ok": false,
            "error": {
                "code": "execution_error",
                "message": err,
            }
        });
        println!(
            "{}",
            serde_json::to_string(&payload).expect("serialize error payload")
        );
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|err| format!("failed to read stdin: {}", err))?;

    let envelope: FrontendCommandEnvelope =
        serde_json::from_str(&input).map_err(|err| format!("invalid envelope json: {}", err))?;

    let safe_test_mode = parse_env_bool("SAFE_TEST_MODE");
    let dispatcher = FrontendDispatcher::new(ProcessRunner, safe_test_mode)
        .map_err(|err| format!("failed to create dispatcher: {}", err))?;

    match execute_frontend_envelope(&dispatcher, envelope) {
        Ok(data) => {
            let payload = json!({"ok": true, "data": data});
            println!(
                "{}",
                serde_json::to_string(&payload).expect("serialize success payload")
            );
            Ok(())
        }
        Err(err) => {
            let payload = json!({"ok": false, "error": err});
            println!(
                "{}",
                serde_json::to_string(&payload).expect("serialize error payload")
            );
            Ok(())
        }
    }
}

fn parse_env_bool(key: &str) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        ),
        Err(_) => false,
    }
}

#[allow(dead_code)]
fn _type_guard(_value: Value) {}
