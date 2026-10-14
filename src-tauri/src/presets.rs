//! Transcription presets.
//!
//! A preset is one named answer to "what should transcribe this recording":
//! a provider plus the model and per-request options that go with it. Several
//! presets can point at the same provider, which is the whole reason this
//! exists — the old model stored the model name *inside* the Keychain entry,
//! so one provider could only ever have one model.
//!
//! Secrets stay in the Keychain, one entry per provider ([`crate::keychain`]).
//! Everything else lives here, in the plain JSON store, so switching presets
//! never touches the Keychain and never triggers an access prompt.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::{AudioFormat, ConfigKey, ConfigStore, Provider};

/// A named transcription configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub provider: Provider,
    /// Model identifier. For Azure this is informational — the deployment is
    /// part of `base_url`.
    #[serde(default)]
    pub model: String,
    /// Azure full transcription endpoint, or the base URL of an
    /// OpenAI-compatible server. Unused by the other providers.
    #[serde(default)]
    pub base_url: Option<String>,
    /// ISO-639-1 language hint. `None` means "let the provider detect it".
    #[serde(default)]
    pub language: Option<String>,
    /// Optional prompt to bias the transcription (proper nouns, jargon…).
    #[serde(default)]
    pub prompt: Option<String>,
    /// Whether to upload WAV as-is or transcode to MP3 first.
    #[serde(default)]
    pub audio_format: AudioFormat,
}

impl Preset {
    /// Create a preset with a fresh id and the provider's default model.
    pub fn new(name: impl Into<String>, provider: Provider) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            model: provider.default_model().to_string(),
            provider,
            base_url: None,
            language: None,
            prompt: None,
            audio_format: AudioFormat::default(),
        }
    }

    /// Whether this preset has everything it needs besides the API key.
    pub fn is_complete(&self) -> bool {
        if self.provider.requires_base_url() {
            match &self.base_url {
                Some(url) if !url.trim().is_empty() => {}
                _ => return false,
            }
        }
        if self.provider.requires_model() && self.model.trim().is_empty() {
            return false;
        }
        true
    }
}

/// The full set of presets plus which one is currently active.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PresetsConfig {
    #[serde(default)]
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub active_id: Option<String>,
}

impl PresetsConfig {
    /// The active preset, if the stored id still resolves to one.
    pub fn active(&self) -> Option<&Preset> {
        let id = self.active_id.as_ref()?;
        self.presets.iter().find(|p| &p.id == id)
    }

    /// Drop a dangling `active_id` and fall back to the first preset.
    fn normalize(&mut self) {
        let active_exists = self
            .active_id
            .as_ref()
            .is_some_and(|id| self.presets.iter().any(|p| &p.id == id));

        if !active_exists {
            self.active_id = self.presets.first().map(|p| p.id.clone());
        }
    }
}

impl ConfigKey<PresetsConfig> {
    pub const PRESETS: Self = Self::new("presetsConfig");
}

/// Load presets, dropping a stale active id if one is present.
pub fn load(store: &impl ConfigStore) -> PresetsConfig {
    let mut config = store
        .get(&ConfigKey::<PresetsConfig>::PRESETS)
        .unwrap_or_default();
    config.normalize();
    config
}

/// Persist presets after normalizing the active id.
pub fn save(store: &impl ConfigStore, mut config: PresetsConfig) -> Result<(), String> {
    config.normalize();
    store.set(&ConfigKey::<PresetsConfig>::PRESETS, config)
}

// ===== Migration from the pre-presets layout =====

/// The shape `appConfig` had before presets existed.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct LegacyAppConfig {
    #[serde(default, alias = "active_provider")]
    active_provider: Option<Provider>,
    #[serde(default, alias = "audio_format")]
    audio_format: AudioFormat,
}

impl ConfigKey<LegacyAppConfig> {
    const LEGACY_APP: Self = Self::new("appConfig");
}

/// Keychain blobs as they were written before presets: model and endpoint were
/// stored alongside the secret. Unknown fields are ignored by serde, so this
/// also parses the new credential-only shape.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyProviderBlob {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    model_id: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
}

impl LegacyProviderBlob {
    fn model_name(&self) -> Option<String> {
        self.model.clone().or_else(|| self.model_id.clone())
    }

    fn url(&self) -> Option<String> {
        self.base_url.clone().or_else(|| self.endpoint.clone())
    }
}

/// Build presets out of the old single-active-provider configuration.
///
/// Runs once: if `presetsConfig` already exists, this is a no-op. Every
/// provider that has a Keychain entry becomes a preset, and the one that was
/// active stays active. Keychain secrets are left untouched — the credential
/// structs ignore the extra legacy fields.
pub fn migrate_from_legacy_config(store: &impl ConfigStore) -> Result<(), String> {
    use crate::keychain::{self, ProviderAccount};
    use log::info;

    if store.get(&ConfigKey::<PresetsConfig>::PRESETS).is_some() {
        return Ok(());
    }

    let legacy: LegacyAppConfig = store.get(&ConfigKey::LEGACY_APP).unwrap_or_default();

    let mut config = PresetsConfig::default();

    for provider in Provider::ALL {
        let account = ProviderAccount::from(provider);

        let blob = match keychain::load_provider_config::<LegacyProviderBlob>(account) {
            Ok(Some(blob)) => blob,
            // No entry, or a blob we cannot parse: nothing to migrate.
            _ => continue,
        };

        let mut preset = Preset::new(provider.display_name(), provider);
        if let Some(model) = blob.model_name() {
            if !model.trim().is_empty() {
                preset.model = model;
            }
        }
        preset.base_url = blob.url().filter(|u| !u.trim().is_empty());
        preset.audio_format = legacy.audio_format;

        if legacy.active_provider == Some(provider) {
            config.active_id = Some(preset.id.clone());
        }

        config.presets.push(preset);
    }

    if config.presets.is_empty() {
        return Ok(());
    }

    info!(
        "Migrated {} provider configuration(s) into presets",
        config.presets.len()
    );

    save(store, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockStore {
        data: RefCell<HashMap<String, serde_json::Value>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                data: RefCell::new(HashMap::new()),
            }
        }
    }

    impl ConfigStore for MockStore {
        fn get<T: serde::de::DeserializeOwned>(&self, key: &ConfigKey<T>) -> Option<T> {
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
    fn active_falls_back_to_first_preset_when_id_is_stale() {
        let store = MockStore::new();
        let preset = Preset::new("OpenAI", Provider::OpenAI);
        let id = preset.id.clone();

        save(
            &store,
            PresetsConfig {
                presets: vec![preset],
                active_id: Some("does-not-exist".to_string()),
            },
        )
        .unwrap();

        let loaded = load(&store);
        assert_eq!(loaded.active_id, Some(id));
    }

    #[test]
    fn active_is_none_when_there_are_no_presets() {
        let store = MockStore::new();
        save(&store, PresetsConfig::default()).unwrap();

        let loaded = load(&store);
        assert_eq!(loaded.active_id, None);
        assert!(loaded.active().is_none());
    }

    #[test]
    fn presets_round_trip_through_the_store() {
        let store = MockStore::new();
        let mut preset = Preset::new("Custom", Provider::CustomEndpoint);
        preset.base_url = Some("http://localhost:8080/v1".to_string());
        preset.language = Some("ru".to_string());
        let expected = preset.clone();

        save(
            &store,
            PresetsConfig {
                active_id: Some(preset.id.clone()),
                presets: vec![preset],
            },
        )
        .unwrap();

        let loaded = load(&store);
        assert_eq!(loaded.presets, vec![expected]);
    }

    #[test]
    fn incomplete_presets_are_rejected() {
        let mut custom = Preset::new("Custom", Provider::CustomEndpoint);
        assert!(!custom.is_complete(), "custom endpoint needs a base URL");

        custom.base_url = Some("https://example.com/v1".to_string());
        assert!(custom.is_complete());

        let mut openai = Preset::new("OpenAI", Provider::OpenAI);
        assert!(openai.is_complete());
        openai.model = String::new();
        assert!(!openai.is_complete(), "OpenAI needs a model");
    }

    #[test]
    fn migration_is_skipped_when_presets_already_exist() {
        let store = MockStore::new();
        let preset = Preset::new("Existing", Provider::OpenAI);
        let id = preset.id.clone();
        save(
            &store,
            PresetsConfig {
                active_id: Some(id.clone()),
                presets: vec![preset],
            },
        )
        .unwrap();

        migrate_from_legacy_config(&store).unwrap();

        let loaded = load(&store);
        assert_eq!(loaded.presets.len(), 1);
        assert_eq!(loaded.active_id, Some(id));
    }
}
