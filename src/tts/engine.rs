//! UI-facing TTS abstraction.
//!
//! The GUI only talks to [`TtsEngine`]. It never depends on sherpa-onnx types
//! directly, which keeps the door open for other backends in the future.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::error::Result;

/// A selectable speaker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voice {
    pub id: i32,
    pub name: String,
}

/// One chunk of synthesized audio.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioChunk {
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.samples.len() as f32 / self.sample_rate as f32
        }
    }
}

/// A minimal, synchronous TTS engine interface.
pub trait TtsEngine: Send + Sync {
    fn sample_rate(&self) -> u32;
    fn voices(&self) -> Vec<Voice>;
    fn synthesize(
        &self,
        text: &str,
        voice_id: i32,
        speed: f32,
        cancel: &Arc<AtomicBool>,
    ) -> Result<AudioChunk>;
}
