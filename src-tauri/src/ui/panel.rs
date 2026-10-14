//! The menu bar panel.
//!
//! Replaces the old native tray menu with a real window: preset switcher,
//! recent history, actions. It hangs off the status item and closes when it
//! loses focus.
//!
//! Deliberately opaque. An earlier version put it on an `NSVisualEffectView`,
//! but a translucent panel means whatever is behind it — a bright document, a
//! photo — is fighting the text on every line, and no amount of tuned contrast
//! makes that consistently comfortable to read. A plain surface in the system
//! theme is duller and much easier to use.

use log::{error, warn};
use tauri::{
    LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use std::sync::Mutex;

use super::window::AnyError;

/// Bundle id of the app that had focus when the panel opened.
///
/// Opening the panel activates VoiceFlow, so anything the panel does *to*
/// another app — pasting, most importantly — has to hand focus back first.
static PREVIOUS_APP: Mutex<Option<String>> = Mutex::new(None);

/// Give focus back to whoever had it before the panel opened.
///
/// Returns whether an app was actually reactivated, so callers know whether to
/// wait for the switch to land.
pub fn restore_previous_app() -> bool {
    let bundle_id = PREVIOUS_APP
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();

    match bundle_id {
        Some(id) => crate::frontmost::activate(&id),
        None => false,
    }
}

pub const PANEL_LABEL: &str = "menubar";

const PANEL_WIDTH: f64 = 360.0;
const PANEL_HEIGHT: f64 = 520.0;

/// Gap between the menu bar and the top of the panel.
const MENUBAR_GAP: f64 = 6.0;

/// Keep the panel this far from the screen edges.
const SCREEN_MARGIN: f64 = 8.0;

/// Create the panel window up front, hidden.
///
/// Building it eagerly keeps the first open instant; it stays hidden until the
/// user clicks the status item.
pub fn create_panel(app: &tauri::AppHandle) -> Result<WebviewWindow, AnyError> {
    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        return Ok(window);
    }

    let window = WebviewWindowBuilder::new(app, PANEL_LABEL, WebviewUrl::App("menubar".into()))
        .title("VoiceFlow")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .resizable(false)
        .decorations(false)
        // The window surface stays transparent only so the page can round its
        // own corners; the page itself paints a fully opaque background.
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(true)
        .visible(false)
        .build()?;

    Ok(window)
}

/// Show the panel anchored under the status item, or hide it if already open.
///
/// `icon_rect` is the status item's position and size in physical pixels, as
/// reported by the tray click event.
pub fn toggle_panel(
    app: &tauri::AppHandle,
    icon_rect: Option<(tauri::PhysicalPosition<f64>, tauri::PhysicalSize<f64>)>,
) {
    let app = app.clone();

    // AppKit window work has to happen on the main thread.
    let result = app.clone().run_on_main_thread(move || {
        let window = match create_panel(&app) {
            Ok(window) => window,
            Err(e) => {
                error!("Failed to create menu bar panel: {}", e);
                return;
            }
        };

        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }

        // Record the target app before we take focus away from it.
        if let Ok(mut previous) = PREVIOUS_APP.lock() {
            *previous = crate::frontmost::current().bundle_id;
        }

        position_panel(&window, icon_rect);

        if let Err(e) = window.show().and_then(|_| window.set_focus()) {
            error!("Failed to show menu bar panel: {}", e);
        }
    });

    if let Err(e) = result {
        error!("Failed to dispatch panel toggle to the main thread: {}", e);
    }
}

/// Hide the panel if it is open.
pub fn hide_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        let _ = window.hide();
    }
}

/// Find the monitor containing a given physical position.
fn find_monitor_for_pos(
    app: &tauri::AppHandle,
    pos: tauri::PhysicalPosition<f64>,
) -> Option<tauri::Monitor> {
    let monitors = app.available_monitors().ok()?;
    for monitor in monitors {
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let m_x = m_pos.x as f64;
        let m_y = m_pos.y as f64;
        let m_w = m_size.width as f64;
        let m_h = m_size.height as f64;

        if pos.x >= m_x && pos.x < m_x + m_w && pos.y >= m_y && pos.y < m_y + m_h {
            return Some(monitor);
        }
    }
    None
}

/// Find the monitor containing the cursor.
fn get_monitor_at_cursor(app: &tauri::AppHandle) -> Option<tauri::Monitor> {
    let cursor_pos = app.cursor_position().ok()?;
    find_monitor_for_pos(app, cursor_pos)
}

/// Anchor the panel under the status item, clamped to the monitor.
fn position_panel(
    window: &WebviewWindow,
    icon_rect: Option<(tauri::PhysicalPosition<f64>, tauri::PhysicalSize<f64>)>,
) {
    let app = window.app_handle();

    let monitor = icon_rect
        .as_ref()
        .and_then(|(pos, _)| find_monitor_for_pos(app, *pos))
        .or_else(|| get_monitor_at_cursor(app))
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        warn!("No monitor available; leaving the panel where it is");
        return;
    };

    let scale = monitor.scale_factor();
    let monitor_pos = monitor.position().to_logical::<f64>(scale);
    let monitor_size = monitor.size().to_logical::<f64>(scale);

    let (x, y) = match icon_rect {
        Some((position, size)) => {
            let icon = position.to_logical::<f64>(scale);
            let icon_size = size.to_logical::<f64>(scale);
            // Center on the icon, hang just below the menu bar.
            (
                icon.x + icon_size.width / 2.0 - PANEL_WIDTH / 2.0,
                icon.y + icon_size.height + MENUBAR_GAP,
            )
        }
        // No rect (keyboard trigger, or a platform that does not report one):
        // fall back to the top-right corner, where the status item lives.
        None => (
            monitor_pos.x + monitor_size.width - PANEL_WIDTH - SCREEN_MARGIN,
            monitor_pos.y + MENUBAR_GAP,
        ),
    };

    let min_x = monitor_pos.x + SCREEN_MARGIN;
    let max_x = monitor_pos.x + monitor_size.width - PANEL_WIDTH - SCREEN_MARGIN;
    // A monitor narrower than the panel would invert the range and make clamp panic.
    let x = if max_x > min_x {
        x.clamp(min_x, max_x)
    } else {
        min_x
    };

    if let Err(e) = window.set_size(LogicalSize::new(PANEL_WIDTH, PANEL_HEIGHT)) {
        warn!("Failed to size the menu bar panel: {}", e);
    }

    if let Err(e) = window.set_position(LogicalPosition::new(x, y)) {
        warn!("Failed to position the menu bar panel: {}", e);
    }
}
