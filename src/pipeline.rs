//! TTS pipeline orchestration.
//!
//! One [`Pipeline`] owns the bounded chunk queue, the TTS producer thread and
//! the audio player. Exactly one pipeline runs at a time; starting a new one
//! stops the previous one.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(feature = "native")]
use std::thread::JoinHandle;

use crate::audio::queue::{self, ChunkSender};
use crate::error::{Result, TinyTtsError};
use crate::text_segmenter;
use crate::tts::{AudioChunk, TtsEngine};

#[cfg(feature = "native")]
use crate::audio::player::Player;

#[cfg(feature = "native")]
pub struct Pipeline {
    cancel: Arc<AtomicBool>,
    producer_done: Arc<AtomicBool>,
    progress_done: Arc<AtomicUsize>,
    progress_total: Arc<AtomicUsize>,
    error: Arc<Mutex<Option<String>>>,
    player: Player,
}

#[cfg(feature = "native")]
impl Pipeline {
    /// Segment `text`, spawn the producer thread and begin playback.
    pub fn start(
        engine: Arc<dyn TtsEngine>,
        text: String,
        voice_id: i32,
        speed: f32,
    ) -> Result<Self> {
        let segments = text_segmenter::segment(&text);
        if segments.is_empty() {
            return Err(TinyTtsError::GenerationFailed(
                "no text to speak".to_string(),
            ));
        }

        let total = segments.len();
        let cancel = Arc::new(AtomicBool::new(false));
        let producer_done = Arc::new(AtomicBool::new(false));
        let progress_done = Arc::new(AtomicUsize::new(0));
        let progress_total = Arc::new(AtomicUsize::new(total));
        let error = Arc::new(Mutex::new(None::<String>));

        let (tx, rx) = queue::bounded(queue::QUEUE_CAPACITY);
        let player = Player::new()?;
        player.start_feeding(rx);

        let _ = spawn_producer(
            ProducerRequest {
                engine: Arc::clone(&engine),
                segments,
                voice_id,
                speed,
                cancel: Arc::clone(&cancel),
            },
            ProducerOutput {
                tx,
                progress_done: Arc::clone(&progress_done),
                error: Arc::clone(&error),
                producer_done: Arc::clone(&producer_done),
            },
        );

        Ok(Self {
            cancel,
            producer_done,
            progress_done,
            progress_total,
            error,
            player,
        })
    }

    pub fn cancel(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.resume();
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn stop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.player.stop();
    }

    pub fn progress_done(&self) -> usize {
        self.progress_done.load(Ordering::Relaxed)
    }

    pub fn progress_total(&self) -> usize {
        self.progress_total.load(Ordering::Relaxed)
    }

    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }

    /// True once generation finished and the player drained its queue.
    pub fn is_finished(&self) -> bool {
        self.producer_done.load(Ordering::Relaxed) && self.player.is_empty()
    }

    /// Number of chunks currently waiting to be played.
    pub fn pending(&self) -> usize {
        self.player.pending()
    }
}

#[cfg(feature = "native")]
impl Drop for Pipeline {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.player.stop();
        // The producer and feeder threads are deliberately not joined here so
        // that dropping a pipeline never blocks the UI thread. The producer
        // observes `cancel` and exits; the feeder exits when the producer drops
        // its queue sender.
    }
}

#[cfg(any(feature = "native", test))]
struct ProducerRequest {
    engine: Arc<dyn TtsEngine>,
    segments: Vec<String>,
    voice_id: i32,
    speed: f32,
    cancel: Arc<AtomicBool>,
}

#[cfg(any(feature = "native", test))]
struct ProducerOutput {
    tx: ChunkSender,
    progress_done: Arc<AtomicUsize>,
    error: Arc<Mutex<Option<String>>>,
    producer_done: Arc<AtomicBool>,
}

#[cfg(feature = "native")]
fn spawn_producer(request: ProducerRequest, output: ProducerOutput) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("tinytts-tts".to_string())
        .spawn(move || {
            run_producer(request, &output);
            output.producer_done.store(true, Ordering::Relaxed);
        })
        .expect("failed to spawn TTS thread")
}

/// Generate every segment and push the resulting chunks into the output sink.
///
/// Kept as a free function so the producer loop is unit-testable without a
/// real audio device.
#[cfg(any(feature = "native", test))]
fn run_producer(request: ProducerRequest, output: &ProducerOutput) {
    let ProducerRequest {
        engine,
        segments,
        voice_id,
        speed,
        cancel,
    } = request;
    for (i, segment) in segments.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        match engine.synthesize(&segment, voice_id, speed, &cancel) {
            Ok(chunk) => {
                if output.tx.send(chunk).is_err() {
                    // The player/consumer is gone: nothing more to do.
                    break;
                }
                output.progress_done.store(i + 1, Ordering::Relaxed);
            }
            Err(e) => {
                if !matches!(e, TinyTtsError::Cancelled) {
                    *output.error.lock().unwrap() = Some(e.to_string());
                }
                break;
            }
        }
    }
}

/// Synthesize an entire text into a single in-memory sample buffer. Used by
/// "Save WAV". The samples are concatenated in segment order.
pub fn synthesize_all(
    engine: &dyn TtsEngine,
    text: &str,
    voice_id: i32,
    speed: f32,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<f32>> {
    let mut out = Vec::new();
    for segment in text_segmenter::segment(text) {
        if cancel.load(Ordering::Relaxed) {
            return Err(TinyTtsError::Cancelled);
        }
        let chunk = engine.synthesize(&segment, voice_id, speed, cancel)?;
        out.extend_from_slice(&chunk.samples);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tts::Voice;
    use std::time::Duration;

    struct MockEngine {
        rate: u32,
        voices: Vec<Voice>,
        samples_per_char: usize,
        delay: Option<Duration>,
    }

    impl TtsEngine for MockEngine {
        fn sample_rate(&self) -> u32 {
            self.rate
        }

        fn voices(&self) -> Vec<Voice> {
            self.voices.clone()
        }

        fn synthesize(
            &self,
            text: &str,
            _voice_id: i32,
            _speed: f32,
            cancel: &Arc<AtomicBool>,
        ) -> Result<AudioChunk> {
            if let Some(delay) = self.delay {
                std::thread::sleep(delay);
            }
            if cancel.load(Ordering::Relaxed) {
                return Err(TinyTtsError::Cancelled);
            }
            let mut samples = Vec::with_capacity(text.chars().count() * self.samples_per_char);
            for ch in text.chars() {
                samples.extend(std::iter::repeat_n(
                    (ch as u32 as f32) / 128.0,
                    self.samples_per_char,
                ));
            }
            Ok(AudioChunk {
                samples,
                sample_rate: self.rate,
            })
        }
    }

    fn mock_engine() -> Arc<MockEngine> {
        Arc::new(MockEngine {
            rate: 24000,
            voices: vec![Voice {
                id: 0,
                name: "mock".to_string(),
            }],
            samples_per_char: 4,
            delay: None,
        })
    }

    #[test]
    fn mock_engine_reports_voices_and_rate() {
        let engine = mock_engine();
        assert_eq!(engine.sample_rate(), 24000);
        assert_eq!(engine.voices().len(), 1);
    }

    #[test]
    fn run_producer_emits_all_chunks() {
        let segments = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let (tx, rx) = queue::bounded(4);
        let done = Arc::new(AtomicUsize::new(0));
        let error = Arc::new(Mutex::new(None));
        let output = ProducerOutput {
            tx,
            progress_done: Arc::clone(&done),
            error: Arc::clone(&error),
            producer_done: Arc::new(AtomicBool::new(false)),
        };
        run_producer(
            ProducerRequest {
                engine: mock_engine(),
                segments,
                voice_id: 0,
                speed: 1.0,
                cancel: Arc::new(AtomicBool::new(false)),
            },
            &output,
        );
        let chunks: Vec<AudioChunk> = rx.try_iter().collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(done.load(Ordering::Relaxed), 3);
        assert!(error.lock().unwrap().is_none());
    }

    #[test]
    fn run_producer_respects_cancellation() {
        let segments = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let (tx, rx) = queue::bounded(4);
        let done = Arc::new(AtomicUsize::new(0));
        let error = Arc::new(Mutex::new(None));
        let cancel = Arc::new(AtomicBool::new(true));
        let output = ProducerOutput {
            tx,
            progress_done: Arc::clone(&done),
            error: Arc::clone(&error),
            producer_done: Arc::new(AtomicBool::new(false)),
        };
        run_producer(
            ProducerRequest {
                engine: mock_engine(),
                segments,
                voice_id: 0,
                speed: 1.0,
                cancel: Arc::clone(&cancel),
            },
            &output,
        );
        assert_eq!(done.load(Ordering::Relaxed), 0);
        assert!(error.lock().unwrap().is_none());
        drop(rx);
    }

    #[test]
    fn synthesize_all_concatenates_samples() {
        let engine = mock_engine();
        let cancel = Arc::new(AtomicBool::new(false));
        let samples = synthesize_all(&*engine, "ab.cd", 0, 1.0, &cancel).unwrap();
        // "ab.cd" -> segments ["ab.", "cd"] -> 3 + 2 chars -> 5 * 4 samples.
        assert_eq!(samples.len(), 20);
    }
}
