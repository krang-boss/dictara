use std::path::Path;

use secrecy::{ExposeSecret, SecretString};

use super::client::TranscriptionClient;
use super::error::TranscriptionError;
use super::options::TranscriptionOptions;

const ELEVENLABS_TRANSCRIPTION_URL: &str = "https://api.elevenlabs.io/v1/speech-to-text";

/// ElevenLabs Speech to Text API client.
pub struct ElevenLabsClient {
    api_key: SecretString,
    options: TranscriptionOptions,
}

impl ElevenLabsClient {
    pub fn new(api_key: SecretString, options: TranscriptionOptions) -> Self {
        Self { api_key, options }
    }
}

impl TranscriptionClient for ElevenLabsClient {
    fn transcription_url(&self) -> String {
        ELEVENLABS_TRANSCRIPTION_URL.to_string()
    }

    fn add_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        request.header("xi-api-key", self.api_key.expose_secret())
    }

    fn build_form_from_path(
        &self,
        file_path: &Path,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError> {
        let mut form = reqwest::blocking::multipart::Form::new()
            .file("file", file_path)
            .map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to read file: {}",
                    e
                )))
            })?
            .text("model_id", self.options.model.clone())
            .text("tag_audio_events", "true")
            .text("diarize", "true");

        // ElevenLabs names the language field differently and has no prompt.
        if let Some(language) = &self.options.language {
            form = form.text("language_code", language.clone());
        }

        Ok(form)
    }
}
