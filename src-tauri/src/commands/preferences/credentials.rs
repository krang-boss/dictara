//! Commands for the API keys held in the macOS Keychain.
//!
//! Keys are write-only from the frontend's point of view: it can store one and
//! ask whether one exists, but never read it back.

use log::error;

use crate::config::{Provider, ProviderCredentials};
use crate::keychain::{self, ProviderAccount};

/// Whether a provider currently has a key stored.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub provider: Provider,
    pub configured: bool,
}

/// Report which providers have credentials stored.
#[tauri::command]
#[specta::specta]
pub fn list_credentials() -> Result<Vec<CredentialStatus>, String> {
    Provider::ALL
        .iter()
        .map(|&provider| {
            let configured = keychain::load_provider_config::<ProviderCredentials>(
                ProviderAccount::from(provider),
            )
            .map_err(|e| {
                let err = format!(
                    "Failed to read {} credentials: {}",
                    provider.display_name(),
                    e
                );
                error!("{}", err);
                err
            })?
            .as_ref()
            .and_then(ProviderCredentials::key)
            .is_some();

            Ok(CredentialStatus {
                provider,
                configured,
            })
        })
        .collect()
}

/// Store a provider's API key. An empty key deletes the entry instead.
#[tauri::command]
#[specta::specta]
pub fn save_credentials(provider: Provider, api_key: String) -> Result<(), String> {
    let account = ProviderAccount::from(provider);

    if api_key.trim().is_empty() {
        return delete_credentials(provider);
    }

    let credentials = ProviderCredentials {
        api_key: Some(api_key),
    };

    keychain::save_provider_config(account, &credentials).map_err(|e| {
        let err = format!(
            "Failed to save {} credentials: {}",
            provider.display_name(),
            e
        );
        error!("{}", err);
        err
    })
}

/// Remove a provider's API key from the Keychain.
#[tauri::command]
#[specta::specta]
pub fn delete_credentials(provider: Provider) -> Result<(), String> {
    keychain::delete_provider_config(ProviderAccount::from(provider)).map_err(|e| {
        let err = format!(
            "Failed to delete {} credentials: {}",
            provider.display_name(),
            e
        );
        error!("{}", err);
        err
    })
}
