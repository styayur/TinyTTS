//! Application error type.

use thiserror::Error;

/// Top-level error type used across TinyTTS.
#[derive(Debug, Error)]
pub enum TinyTtsError {
    #[error("Kokoro model files were not found. Expected path: {0}")]
    ModelNotFound(String),

    #[error("Failed to load Kokoro model: {0}")]
    ModelLoadFailed(String),

    #[error("Audio device unavailable: {0}")]
    AudioDeviceUnavailable(String),

    #[error("Clipboard unavailable: {0}")]
    ClipboardUnavailable(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Invalid voice: {0}")]
    InvalidVoice(String),

    #[error("WAV save failed: {0}")]
    WavSaveFailed(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("GUI error: {0}")]
    Gui(String),

    #[error("Cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, TinyTtsError>;
