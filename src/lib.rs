//! TinyTTS core library.
//!
//! The modules here are split so the dependency-light core (text segmentation,
//! settings, paths, WAV output and the TTS engine abstraction) can be built and
//! tested without the GUI/native runtime. The `native` feature (on by default)
//! enables the sherpa-onnx Kokoro engine, audio playback and the egui window.

pub mod audio;
pub mod error;
pub mod log;
pub mod paths;
pub mod pipeline;
pub mod settings;
pub mod text_segmenter;
pub mod tts;

#[cfg(feature = "native")]
pub mod app;
#[cfg(feature = "native")]
pub mod clipboard;
#[cfg(feature = "native")]
pub mod hotkey;
