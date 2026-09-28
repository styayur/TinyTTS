//! Text-to-speech abstraction and the sherpa-onnx Kokoro implementation.

pub mod engine;
#[cfg(feature = "native")]
pub mod sherpa_kokoro;

pub use engine::{AudioChunk, TtsEngine, Voice};
#[cfg(feature = "native")]
pub use sherpa_kokoro::SherpaKokoroEngine;
