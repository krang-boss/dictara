//! Commands for reading and pruning transcription history.

use log::error;
use tauri::State;

use crate::config::{self, ConfigKey, ConfigStore};
use crate::history::{History, HistoryEntry, HistoryQuery, HistoryStats};

/// List history entries, newest first, optionally filtered and searched.
#[tauri::command]
#[specta::specta]
pub fn list_history(
    history: State<History>,
    query: Option<HistoryQuery>,
) -> Result<Vec<HistoryEntry>, String> {
    history
        .list(&query.unwrap_or_default())
        .map_err(|e| db_error("list history", e))
}

/// Fetch a single history entry.
#[tauri::command]
#[specta::specta]
pub fn get_history_entry(history: State<History>, id: i64) -> Result<Option<HistoryEntry>, String> {
    history
        .get(id)
        .map_err(|e| db_error("read history entry", e))
}

/// Delete one history entry. Returns whether it existed.
#[tauri::command]
#[specta::specta]
pub fn delete_history_entry(history: State<History>, id: i64) -> Result<bool, String> {
    history
        .delete(id)
        .map_err(|e| db_error("delete history entry", e))
}

/// Delete the entire history.
#[tauri::command]
#[specta::specta]
pub fn clear_history(history: State<History>) -> Result<(), String> {
    history.clear().map_err(|e| db_error("clear history", e))
}

/// Aggregate counts for the history screen.
#[tauri::command]
#[specta::specta]
pub fn history_stats(history: State<History>) -> Result<HistoryStats, String> {
    history
        .stats()
        .map_err(|e| db_error("read history stats", e))
}

/// Apply the configured retention window right now.
///
/// Also runs on startup; exposed so changing the setting takes effect
/// immediately instead of at the next launch.
#[tauri::command]
#[specta::specta]
pub fn prune_history(
    history: State<History>,
    config_store: State<config::Config>,
) -> Result<u32, String> {
    let retention = config_store
        .get(&ConfigKey::APP)
        .unwrap_or_default()
        .history_retention_days;

    history
        .prune(retention)
        .map(|removed| removed as u32)
        .map_err(|e| db_error("prune history", e))
}

fn db_error(action: &str, error: rusqlite::Error) -> String {
    let message = format!("Failed to {}: {}", action, error);
    error!("{}", message);
    message
}
