//! Bounded audio chunk queue.
//!
//! This is the back-pressure boundary between the TTS worker (producer) and the
//! audio player (consumer). Keeping the queue small guarantees that a fast TTS
//! engine can never race ahead and grow memory without bound on a long text.

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use crate::tts::AudioChunk;

/// Number of generated chunks allowed to wait for playback.
pub const QUEUE_CAPACITY: usize = 4;

pub type ChunkSender = SyncSender<AudioChunk>;
pub type ChunkReceiver = Receiver<AudioChunk>;

/// Create a bounded producer/consumer queue of audio chunks.
pub fn bounded(capacity: usize) -> (ChunkSender, ChunkReceiver) {
    sync_channel(capacity)
}
