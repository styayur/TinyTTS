//! Audio playback via rodio.
//!
//! A single [`Player`] owns the output stream and a [`rodio::Sink`]. The TTS
//! worker pushes chunks through the bounded queue, a feeder thread appends them
//! to the sink, and the sink plays them back in order.

use std::num::NonZero;
use std::sync::Arc;

use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player as RodioPlayer};

use super::queue::ChunkReceiver;
use crate::error::{Result, TinyTtsError};

pub struct Player {
    _stream: MixerDeviceSink,
    sink: Arc<RodioPlayer>,
}

impl Player {
    /// Open the default output device and create a ready-to-play sink.
    pub fn new() -> Result<Self> {
        let mut stream = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| TinyTtsError::AudioDeviceUnavailable(e.to_string()))?;
        stream.log_on_drop(false);
        let sink = RodioPlayer::connect_new(stream.mixer());
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
                    let Some(sample_rate) = NonZero::new(chunk.sample_rate) else {
                        continue;
                    };
                    let source =
                        SamplesBuffer::new(NonZero::<u16>::MIN, sample_rate, chunk.samples);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::queue;
    use crate::tts::AudioChunk;
    use std::f32::consts::TAU;
    use std::time::{Duration, Instant};

    #[test]
    #[ignore = "requires a real audio output device"]
    fn default_output_plays_pause_resume_and_clears() -> Result<()> {
        let player = Player::new()?;
        player.pause();
        assert!(player.is_paused());
        player.resume();
        assert!(!player.is_paused());

        let (tx, rx) = queue::bounded(1);
        player.start_feeding(rx);
        let sample_rate = 24_000;
        let samples = (0..sample_rate / 4)
            .map(|index| (TAU * 440.0 * index as f32 / sample_rate as f32).sin() * 0.05)
            .collect();
        tx.send(AudioChunk {
            samples,
            sample_rate,
        })
        .expect("audio queue receiver must remain active");
        drop(tx);

        let deadline = Instant::now() + Duration::from_secs(5);
        while !player.is_empty() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            player.is_empty(),
            "rodio player did not drain the queued audio"
        );
        player.stop();
        Ok(())
    }
}
