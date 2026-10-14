use keyring::Entry;
use serde::{de::DeserializeOwned, Serialize};

use crate::error;

#[cfg(debug_assertions)]
const BUNDLE: &str = "app.voiceflow.dev";

#[cfg(not(debug_assertions))]
const BUNDLE: &str = "app.voiceflow";

/// Bundle identifiers used before the app was renamed to VoiceFlow.
/// Kept only so existing API keys survive the rename.
#[cfg(debug_assertions)]
const LEGACY_BUNDLE: &str = "app.dictara.dev";

#[cfg(not(debug_assertions))]
const LEGACY_BUNDLE: &str = "app.dictara";

#[derive(Debug, Clone, Copy, strum::AsRefStr, strum::EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum ProviderAccount {
    OpenAI,
    AzureOpenAI,
    OpenRouter,
    ElevenLabs,
    CustomEndpoint,
}

impl From<crate::config::Provider> for ProviderAccount {
    fn from(provider: crate::config::Provider) -> Self {
        use crate::config::Provider;
        match provider {
            Provider::OpenAI => ProviderAccount::OpenAI,
            Provider::AzureOpenAI => ProviderAccount::AzureOpenAI,
            Provider::OpenRouter => ProviderAccount::OpenRouter,
            Provider::ElevenLabs => ProviderAccount::ElevenLabs,
            Provider::CustomEndpoint => ProviderAccount::CustomEndpoint,
        }
    }
}

/// Save provider configuration as JSON to keychain
pub fn save_provider_config<T: Serialize>(
    account: ProviderAccount,
    config: &T,
) -> Result<(), error::Error> {
    let account_name = account.as_ref();
    let entry = Entry::new(BUNDLE, account_name)?;
    let json = serde_json::to_string(config)?;
    entry.set_password(&json)?;
    Ok(())
}

/// Load provider configuration from keychain as JSON
pub fn load_provider_config<T: DeserializeOwned>(
    account: ProviderAccount,
) -> Result<Option<T>, error::Error> {
    let account_name = account.as_ref();
    let entry = Entry::new(BUNDLE, account_name)?;

    match entry.get_password() {
        Ok(json) => {
            let config: T = serde_json::from_str(&json)?;
            Ok(Some(config))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete provider configuration from keychain
pub fn delete_provider_config(account: ProviderAccount) -> Result<(), error::Error> {
    let account_name = account.as_ref();
    let entry = Entry::new(BUNDLE, account_name)?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Copy credentials stored under the pre-rename bundle id into the new one.
///
/// Runs once on startup. Entries already present under the new bundle id are
/// left untouched, so this is safe to call repeatedly. The legacy entries are
/// deliberately not deleted — if something goes wrong they are still
/// recoverable from Keychain Access.
pub fn migrate_legacy_bundle() {
    use log::{info, warn};

    for account in [
        ProviderAccount::OpenAI,
        ProviderAccount::AzureOpenAI,
        ProviderAccount::OpenRouter,
        ProviderAccount::ElevenLabs,
        ProviderAccount::CustomEndpoint,
    ] {
        let name = account.as_ref();

        // Skip if the new entry already exists.
        match Entry::new(BUNDLE, name).and_then(|e| e.get_password()) {
            Ok(_) => continue,
            Err(keyring::Error::NoEntry) => {}
            Err(e) => {
                warn!("Keychain read failed for {}: {}", name, e);
                continue;
            }
        }

        let legacy = match Entry::new(LEGACY_BUNDLE, name).and_then(|e| e.get_password()) {
            Ok(secret) => secret,
            Err(keyring::Error::NoEntry) => continue,
            Err(e) => {
                warn!("Legacy keychain read failed for {}: {}", name, e);
                continue;
            }
        };

        match Entry::new(BUNDLE, name).and_then(|e| e.set_password(&legacy)) {
            Ok(()) => info!("Migrated {} credentials from {}", name, LEGACY_BUNDLE),
            Err(e) => warn!("Failed to migrate {} credentials: {}", name, e),
        }
    }
}
