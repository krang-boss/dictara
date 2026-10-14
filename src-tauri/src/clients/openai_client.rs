use std::path::Path;

use secrecy::{ExposeSecret, SecretString};

use super::client::TranscriptionClient;
use super::error::TranscriptionError;
use super::options::TranscriptionOptions;

const OPENAI_TRANSCRIPTION_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const OPENAI_DEFAULT_MODEL: &str = "whisper-1";

/// OpenAI Whisper API client
pub struct OpenAIClient {
    api_key: SecretString,
    options: TranscriptionOptions,
}

impl OpenAIClient {
    pub fn new(api_key: SecretString, options: TranscriptionOptions) -> Self {
        Self { api_key, options }
    }

    fn model(&self) -> &str {
        let model = self.options.model.trim();
        if model.is_empty() {
            OPENAI_DEFAULT_MODEL
        } else {
            model
        }
    }
}

impl TranscriptionClient for OpenAIClient {
    fn transcription_url(&self) -> String {
        OPENAI_TRANSCRIPTION_URL.to_string()
    }

    fn add_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        request.bearer_auth(self.api_key.expose_secret())
    }

    fn build_form_from_path(
        &self,
        file_path: &Path,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError> {
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", file_path)
            .map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to read file: {}",
                    e
                )))
            })?
            .text("model", self.model().to_string())
            .text("temperature", "0.0")
            .text("response_format", "json");

        Ok(self.options.apply_to_form(form))
    }
}
