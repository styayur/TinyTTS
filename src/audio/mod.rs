//! Audio playback and WAV output.

pub mod queue;
pub mod wav;

#[cfg(feature = "native")]
pub mod player;
