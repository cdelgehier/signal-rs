pub mod commands;
pub mod models;
pub mod signal;

use commands::{channels, messages, pairing};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("signal_rs=debug".parse().unwrap()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // AppHandle is only available inside setup — pass it to the actor
            let handle = signal::manager::spawn_actor(app.handle().clone());
            app.manage(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pairing::generate_qr_code,
            pairing::get_pairing_status,
            channels::get_channels,
            messages::get_messages,
            messages::send_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
