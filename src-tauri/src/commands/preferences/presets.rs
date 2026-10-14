//! Commands for managing transcription presets.

use log::error;
use tauri::State;

use crate::clients::Transcriber;
use crate::config::{self, Provider};
use crate::presets::{self, Preset, PresetsConfig};

/// Load every preset plus the active selection.
#[tauri::command]
#[specta::specta]
pub fn load_presets(config_store: State<config::Config>) -> Result<PresetsConfig, String> {
    Ok(presets::load(&*config_store))
}

/// Create a preset, or update it in place when the id already exists.
///
/// Returns the stored preset so the frontend picks up a generated id.
#[tauri::command]
#[specta::specta]
pub fn save_preset(config_store: State<config::Config>, preset: Preset) -> Result<Preset, String> {
    if preset.name.trim().is_empty() {
        return Err("Preset name cannot be empty".to_string());
    }

    let mut config = presets::load(&*config_store);

    let mut preset = preset;
    if preset.id.trim().is_empty() {
        preset.id = uuid::Uuid::new_v4().to_string();
    }

    match config.presets.iter_mut().find(|p| p.id == preset.id) {
        Some(existing) => *existing = preset.clone(),
        None => {
            // First preset created becomes the active one.
            if config.presets.is_empty() {
                config.active_id = Some(preset.id.clone());
            }
            config.presets.push(preset.clone());
        }
    }

    presets::save(&*config_store, config)?;
    Ok(preset)
}

/// Duplicate a preset, appending it right after the original.
#[tauri::command]
#[specta::specta]
pub fn duplicate_preset(config_store: State<config::Config>, id: String) -> Result<Preset, String> {
    let mut config = presets::load(&*config_store);

    let index = config
        .presets
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| format!("Preset not found: {}", id))?;

    let mut copy = config.presets[index].clone();
    copy.id = uuid::Uuid::new_v4().to_string();
    copy.name = format!("{} copy", copy.name);

    config.presets.insert(index + 1, copy.clone());
    presets::save(&*config_store, config)?;

    Ok(copy)
}

/// Delete a preset. If it was active, the first remaining preset takes over.
#[tauri::command]
#[specta::specta]
pub fn delete_preset(config_store: State<config::Config>, id: String) -> Result<(), String> {
    let mut config = presets::load(&*config_store);
    config.presets.retain(|p| p.id != id);
    presets::save(&*config_store, config)
}

/// Switch the active preset.
#[tauri::command]
#[specta::specta]
pub fn set_active_preset(
    config_store: State<config::Config>,
    id: Option<String>,
) -> Result<(), String> {
    let mut config = presets::load(&*config_store);

    if let Some(id) = &id {
        if !config.presets.iter().any(|p| &p.id == id) {
            return Err(format!("Preset not found: {}", id));
        }
    }

    config.active_id = id;
    presets::save(&*config_store, config)
}

/// Move to the next preset in the list, wrapping around.
///
/// Used by the menu bar and the cycle shortcut.
#[tauri::command]
#[specta::specta]
pub fn cycle_active_preset(config_store: State<config::Config>) -> Result<Option<Preset>, String> {
    let mut config = presets::load(&*config_store);

    if config.presets.is_empty() {
        return Ok(None);
    }

    let current = config
        .active_id
        .as_ref()
        .and_then(|id| config.presets.iter().position(|p| &p.id == id))
        .unwrap_or(0);

    let next = (current + 1) % config.presets.len();
    let preset = config.presets[next].clone();
    config.active_id = Some(preset.id.clone());

    presets::save(&*config_store, config)?;
    Ok(Some(preset))
}

/// Reorder presets to match the given list of ids.
///
/// Ids that are not in the list keep their relative order at the end, so a
/// stale frontend list cannot silently drop presets.
#[tauri::command]
#[specta::specta]
pub fn reorder_presets(
    config_store: State<config::Config>,
    ids: Vec<String>,
) -> Result<(), String> {
    let mut config = presets::load(&*config_store);

    let mut ordered: Vec<Preset> = Vec::with_capacity(config.presets.len());
    for id in &ids {
        if let Some(index) = config.presets.iter().position(|p| &p.id == id) {
            ordered.push(config.presets.remove(index));
        }
    }
    ordered.append(&mut config.presets);
    config.presets = ordered;

    presets::save(&*config_store, config)
}

/// Send a short silent clip through a preset to check it end to end.
///
/// `Ok(false)` means the credentials were rejected; `Err` means the request
/// itself failed.
#[tauri::command]
#[specta::specta]
pub async fn test_preset(preset: Preset) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Transcriber::test_preset(&preset).map_err(|e| {
            let err = format!("Preset test failed: {}", e);
            error!("{}", err);
            err
        })
    })
    .await
    .map_err(|e| format!("Preset test task failed: {}", e))?
}

/// Static description of a provider, for building the preset editor.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub provider: Provider,
    pub display_name: String,
    pub default_model: String,
    pub requires_base_url: bool,
    pub requires_model: bool,
    pub requires_api_key: bool,
}

/// List the supported providers and what each of them needs.
#[tauri::command]
#[specta::specta]
pub fn list_providers() -> Vec<ProviderInfo> {
    Provider::ALL
        .iter()
        .map(|&provider| ProviderInfo {
            provider,
            display_name: provider.display_name().to_string(),
            default_model: provider.default_model().to_string(),
            requires_base_url: provider.requires_base_url(),
            requires_model: provider.requires_model(),
            requires_api_key: provider.requires_api_key(),
        })
        .collect()
}
