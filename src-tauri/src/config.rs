use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Provider types supported by the application
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, specta::Type)]
pub enum Provider {
    #[serde(rename = "open_ai", alias = "openai", alias = "open_a_i")]
    OpenAI,
    #[serde(
        rename = "azure_open_ai",
        alias = "azure",
        alias = "azure_openai",
        alias = "azure_open_a_i"
    )]
    AzureOpenAI,
    #[serde(rename = "open_router", alias = "openrouter")]
    OpenRouter,
    #[serde(rename = "eleven_labs", alias = "elevenlabs", alias = "elevenLabs")]
    ElevenLabs,
    #[serde(rename = "custom_endpoint", alias = "customEndpoint")]
    CustomEndpoint,
}

impl Provider {
    /// Every provider, in the order they are offered in the UI.
    pub const ALL: [Provider; 5] = [
        Provider::OpenAI,
        Provider::AzureOpenAI,
        Provider::OpenRouter,
        Provider::ElevenLabs,
        Provider::CustomEndpoint,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::AzureOpenAI => "Azure OpenAI",
            Provider::OpenRouter => "OpenRouter",
            Provider::ElevenLabs => "ElevenLabs",
            Provider::CustomEndpoint => "Custom Endpoint",
        }
    }

    /// Model preselected when a new preset is created for this provider.
    pub fn default_model(self) -> &'static str {
        match self {
            Provider::OpenAI => "whisper-1",
            // Azure encodes the deployment in the endpoint URL.
            Provider::AzureOpenAI => "",
            Provider::OpenRouter => "openai/whisper-1",
            Provider::ElevenLabs => "scribe_v1",
            Provider::CustomEndpoint => "whisper-1",
        }
    }

    /// Whether the preset must carry a URL of its own.
    pub fn requires_base_url(self) -> bool {
        matches!(self, Provider::AzureOpenAI | Provider::CustomEndpoint)
    }

    /// Whether the preset must name a model (Azure gets it from the URL).
    pub fn requires_model(self) -> bool {
        !matches!(self, Provider::AzureOpenAI)
    }

    /// Whether an API key is mandatory. Self-hosted OpenAI-compatible servers
    /// often need none.
    pub fn requires_api_key(self) -> bool {
        !matches!(self, Provider::CustomEndpoint)
    }
}

/// Recording trigger key options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum RecordingTrigger {
    #[default]
    Fn,
    Control,
    Option,
    Command,
}

impl RecordingTrigger {
    /// Convert to the keyboard crate's Key type
    #[allow(dead_code)]
    pub fn to_key(self) -> voiceflow_keyboard::Key {
        match self {
            RecordingTrigger::Fn => voiceflow_keyboard::Key::Function,
            RecordingTrigger::Control => voiceflow_keyboard::Key::ControlLeft,
            RecordingTrigger::Option => voiceflow_keyboard::Key::Alt,
            RecordingTrigger::Command => voiceflow_keyboard::Key::MetaLeft,
        }
    }
}

/// A single key in a shortcut combination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutKey {
    pub keycode: u32,
    pub label: String,
}

/// A keyboard shortcut (1-3 keys)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    pub keys: Vec<ShortcutKey>,
}

impl Shortcut {
    /// Check if this shortcut matches currently pressed keys
    pub fn matches(&self, pressed_keys: &std::collections::HashSet<u32>) -> bool {
        // Exact match: same count AND all keys present
        self.keys.len() == pressed_keys.len()
            && self.keys.iter().all(|k| pressed_keys.contains(&k.keycode))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.keys.is_empty() || self.keys.len() > 3 {
            return Err("Shortcut must have 1-3 keys".to_string());
        }
        // Check for duplicates
        let mut seen = std::collections::HashSet::new();
        for key in &self.keys {
            if !seen.insert(key.keycode) {
                return Err(format!("Duplicate key: {}", key.label));
            }
        }
        Ok(())
    }
}

/// Complete shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutsConfig {
    /// Push-to-talk: Hold to record, release to stop
    pub push_to_record: Shortcut,
    /// Hands-free: Press to toggle (start/stop)
    pub hands_free: Shortcut,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        let fn_key = voiceflow_keyboard::Key::Function;
        let space_key = voiceflow_keyboard::Key::Space;

        Self {
            push_to_record: Shortcut {
                keys: vec![ShortcutKey {
                    keycode: fn_key.to_macos_keycode(),
                    label: fn_key.to_label(),
                }],
            },
            hands_free: Shortcut {
                keys: vec![
                    ShortcutKey {
                        keycode: fn_key.to_macos_keycode(),
                        label: fn_key.to_label(),
                    },
                    ShortcutKey {
                        keycode: space_key.to_macos_keycode(),
                        label: space_key.to_label(),
                    },
                ],
            },
        }
    }
}

use std::marker::PhantomData;

/// Type-safe configuration key that associates a key name with its value type
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ConfigKey<T> {
    name: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> ConfigKey<T> {
    pub(crate) const fn new(name: &'static str) -> Self {
        Self {
            name,
            _phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn key_name(&self) -> &'static str {
        self.name
    }
}

/// Audio format options for transmission
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum AudioFormat {
    #[default]
    Wav,
    Mp3,
}

// ===== App Configuration =====

/// App configuration (stored locally).
///
/// Provider and model selection moved to [`crate::presets`]; what is left here
/// is genuinely app-wide.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Key used to trigger recording. Superseded by `ShortcutsConfig`; kept
    /// because the shortcuts migration still reads it.
    #[serde(default, alias = "recording_trigger")]
    pub recording_trigger: RecordingTrigger,
    /// Whether autostart has been set up on first launch
    /// This prevents re-enabling autostart after user manually disables it
    #[serde(default)]
    pub autostart_initial_setup_done: bool,
    /// How many days of transcription history to keep. 0 means "forever".
    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: u32,
}

fn default_history_retention_days() -> u32 {
    90
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            recording_trigger: RecordingTrigger::default(),
            autostart_initial_setup_done: false,
            history_retention_days: default_history_retention_days(),
        }
    }
}

impl ConfigKey<AppConfig> {
    #[allow(dead_code)]
    pub const APP: Self = Self::new("appConfig");
}

// ===== Onboarding Configuration =====

/// Onboarding step enum - tracks current position in the wizard
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type, Default)]
pub enum OnboardingStep {
    #[default]
    #[serde(rename = "welcome")]
    Welcome,
    #[serde(rename = "accessibility")]
    Accessibility,
    #[serde(rename = "microphone")]
    Microphone,
    #[serde(rename = "api_keys")]
    ApiKeys,
    #[serde(rename = "shortcuts")]
    Shortcuts,
    #[serde(rename = "fn_hold")]
    FnHold,
    #[serde(rename = "fn_space")]
    FnSpace,
    #[serde(rename = "complete")]
    Complete,
}

/// Onboarding configuration (stored locally)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingConfig {
    /// Whether the user has completed or skipped onboarding
    pub finished: bool,
    /// Current step in the onboarding flow
    #[serde(alias = "current_step")]
    pub current_step: OnboardingStep,
    /// Flag to track if we're resuming after an accessibility restart
    #[serde(alias = "pending_restart")]
    pub pending_restart: bool,
}

impl ConfigKey<OnboardingConfig> {
    #[allow(dead_code)]
    pub const ONBOARDING: Self = Self::new("onboardingConfig");
}

impl ConfigKey<ShortcutsConfig> {
    #[allow(dead_code)]
    pub const SHORTCUTS: Self = Self::new("shortcutsConfig");
}

// ===== Keychain-stored Credentials =====

/// The only thing that belongs in the Keychain: the secret itself.
///
/// Everything else a provider needs (model, endpoint, language) lives in the
/// preset, so changing it never has to touch the Keychain.
#[derive(Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCredentials {
    /// `None` for providers that accept unauthenticated requests.
    #[serde(default)]
    pub api_key: Option<String>,
}

impl std::fmt::Debug for ProviderCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderCredentials")
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

impl ProviderCredentials {
    /// A usable, non-blank key, if there is one.
    pub fn key(&self) -> Option<&str> {
        self.api_key.as_deref().filter(|k| !k.trim().is_empty())
    }
}

// ===== Type-Safe Config Store =====

pub trait ConfigStore {
    fn get<T: DeserializeOwned>(&self, key: &ConfigKey<T>) -> Option<T>;
    fn set<T: Serialize>(&self, key: &ConfigKey<T>, value: T) -> Result<(), String>;
    #[allow(dead_code)]
    fn delete<T>(&self, key: &ConfigKey<T>) -> Result<(), String>;
}

/// Type-safe configuration store that wraps the Tauri plugin store
#[derive(Clone)]
pub struct Config {
    store: std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>,
}

impl Config {
    pub fn new(store: std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>) -> Self {
        Self { store }
    }
}

impl ConfigStore for Config {
    fn get<T: DeserializeOwned>(&self, key: &ConfigKey<T>) -> Option<T> {
        self.store
            .get(key.key_name())
            .and_then(|v| serde_json::from_value(v).ok())
    }

    fn set<T: Serialize>(&self, key: &ConfigKey<T>, value: T) -> Result<(), String> {
        let val = serde_json::to_value(value).map_err(|e| e.to_string())?;
        self.store.set(key.key_name(), val);
        self.store.save().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete<T>(&self, key: &ConfigKey<T>) -> Result<(), String> {
        self.store.delete(key.key_name());
        self.store.save().map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Migrate from RecordingTrigger to ShortcutsConfig (run once on startup)
pub fn migrate_trigger_to_shortcuts(store: &impl ConfigStore) -> Result<(), String> {
    // Skip if already migrated
    if store
        .get(&ConfigKey::<ShortcutsConfig>::SHORTCUTS)
        .is_some()
    {
        return Ok(());
    }

    // Load old trigger config
    let app_config = store.get(&ConfigKey::APP).unwrap_or_default();

    // Convert old trigger to key, then to keycode and label
    let key = app_config.recording_trigger.to_key();
    let trigger_key = ShortcutKey {
        keycode: key.to_macos_keycode(),
        label: key.to_label(),
    };

    // Create new shortcuts config preserving user's trigger choice
    let space_key = voiceflow_keyboard::Key::Space;
    let shortcuts = ShortcutsConfig {
        push_to_record: Shortcut {
            keys: vec![trigger_key.clone()],
        },
        hands_free: Shortcut {
            keys: vec![
                trigger_key,
                ShortcutKey {
                    keycode: space_key.to_macos_keycode(),
                    label: space_key.to_label(),
                },
            ],
        },
    };

    store.set(&ConfigKey::<ShortcutsConfig>::SHORTCUTS, shortcuts)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Simple in-memory mock store for testing
    struct MockConfigStore {
        data: RefCell<HashMap<String, serde_json::Value>>,
    }

    impl MockConfigStore {
        fn new() -> Self {
            Self {
                data: RefCell::new(HashMap::new()),
            }
        }
    }

    impl ConfigStore for MockConfigStore {
        fn get<T: DeserializeOwned>(&self, key: &ConfigKey<T>) -> Option<T> {
            self.data
                .borrow()
                .get(key.key_name())
                .and_then(|v| serde_json::from_value(v.clone()).ok())
        }

        fn set<T: Serialize>(&self, key: &ConfigKey<T>, value: T) -> Result<(), String> {
            let val = serde_json::to_value(value).map_err(|e| e.to_string())?;
            self.data
                .borrow_mut()
                .insert(key.key_name().to_string(), val);
            Ok(())
        }

        fn delete<T>(&self, key: &ConfigKey<T>) -> Result<(), String> {
            self.data.borrow_mut().remove(key.key_name());
            Ok(())
        }
    }

    #[test]
    fn test_app_config_store() {
        let test_cases = vec![(
            "AppConfig with all fields set",
            ConfigKey::APP,
            AppConfig {
                recording_trigger: RecordingTrigger::Control,
                autostart_initial_setup_done: false,
                history_retention_days: 30,
            },
        )];

        for (description, key, config) in test_cases {
            let store = MockConfigStore::new();
            test_config_lifecycle(&store, &key, config, description);
        }
    }

    #[test]
    fn test_onboarding_config_store() {
        let test_cases = vec![
            (
                "OnboardingConfig with all fields set",
                ConfigKey::ONBOARDING,
                OnboardingConfig {
                    finished: true,
                    current_step: OnboardingStep::ApiKeys,
                    pending_restart: false,
                },
            ),
            (
                "OnboardingConfig with defaults",
                ConfigKey::ONBOARDING,
                OnboardingConfig {
                    finished: false,
                    current_step: OnboardingStep::Welcome,
                    pending_restart: false,
                },
            ),
            (
                "OnboardingConfig with pending restart",
                ConfigKey::ONBOARDING,
                OnboardingConfig {
                    finished: false,
                    current_step: OnboardingStep::Accessibility,
                    pending_restart: true,
                },
            ),
            (
                "OnboardingConfig completed",
                ConfigKey::ONBOARDING,
                OnboardingConfig {
                    finished: true,
                    current_step: OnboardingStep::Complete,
                    pending_restart: false,
                },
            ),
        ];

        for (description, key, config) in test_cases {
            let store = MockConfigStore::new();
            test_config_lifecycle(&store, &key, config, description);
        }
    }

    // Helper function to check if a string is in camelCase format
    fn is_camel_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();

        // First character must be lowercase letter
        if let Some(first) = chars.next() {
            if !first.is_ascii_lowercase() {
                return false;
            }
        }

        // Rest can be letters or digits, but no underscores or hyphens
        for c in chars {
            if !c.is_alphanumeric() {
                return false;
            }
        }

        true
    }

    // Helper function to verify camelCase format dynamically
    fn verify_camel_case<T>(store: &MockConfigStore, key: &ConfigKey<T>) {
        // Verify the config key name itself is camelCase
        assert!(
            is_camel_case(key.key_name()),
            "Config key '{}' should be camelCase",
            key.key_name()
        );

        // Get the stored JSON and verify all field keys are camelCase
        let stored_json = store.data.borrow().get(key.key_name()).cloned();
        if let Some(json_value) = stored_json {
            if let Some(obj) = json_value.as_object() {
                for field_key in obj.keys() {
                    assert!(
                        is_camel_case(field_key),
                        "Field '{}' in {} should be camelCase",
                        field_key,
                        key.key_name()
                    );
                }
            }
        }
    }

    // Helper function to test the full lifecycle of a config
    fn test_config_lifecycle<T>(
        store: &MockConfigStore,
        key: &ConfigKey<T>,
        test_config: T,
        description: &str,
    ) where
        T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug + Clone,
    {
        // Step 1: Get should return None before setting
        let result: Option<T> = store.get(key);
        assert!(
            result.is_none(),
            "{}: Get should return None before set",
            description
        );

        // Step 2: Set the value
        store
            .set(key, test_config.clone())
            .expect(&format!("{}: Set should succeed", description));

        // Step 3: Get should return the same object
        let result: Option<T> = store.get(key);
        assert!(
            result.is_some(),
            "{}: Get should return Some after set",
            description
        );
        let retrieved_config = result.unwrap();
        assert_eq!(
            retrieved_config, test_config,
            "{}: Retrieved config should match",
            description
        );

        // Step 4: Verify camelCase formatting
        verify_camel_case(&store, key);

        // Step 5: Delete the value
        store
            .delete(key)
            .expect(&format!("{}: Delete should succeed", description));

        // Step 6: Get should return None after delete
        let result: Option<T> = store.get(key);
        assert!(
            result.is_none(),
            "{}: Get should return None after delete",
            description
        );
    }
}
