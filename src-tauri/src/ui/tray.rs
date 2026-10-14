use crate::recording::LastRecordingState;
use crate::ui::{
    menu::{Menu, MenuId},
    panel, window,
};
use log::{error, warn};
use std::str::FromStr;
use tauri::{self, menu::MenuEvent, tray, Manager, Wry};

/// The menu bar icon, rendered from `icons/tray-icon.svg`.
///
/// AppKit scales it to a fixed 18pt height, so the file is deliberately larger
/// than that: the extra pixels are what keep the stroke sharp on Retina.
const TRAY_ICON_BYTES: &[u8] = include_bytes!("../../icons/tray-icon.png");

pub struct Tray {
    #[allow(dead_code)]
    tray_icon: tray::TrayIcon<Wry>,
}

impl Tray {
    pub fn new(app: &tauri::App<Wry>, menu: &Menu) -> Result<Self, tauri::Error> {
        let icon = Self::load_icon();
        let tray_icon = Self::build_tray(app, icon, menu)?;

        Ok(Tray { tray_icon })
    }

    fn load_icon() -> tauri::image::Image<'static> {
        let tray_icon_image = image::load_from_memory(TRAY_ICON_BYTES)
            .expect("Failed to load tray icon")
            .to_rgba8();
        let (width, height) = tray_icon_image.dimensions();
        tauri::image::Image::new_owned(tray_icon_image.into_raw(), width, height)
    }

    fn build_tray(
        app: &tauri::App<Wry>,
        icon: tauri::image::Image<'static>,
        menu: &Menu,
    ) -> Result<tray::TrayIcon<Wry>, tauri::Error> {
        tray::TrayIconBuilder::new()
            .icon(icon)
            .icon_as_template(true) // macOS template image - auto-adapts to light/dark mode
            .menu(&menu.menu)
            // Left click opens the panel; the native menu stays on right click
            // as an escape hatch if the panel ever fails to show.
            .show_menu_on_left_click(false)
            .on_menu_event(Self::handle_menu_event)
            .on_tray_icon_event(Self::handle_tray_event)
            .build(app)
    }

    /// Toggle the menu bar panel on a left click, anchored to the icon.
    fn handle_tray_event(tray_icon: &tray::TrayIcon<Wry>, event: tray::TrayIconEvent) {
        let tray::TrayIconEvent::Click {
            button: tray::MouseButton::Left,
            button_state: tray::MouseButtonState::Up,
            rect,
            ..
        } = event
        else {
            return;
        };

        let icon_rect = match (rect.position, rect.size) {
            (tauri::Position::Physical(position), tauri::Size::Physical(size)) => Some((
                tauri::PhysicalPosition::new(position.x as f64, position.y as f64),
                tauri::PhysicalSize::new(size.width as f64, size.height as f64),
            )),
            _ => None,
        };

        panel::toggle_panel(tray_icon.app_handle(), icon_rect);
    }

    fn handle_menu_event(app: &tauri::AppHandle<Wry>, event: MenuEvent) {
        let Ok(menu_id) = MenuId::from_str(event.id().as_ref()) else {
            warn!("Unknown menu event id: {}", event.id().as_ref());
            return;
        };

        match menu_id {
            MenuId::Preferences => {
                if let Err(e) = window::open_preferences_window(app) {
                    error!("Failed to open preferences window: {}", e);
                }
            }
            MenuId::PasteLastRecording => {
                Self::handle_paste_last_recording(app);
            }
            MenuId::Quit => {
                app.exit(0);
            }
        }
    }

    // TODO: Refactor: I do not like this nesting. Will leave it like this for now and will refactor later.
    // Also might need to pull the pasting into a separate module
    fn handle_paste_last_recording(app: &tauri::AppHandle<Wry>) {
        if let Some(state) = app.try_state::<LastRecordingState>() {
            if let Ok(last_recording) = state.lock() {
                if let Some(text) = &last_recording.text {
                    if let Err(e) = crate::text_paster::paste_text(text) {
                        error!("Failed to paste last recording: {:?}", e);
                    }
                }
            } else {
                error!("Failed to lock last recording state");
            }
        }
    }
}
