#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "desktop")]
fn main() {
    gh_client_backend::desktop::run();
}

#[cfg(not(feature = "desktop"))]
fn main() {
    eprintln!("desktop feature is disabled. Run with --features desktop");
}
