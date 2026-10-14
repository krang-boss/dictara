/// Single source of truth for all commands
/// This macro takes a wrapper macro path and applies it to the command list
#[macro_export]
macro_rules! with_commands {
    ($($wrapper:tt)*) => {
        $($wrapper)*![
            // App
            $crate::commands::get_app_version,
            $crate::commands::hide_menubar_panel,
            $crate::commands::open_preferences,
            $crate::commands::paste_text,
            $crate::commands::quit_app,
            // Accessibility
            $crate::commands::check_accessibility_permission,
            $crate::commands::request_accessibility_permission,
            // Microphone
            $crate::commands::check_microphone_permission,
            $crate::commands::request_microphone_permission,
            $crate::commands::open_microphone_settings,
            // App configuration
            $crate::commands::load_app_config,
            $crate::commands::save_app_config,
            // Transcription presets
            $crate::commands::load_presets,
            $crate::commands::save_preset,
            $crate::commands::duplicate_preset,
            $crate::commands::delete_preset,
            $crate::commands::set_active_preset,
            $crate::commands::cycle_active_preset,
            $crate::commands::reorder_presets,
            $crate::commands::test_preset,
            $crate::commands::list_providers,
            // Provider credentials (Keychain)
            $crate::commands::list_credentials,
            $crate::commands::save_credentials,
            $crate::commands::delete_credentials,
            // Recording popup appearance
            $crate::commands::load_popup_style,
            $crate::commands::save_popup_style,
            $crate::commands::reset_popup_style,
            // History
            $crate::commands::list_history,
            $crate::commands::get_history_entry,
            $crate::commands::delete_history_entry,
            $crate::commands::clear_history,
            $crate::commands::history_stats,
            $crate::commands::prune_history,
            // Recording
            $crate::commands::stop_recording,
            $crate::commands::cancel_recording,
            $crate::commands::retry_transcription,
            $crate::commands::dismiss_error,
            $crate::commands::resize_popup_for_error,
            $crate::commands::register_audio_level_channel,
            // Onboarding
            $crate::commands::restart_app,
            $crate::commands::load_onboarding_config,
            $crate::commands::save_onboarding_step,
            $crate::commands::finish_onboarding,
            $crate::commands::skip_onboarding,
            $crate::commands::set_pending_restart,
            $crate::commands::restart_onboarding,
            // Shortcuts
            $crate::commands::load_shortcuts_config,
            $crate::commands::save_shortcuts_config,
            $crate::commands::reset_shortcuts_config,
            $crate::commands::start_key_capture,
            $crate::commands::stop_key_capture,
            // System
            $crate::commands::enable_autostart,
            $crate::commands::disable_autostart,
            $crate::commands::is_autostart_enabled,
            $crate::commands::mark_autostart_setup_done,
            $crate::commands::is_autostart_setup_done,
        ]
    };
}
