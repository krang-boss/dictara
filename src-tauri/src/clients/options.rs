//! Per-request transcription options carried by a preset.

use crate::presets::Preset;

/// The parts of a preset that shape a single transcription request.
#[derive(Debug, Clone, Default)]
pub struct TranscriptionOptions {
    pub model: String,
    /// ISO-639-1 hint, or `None` to let the provider detect the language.
    pub language: Option<String>,
    /// Prompt biasing the decoder toward specific vocabulary.
    pub prompt: Option<String>,
}

impl TranscriptionOptions {
    pub fn from_preset(preset: &Preset) -> Self {
        Self {
            model: preset.model.clone(),
            language: non_blank(preset.language.as_deref()),
            prompt: non_blank(preset.prompt.as_deref()),
        }
    }

    /// Attach the optional fields shared by the OpenAI-compatible providers.
    pub fn apply_to_form(
        &self,
        mut form: reqwest::blocking::multipart::Form,
    ) -> reqwest::blocking::multipart::Form {
        if let Some(language) = &self.language {
            form = form.text("language", language.clone());
        }
        if let Some(prompt) = &self.prompt {
            form = form.text("prompt", prompt.clone());
        }
        form
    }
}

fn non_blank(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}
