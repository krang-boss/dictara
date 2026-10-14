/// Get the application version with -local suffix when running in debug mode
#[tauri::command]
#[specta::specta]
pub fn get_app_version() -> String {
    let version = env!("CARGO_PKG_VERSION");

    if cfg!(debug_assertions) {
        format!("{}-local", version)
    } else {
        version.to_string()
    }
}

/// Close the menu bar panel.
///
/// Used by the panel itself after an action that should dismiss it.
#[tauri::command]
#[specta::specta]
pub fn hide_menubar_panel(app: tauri::AppHandle) {
    crate::ui::panel::hide_panel(&app);
}

/// Open the preferences window, creating it if needed.
#[tauri::command]
#[specta::specta]
pub fn open_preferences(app: tauri::AppHandle) -> Result<(), String> {
    crate::ui::window::open_preferences_window(&app).map_err(|e| {
        let err = format!("Failed to open preferences: {}", e);
        log::error!("{}", err);
        err
    })
}

/// Paste arbitrary text into the frontmost application.
///
/// Backs "paste again" in the history list.
#[tauri::command]
#[specta::specta]
pub fn paste_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    // Dismiss the panel and hand focus back, otherwise the synthetic Cmd+V
    // lands in our own window rather than the one the user was typing in.
    crate::ui::panel::hide_panel(&app);

    if crate::ui::panel::restore_previous_app() {
        // Activation is asynchronous; give the window server a moment to make
        // the other app key before synthesizing the keystroke.
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    crate::text_paster::paste_text(&text).map_err(|e| {
        let err = format!("Failed to paste text: {}", e);
        log::error!("{}", err);
        err
    })
}

/// Quit the application.
#[tauri::command]
#[specta::specta]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
