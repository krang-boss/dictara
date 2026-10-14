//! Commands for the recording popup's appearance.

use tauri::State;

use crate::config::{self, ConfigKey, ConfigStore};
use crate::popup_style::PopupStyle;

/// Load the popup style, falling back to the default.
#[tauri::command]
#[specta::specta]
pub fn load_popup_style(config_store: State<config::Config>) -> Result<PopupStyle, String> {
    Ok(config_store
        .get(&ConfigKey::<PopupStyle>::POPUP_STYLE)
        .unwrap_or_default()
        .sanitized())
}

/// Save the popup style chosen in the laboratory.
#[tauri::command]
#[specta::specta]
pub fn save_popup_style(
    config_store: State<config::Config>,
    style: PopupStyle,
) -> Result<PopupStyle, String> {
    let style = style.sanitized();
    config_store.set(&ConfigKey::<PopupStyle>::POPUP_STYLE, style.clone())?;
    Ok(style)
}

/// Restore the default popup style.
#[tauri::command]
#[specta::specta]
pub fn reset_popup_style(config_store: State<config::Config>) -> Result<PopupStyle, String> {
    let style = PopupStyle::default();
    config_store.set(&ConfigKey::<PopupStyle>::POPUP_STYLE, style.clone())?;
    Ok(style)
}
