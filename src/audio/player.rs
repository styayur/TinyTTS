//! Audio playback via rodio.
//!
//! A single [`Player`] owns the output stream and a [`rodio::Sink`]. The TTS
//! worker pushes chunks through the bounded queue, a feeder thread appends them
//! to the sink, and the sink plays them back in order.

use std::sync::Arc;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use super::queue::ChunkReceiver;
use crate::error::{Result, TinyTtsError};

pub struct Player {
    _stream: OutputStream,
    sink: Arc<Sink>,
}

impl Player {
    /// Open the default output device and create a ready-to-play sink.
    pub fn new() -> Result<Self> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|e| TinyTtsError::AudioDeviceUnavailable(e.to_string()))?;
        let sink = Sink::try_new(&handle)
            .map_err(|e| TinyTtsError::AudioDeviceUnavailable(e.to_string()))?;
        Ok(Self {
            _stream: stream,
            sink: Arc::new(sink),
        })
    }

    /// Spawn a feeder thread that moves chunks from the bounded queue into the
    /// sink. The thread exits when the queue is disconnected (producer done).
    pub fn start_feeding(&self, rx: ChunkReceiver) {
        let sink = Arc::clone(&self.sink);
        let _ = std::thread::Builder::new()
            .name("tinytts-player".to_string())
            .spawn(move || {
                for chunk in rx {
                    let source = SamplesBuffer::new(1, chunk.sample_rate, chunk.samples);
                    sink.append(source);
                }
            });
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Stop playback and clear any queued sounds.
    pub fn stop(&self) {
        self.sink.stop();
    }

    /// Number of chunks still queued on the sink.
    pub fn pending(&self) -> usize {
        self.sink.len()
    }

    /// True once the sink has finished playing everything it received.
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }
}
