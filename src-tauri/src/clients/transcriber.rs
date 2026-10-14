use std::path::{Path, PathBuf};

use log::{error, warn};
use secrecy::SecretString;
use tauri::{AppHandle, Manager};

use crate::config::{self, AudioFormat, Provider, ProviderCredentials};
use crate::keychain::{self, ProviderAccount};
use crate::presets::{self, Preset};

use super::api_transcriber::ApiTranscriber;
use super::azure_client::AzureClient;
use super::client::TranscriptionClient;
use super::custom_endpoint_client::CustomEndpointClient;
use super::elevenlabs_client::ElevenLabsClient;
use super::error::TranscriptionError;
use super::openai_client::OpenAIClient;
use super::openrouter_transcriber::OpenRouterTranscriber;
use super::options::TranscriptionOptions;
use super::service::TranscriptionService;

const MIN_AUDIO_DURATION_MS: u64 = 500; // Minimum 0.5 seconds
const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25MB limit

/// Timeout for transcription requests in seconds (applies to all providers)
pub const TRANSCRIPTION_TIMEOUT_SECS: u64 = 120;

// Pre-generated 1-second silent WAV file (16kHz, mono) for API testing
static SILENT_WAV: &[u8] = include_bytes!("../../assets/silent_1s.wav");

/// Transcription service that orchestrates audio transcription.
///
/// Built from the active preset: the preset decides the provider, the model
/// and the per-request options, while the secret comes from the Keychain.
pub struct Transcriber {
    service: Box<dyn TranscriptionService>,
    audio_format: AudioFormat,
    preset: Preset,
}

impl Transcriber {
    /// Create a new Transcriber from the active preset.
    pub fn from_app(app: &AppHandle) -> Result<Self, TranscriptionError> {
        let config_store = app.state::<config::Config>();
        let presets = presets::load(&*config_store);

        let preset = presets
            .active()
            .cloned()
            .ok_or(TranscriptionError::NoPresetSelected)?;

        Ok(Self {
            service: Self::create_service(&preset)?,
            audio_format: preset.audio_format,
            preset,
        })
    }

    /// The preset this transcriber was built from.
    pub fn preset(&self) -> &Preset {
        &self.preset
    }

    /// Check a preset end to end by transcribing an embedded silent clip.
    ///
    /// # Returns
    /// * `Ok(true)` - Preset works
    /// * `Ok(false)` - Credentials were rejected (401)
    /// * `Err(TranscriptionError)` - Network or other API error
    pub fn test_preset(preset: &Preset) -> Result<bool, TranscriptionError> {
        let service = Self::create_service(preset)?;

        let temp_path = std::env::temp_dir().join(format!("voiceflow_test_{}.wav", preset.id));
        std::fs::write(&temp_path, SILENT_WAV).map_err(|e| {
            TranscriptionError::IoError(std::io::Error::other(format!(
                "Failed to write test audio: {}",
                e
            )))
        })?;

        let result = match service.transcribe(&temp_path) {
            Ok(_) => Ok(true),
            Err(TranscriptionError::ApiError(msg))
                if msg.contains("401") || msg.contains("Unauthorized") =>
            {
                warn!("Credentials rejected for preset '{}'", preset.name);
                Ok(false)
            }
            Err(e) => Err(e),
        };

        if let Err(e) = std::fs::remove_file(&temp_path) {
            warn!(
                "Failed to clean up temp file '{}': {}. File may need manual cleanup.",
                temp_path.display(),
                e
            );
        }

        result
    }

    /// Transcribe audio file to text.
    ///
    /// # Arguments
    /// * `file_path` - Path to the audio file (WAV, MP3, etc.)
    /// * `duration_ms` - Duration of the recording in milliseconds (for validation)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Error details
    pub fn transcribe(
        &self,
        file_path: PathBuf,
        duration_ms: u64,
    ) -> Result<String, TranscriptionError> {
        // Validate minimum duration
        if duration_ms < MIN_AUDIO_DURATION_MS {
            warn!(
                "Audio too short: {}ms < {}ms minimum",
                duration_ms, MIN_AUDIO_DURATION_MS
            );
            return Ok(String::new());
        }

        // Validate file
        self.validate_file(&file_path)?;

        if self.audio_format != AudioFormat::Mp3 {
            return self.service.transcribe(&file_path);
        }

        log::info!(
            "Converting WAV to MP3 for API transmission: {:?}",
            file_path
        );
        let mp3_path = convert_wav_to_mp3(&file_path)?;

        let result = self.service.transcribe(&mp3_path);

        if let Err(e) = std::fs::remove_file(&mp3_path) {
            warn!(
                "Failed to delete temporary MP3 file '{:?}': {}",
                mp3_path, e
            );
        }

        result
    }

    // ========== Private methods ==========

    /// Build the transcription service described by a preset.
    fn create_service(
        preset: &Preset,
    ) -> Result<Box<dyn TranscriptionService>, TranscriptionError> {
        if !preset.is_complete() {
            return Err(TranscriptionError::PresetIncomplete(preset.name.clone()));
        }

        let credentials = Self::load_credentials(preset.provider)?;
        let options = TranscriptionOptions::from_preset(preset);

        // OpenRouter speaks JSON + base64 rather than multipart, so it is a
        // service of its own rather than a `TranscriptionClient`.
        if preset.provider == Provider::OpenRouter {
            let key = Self::require_key(&credentials, preset.provider)?;
            return Ok(Box::new(OpenRouterTranscriber::new(key, options)));
        }

        let client: Box<dyn TranscriptionClient> = match preset.provider {
            Provider::OpenAI => Box::new(OpenAIClient::new(
                Self::require_key(&credentials, preset.provider)?,
                options,
            )),
            Provider::AzureOpenAI => Box::new(AzureClient::new(
                Self::require_key(&credentials, preset.provider)?,
                preset.base_url.clone().unwrap_or_default(),
                options,
            )),
            Provider::ElevenLabs => Box::new(ElevenLabsClient::new(
                Self::require_key(&credentials, preset.provider)?,
                options,
            )),
            Provider::CustomEndpoint => Box::new(CustomEndpointClient::new(
                credentials.key().map(SecretString::from),
                preset.base_url.clone().unwrap_or_default(),
                options,
            )),
            Provider::OpenRouter => unreachable!("handled above"),
        };

        Ok(Box::new(ApiTranscriber::new(client)))
    }

    fn load_credentials(provider: Provider) -> Result<ProviderCredentials, TranscriptionError> {
        let account = ProviderAccount::from(provider);
        let stored = keychain::load_provider_config::<ProviderCredentials>(account)
            .map_err(|_| TranscriptionError::ApiKeyMissing)?;

        match stored {
            Some(credentials) => Ok(credentials),
            // A provider that needs no key is fine without a Keychain entry.
            None if !provider.requires_api_key() => Ok(ProviderCredentials::default()),
            None => Err(TranscriptionError::ApiKeyMissing),
        }
    }

    fn require_key(
        credentials: &ProviderCredentials,
        provider: Provider,
    ) -> Result<SecretString, TranscriptionError> {
        credentials.key().map(SecretString::from).ok_or_else(|| {
            error!("No API key stored for {}", provider.display_name());
            TranscriptionError::ApiKeyMissing
        })
    }

    /// Validate file exists and is within size limits.
    fn validate_file(&self, file_path: &Path) -> Result<(), TranscriptionError> {
        if !file_path.exists() {
            error!("File not found: {:?}", file_path);
            return Err(TranscriptionError::FileNotFound(
                file_path.to_string_lossy().to_string(),
            ));
        }

        let metadata = std::fs::metadata(file_path)?;
        let file_size = metadata.len();

        if file_size > MAX_FILE_SIZE_BYTES {
            error!(
                "File too large: {} bytes > {} bytes",
                file_size, MAX_FILE_SIZE_BYTES
            );
            return Err(TranscriptionError::FileTooLarge {
                size_bytes: file_size,
            });
        }

        Ok(())
    }
}

/// Helper function to convert 16kHz mono WAV file to MP3 using pure-Rust shine-rs encoder
fn convert_wav_to_mp3(wav_path: &Path) -> Result<PathBuf, TranscriptionError> {
    use hound::WavReader;
    use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};
    use std::fs::File;
    use std::io::Write;

    let mut reader = WavReader::open(wav_path).map_err(|e| {
        TranscriptionError::IoError(std::io::Error::other(format!("Failed to open WAV: {}", e)))
    })?;
    let spec = reader.spec();

    if spec.channels != 1 {
        return Err(TranscriptionError::ApiError(
            "Only mono audio is supported for MP3 conversion".to_string(),
        ));
    }
    if spec.sample_rate != 16000 {
        return Err(TranscriptionError::ApiError(
            "Only 16kHz audio is supported for MP3 conversion".to_string(),
        ));
    }

    let mp3_path = wav_path.with_extension("mp3");
    let mut mp3_file = File::create(&mp3_path).map_err(|e| {
        TranscriptionError::IoError(std::io::Error::other(format!(
            "Failed to create MP3 file: {}",
            e
        )))
    })?;

    let config = Mp3EncoderConfig::new()
        .sample_rate(spec.sample_rate)
        .bitrate(64) // 64 kbps mono is excellent quality and very compact
        .channels(1)
        .stereo_mode(StereoMode::Mono);

    let mut encoder = Mp3Encoder::new(config).map_err(|e| {
        TranscriptionError::ApiError(format!("Failed to create MP3 encoder: {:?}", e))
    })?;

    let mut pcm_buffer = Vec::new();
    for sample in reader.samples::<i16>() {
        let sample = sample.map_err(|e| {
            TranscriptionError::IoError(std::io::Error::other(format!(
                "Failed to read WAV sample: {}",
                e
            )))
        })?;
        pcm_buffer.push(sample);

        if pcm_buffer.len() == encoder.samples_per_frame() {
            let mp3_frames = encoder.encode_interleaved(&pcm_buffer).map_err(|e| {
                TranscriptionError::ApiError(format!("MP3 encoding failed: {:?}", e))
            })?;
            for frame in mp3_frames {
                mp3_file.write_all(&frame).map_err(|e| {
                    TranscriptionError::IoError(std::io::Error::other(format!(
                        "Failed to write MP3 frame: {}",
                        e
                    )))
                })?;
            }
            pcm_buffer.clear();
        }
    }

    // Flush remaining samples by padding with zeros if necessary
    if !pcm_buffer.is_empty() {
        pcm_buffer.resize(encoder.samples_per_frame(), 0);
        let mp3_frames = encoder
            .encode_interleaved(&pcm_buffer)
            .map_err(|e| TranscriptionError::ApiError(format!("MP3 encoding failed: {:?}", e)))?;
        for frame in mp3_frames {
            mp3_file.write_all(&frame).map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to write final MP3 frame: {}",
                    e
                )))
            })?;
        }
    }

    let final_data = encoder.finish().map_err(|e| {
        TranscriptionError::ApiError(format!("Failed to finalize MP3 encoder: {:?}", e))
    })?;
    mp3_file.write_all(&final_data).map_err(|e| {
        TranscriptionError::IoError(std::io::Error::other(format!(
            "Failed to write final MP3 bytes: {}",
            e
        )))
    })?;

    Ok(mp3_path)
}
