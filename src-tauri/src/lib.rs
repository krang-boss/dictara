mod autolaunch;
mod clients;
mod commands;
mod config;
mod error;
mod frontmost;
mod globe_key;
mod history;
mod keyboard_listener;
mod keychain;
mod log;
mod popup_style;
mod presets;
mod recording;
mod setup;
mod shortcuts;
mod specta;
mod text_paster;
mod ui;

pub fn run() {
    tauri::Builder::default()
        .plugin(log::create_plugin().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(setup::setup_app)
        .invoke_handler(with_commands!(tauri::generate_handler))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
