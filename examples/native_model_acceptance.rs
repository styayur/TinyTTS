//! Real, opt-in native model acceptance with self-authored synthetic text.
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use serde::Serialize;
use tinytts::audio::wav::write_wav;
use tinytts::error::TinyTtsError;
use tinytts::paths::Paths;
use tinytts::tts::{SherpaKokoroEngine, TtsEngine};

#[derive(Serialize)]
struct Case {
    name: String,
    speaker: i32,
    speed: f32,
    frames: usize,
    sample_rate: u32,
    rms: f64,
}

#[derive(Serialize)]
struct Report {
    speakers: usize,
    missing_model_rejected: bool,
    pre_cancelled_rejected: bool,
    in_flight_cancelled: bool,
    cases: Vec<Case>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_dir = PathBuf::from(std::env::var("TINY_TTS_TEST_MODEL")?);
    let output = PathBuf::from("work/model-acceptance");
    std::fs::create_dir_all(&output)?;
    let paths = Paths::new(&output);
    assert!(paths
        .model_files(Some(&output.join("missing-model")))
        .is_err());
    let model = paths.model_files(Some(&model_dir))?;
    let engine = Arc::new(SherpaKokoroEngine::new(&model, 2)?);
    let voices = engine.voices();
    assert_eq!(voices.len(), 54);
    assert_eq!(voices[3].name, "af_heart");
    assert_eq!(voices[45].name, "zf_xiaobei");
    assert_eq!(voices[53].name, "em_santa");
    assert_eq!(engine.sample_rate(), 24000);
    let cancelled = Arc::new(AtomicBool::new(true));
    assert!(matches!(
        engine.synthesize(
            "Do not synthesize this cancelled request.",
            3,
            1.0,
            &cancelled
        ),
        Err(TinyTtsError::Cancelled)
    ));
    let cancel = Arc::new(AtomicBool::new(false));
    let mut cases = Vec::new();
    for (name, text, speaker, speed) in [
        (
            "english",
            "This is an offline speech test. One, two, three.",
            3,
            1.0,
        ),
        (
            "english-fast",
            "This is an offline speech test. One, two, three.",
            3,
            1.5,
        ),
        (
            "chinese",
            "这是一段离线语音测试，内容来自合成样例。",
            45,
            1.0,
        ),
        (
            "spanish-new-speaker",
            "Esta es una prueba de voz sin conexión.",
            53,
            1.0,
        ),
        (
            "english-unknown-names",
            "The product names are Quorvex and Zorblax.",
            3,
            1.0,
        ),
    ] {
        let audio = engine.synthesize(text, speaker, speed, &cancel)?;
        assert_eq!(audio.sample_rate, 24000);
        assert!(audio.samples.len() > 12000);
        assert!(audio.samples.iter().all(|sample| sample.is_finite()));
        let rms = (audio
            .samples
            .iter()
            .map(|sample| (*sample as f64).powi(2))
            .sum::<f64>()
            / audio.samples.len() as f64)
            .sqrt();
        assert!((0.001..0.9).contains(&rms));
        write_wav(
            &output.join(format!("{name}.wav")),
            &audio.samples,
            audio.sample_rate,
        )?;
        cases.push(Case {
            name: name.into(),
            speaker,
            speed,
            frames: audio.samples.len(),
            sample_rate: audio.sample_rate,
            rms,
        });
    }
    assert!(cases[1].frames < cases[0].frames * 9 / 10);
    let in_flight = Arc::new(AtomicBool::new(false));
    let worker_cancel = Arc::clone(&in_flight);
    let worker_engine = Arc::clone(&engine);
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let text = "A self-authored long synthetic passage exercises cancellation. ".repeat(40);
        let result = worker_engine.synthesize(&text, 3, 1.0, &worker_cancel);
        let _ = sender.send(result);
    });
    std::thread::sleep(Duration::from_millis(20));
    in_flight.store(true, Ordering::Relaxed);
    assert!(matches!(
        receiver.recv_timeout(Duration::from_secs(30))?,
        Err(TinyTtsError::Cancelled)
    ));
    // The same loaded engine must remain usable after a cancelled generation.
    let recovered =
        engine.synthesize("The engine recovered after cancellation.", 3, 1.0, &cancel)?;
    assert!(!recovered.samples.is_empty());
    let report = Report {
        speakers: voices.len(),
        missing_model_rejected: true,
        pre_cancelled_rejected: true,
        in_flight_cancelled: true,
        cases,
    };
    std::fs::write(
        output.join("native-report.toml"),
        toml::to_string_pretty(&report)?,
    )?;
    Ok(())
}
