mod api_transcriber;
mod azure_client;
mod client;
mod custom_endpoint_client;
mod elevenlabs_client;
mod error;
mod openai_client;
mod openrouter_transcriber;
mod options;
mod service;
mod transcriber;

// Re-export public types
pub use error::TranscriptionError;
pub use transcriber::Transcriber;
