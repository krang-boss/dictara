use crate::{
    autolaunch,
    config::{self, ConfigKey, ConfigStore, OnboardingStep, ShortcutsConfig},
    globe_key,
    history::History,
    keyboard_listener::KeyListener,
    keychain, presets,
    recording::{
        cleanup_old_recordings, Controller, LastRecording, LastRecordingState, RecordingCommand,
        RecordingStateManager,
    },
    specta,
    ui::{menu::Menu, panel, tray::Tray, window},
};
use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::mpsc;

pub struct RecordingCommandSender {
    pub sender: mpsc::Sender<RecordingCommand>,
}

pub struct AudioLevelChannel {
    pub channel: Arc<Mutex<Option<Channel<f32>>>>,
}

pub fn setup_app(app: &mut tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
    info!("VoiceFlow v{}", env!("CARGO_PKG_VERSION"));

    // Setup Specta for type-safe TypeScript bindings and event emission
    specta::setup(app.handle());

    // Clean up old recordings from previous sessions
    cleanup_old_recordings(app.app_handle());

    // Check accessibility permission on macOS
    #[cfg(target_os = "macos")]
    {
        let has_permission = macos_accessibility_client::accessibility::application_is_trusted();
        if !has_permission {
            warn!("Accessibility permission not granted");
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Keep the app running in the background
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    // Load app config and check if properly configured
    // Carry API keys over from the pre-rename bundle id (one-time, idempotent)
    keychain::migrate_legacy_bundle();

    let store = app.store("config.json")?;

    // Create and register Config as managed state
    let config_store = config::Config::new(store.clone());
    app.manage(config_store.clone());

    let mut app_config = config_store.get(&ConfigKey::APP).unwrap_or_default();
    let mut onboarding_config = config_store.get(&ConfigKey::ONBOARDING).unwrap_or_default();

    // Enable autostart on first launch (before onboarding is completed)
    autolaunch::setup_autolaunch_if_needed(app.handle(), &config_store, &mut app_config);

    // Load or migrate shortcuts config
    if let Err(e) = config::migrate_trigger_to_shortcuts(&config_store) {
        warn!("Failed to migrate shortcuts config: {}", e);
    }
    let shortcuts_config = config_store
        .get(&ConfigKey::<ShortcutsConfig>::SHORTCUTS)
        .unwrap_or_default();

    // Handle pending restart from accessibility step
    if onboarding_config.pending_restart {
        onboarding_config.pending_restart = false;

        // Check if accessibility is now granted
        #[cfg(target_os = "macos")]
        {
            let has_accessibility =
                macos_accessibility_client::accessibility::application_is_trusted();
            if has_accessibility {
                onboarding_config.current_step = OnboardingStep::Microphone;
            }
        }

        // Save the updated config
        config_store.set(&ConfigKey::ONBOARDING, onboarding_config.clone())?;
    }

    // Open the transcription history and apply the retention window.
    let history_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?
        .join("history.sqlite3");

    let history = History::open(&history_path)
        .map_err(|e| format!("Failed to open history database: {}", e))?;

    if let Err(e) = history.prune(app_config.history_retention_days) {
        warn!("Failed to prune history: {}", e);
    }
    app.manage(history);

    // Build presets out of the old single-provider config, once.
    if let Err(e) = presets::migrate_from_legacy_config(&config_store) {
        warn!("Failed to migrate provider config into presets: {}", e);
    }

    // The app is unusable until at least one preset can actually transcribe.
    let needs_configuration = presets::load(&config_store).active().is_none();
    if needs_configuration {
        warn!("No transcription preset configured");
    }

    // Determine if we need to show onboarding
    let show_onboarding = !onboarding_config.finished;

    // ========================================
    // CHANNEL-BASED ARCHITECTURE WITH CONTROLLER
    // Setup creates the channel and wires components together
    // ========================================

    // Create channel for recording commands (KeyListener → Controller)
    let (command_tx, command_rx) = mpsc::channel::<RecordingCommand>(100);
    let state_manager = Arc::new(RecordingStateManager::new());

    // Clone sender for Tauri state (mpsc::Sender is Clone + Send + Sync)
    let command_sender_state = RecordingCommandSender {
        sender: command_tx.clone(),
    };

    // Create audio level channel state
    let audio_level_channel = AudioLevelChannel {
        channel: Arc::new(Mutex::new(None)),
    };

    // Create last recording state for paste retry functionality
    let last_recording_state: LastRecordingState = Arc::new(Mutex::new(LastRecording::new()));

    let menu = Menu::new(app)?;
    let _tray = Tray::new(app, &menu)?;

    // Build the menu bar panel now so the first click opens instantly, and
    // dismiss it whenever it loses focus, the way a real popover behaves.
    match panel::create_panel(app.app_handle()) {
        Ok(panel_window) => {
            let handle = panel_window.clone();
            panel_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = handle.hide();
                }
            });
        }
        Err(e) => error!("Failed to create menu bar panel: {}", e),
    }

    // Initialize controller (transcriber created on-demand from config)
    let controller = Controller::new(
        command_rx,
        app.app_handle().clone(),
        state_manager.clone(),
        audio_level_channel.channel.clone(),
        last_recording_state.clone(),
        menu,
    );

    // Spawn controller in blocking thread (cpal::Stream is not Send)
    std::thread::spawn(move || {
        controller.run();
    });

    // Store sender and audio level channel in app state for Tauri commands
    app.manage(command_sender_state);
    app.manage(audio_level_channel);
    app.manage(last_recording_state.clone());

    // Only start keyboard listener if accessibility permission is granted
    // This prevents the permission dialog from appearing during onboarding
    #[cfg(target_os = "macos")]
    let has_accessibility = macos_accessibility_client::accessibility::application_is_trusted();
    #[cfg(not(target_os = "macos"))]
    let has_accessibility = true;

    if has_accessibility {
        let listener =
            KeyListener::start(command_tx, state_manager.clone(), shortcuts_config.clone());

        // Manage KeyListener in Tauri state for hot-swapping
        app.manage(listener);
    }

    // Only fix the Globe key setting when using Fn in any shortcut
    // This prevents the emoji picker from appearing when using Fn for recording
    if KeyListener::uses_fn_key(&shortcuts_config) {
        globe_key::fix_globe_key_if_needed();
    }

    // Decide which window to open
    if show_onboarding {
        if let Err(e) = window::open_onboarding_window(app.app_handle()) {
            error!("Failed to open onboarding window: {}", e);
        }
    } else if needs_configuration {
        if let Err(e) = window::open_preferences_window(app.app_handle()) {
            error!("Failed to open preferences window: {}", e);
        }
    }

    Ok(())
}
