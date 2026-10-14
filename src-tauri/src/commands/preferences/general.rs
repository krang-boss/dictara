use crate::config::{self, AppConfig, ConfigKey, ConfigStore, RecordingTrigger};
use tauri::State;

// ===== GENERAL APP CONFIGURATION COMMANDS =====

/// Load the entire app configuration
#[tauri::command]
#[specta::specta]
pub fn load_app_config(config_store: State<config::Config>) -> Result<AppConfig, String> {
    Ok(config_store.get(&ConfigKey::APP).unwrap_or_default())
}

/// Save app configuration. Only the fields that are passed get updated.
#[tauri::command]
#[specta::specta]
pub fn save_app_config(
    config_store: State<config::Config>,
    recording_trigger: Option<RecordingTrigger>,
    history_retention_days: Option<u32>,
) -> Result<(), String> {
    let mut config = config_store.get(&ConfigKey::APP).unwrap_or_default();

    if let Some(trigger) = recording_trigger {
        config.recording_trigger = trigger;
    }

    if let Some(days) = history_retention_days {
        config.history_retention_days = days;
    }

    config_store.set(&ConfigKey::APP, config)
}
