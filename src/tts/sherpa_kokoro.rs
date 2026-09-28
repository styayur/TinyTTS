//! sherpa-onnx backed Kokoro TTS engine.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig,
    OfflineTtsModelConfig,
};

use super::engine::{AudioChunk, TtsEngine, Voice};
use crate::error::{Result, TinyTtsError};
use crate::paths::ModelPaths;

/// Canonical speaker names for `kokoro-multi-lang-v1_0` (index order matches
/// `voices.bin`). Used only as a fallback when `voices.txt` is absent.
const KOKORO_VOICES: &[&str] = &[
    "af_alloy",
    "af_aoede",
    "af_bella",
    "af_heart",
    "af_jessica",
    "af_kore",
    "af_nicole",
    "af_nova",
    "af_river",
    "af_sarah",
    "af_sky",
    "am_adam",
    "am_echo",
    "am_eric",
    "am_fenrir",
    "am_liam",
    "am_michael",
    "am_onyx",
    "am_puck",
    "am_santa",
    "bf_alice",
    "bf_emma",
    "bf_isabella",
    "bf_lily",
    "bm_daniel",
    "bm_fable",
    "bm_george",
    "bm_lewis",
    "ef_dora",
    "em_alex",
    "ff_siwis",
    "hf_alpha",
    "hf_beta",
    "hm_omega",
    "hm_psi",
    "if_sara",
    "im_nicola",
    "jf_alpha",
    "jf_gongitsune",
    "jf_nezumi",
    "jf_tebukuro",
    "jm_kumo",
    "pf_dora",
    "pm_alex",
    "pm_santa",
    "zf_xiaobei",
    "zf_xiaoni",
    "zf_xiaoxiao",
    "zf_xiaoyi",
    "zm_yunjian",
    "zm_yunxi",
    "zm_yunxia",
    "zm_yunyang",
];

/// Kokoro TTS via the official sherpa-onnx Rust API.
pub struct SherpaKokoroEngine {
    tts: Arc<OfflineTts>,
    voices: Vec<Voice>,
    sample_rate: u32,
}

impl SherpaKokoroEngine {
    pub fn new(model: &ModelPaths, num_threads: u8) -> Result<Self> {
        let config = OfflineTtsConfig {
            model: OfflineTtsModelConfig {
                kokoro: OfflineTtsKokoroModelConfig {
                    model: Some(model.model.display().to_string()),
                    voices: Some(model.voices.display().to_string()),
                    tokens: Some(model.tokens.display().to_string()),
                    data_dir: Some(model.data_dir.display().to_string()),
                    dict_dir: model.dict_dir.as_ref().map(|p| p.display().to_string()),
                    lexicon: model.lexicon.clone(),
                    length_scale: 1.0,
                    ..Default::default()
                },
                num_threads: num_threads as i32,
                debug: false,
                ..Default::default()
            },
            rule_fsts: model.rule_fsts.clone(),
            ..Default::default()
        };

        let tts = OfflineTts::create(&config).ok_or_else(|| {
            TinyTtsError::ModelLoadFailed(format!(
                "could not initialize Kokoro from {}",
                model.model.display()
            ))
        })?;

        let sample_rate = tts.sample_rate() as u32;
        let voices = load_voices(&model.voices_txt, tts.num_speakers());

        Ok(Self {
            tts: Arc::new(tts),
            voices,
            sample_rate,
        })
    }
}

impl TtsEngine for SherpaKokoroEngine {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn voices(&self) -> Vec<Voice> {
        self.voices.clone()
    }

    fn synthesize(
        &self,
        text: &str,
        voice_id: i32,
        speed: f32,
        cancel: &AtomicBool,
    ) -> Result<AudioChunk> {
        let generation = GenerationConfig {
            sid: voice_id,
            speed,
            ..Default::default()
        };

        let audio = self.tts.generate_with_config(
            text,
            &generation,
            Some(|_samples, _progress| !cancel.load(Ordering::Relaxed)),
        );

        match audio {
            Some(a) => Ok(AudioChunk {
                samples: a.samples().to_vec(),
                sample_rate: a.sample_rate() as u32,
            }),
            None => {
                if cancel.load(Ordering::Relaxed) {
                    Err(TinyTtsError::Cancelled)
                } else {
                    let preview: String = text.chars().take(40).collect();
                    Err(TinyTtsError::GenerationFailed(preview))
                }
            }
        }
    }
}

fn load_voices(voices_txt: &Path, num_speakers: i32) -> Vec<Voice> {
    let count = num_speakers.max(0) as usize;

    if let Ok(text) = std::fs::read_to_string(voices_txt) {
        let names: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();
        if !names.is_empty() {
            return names
                .into_iter()
                .take(count)
                .enumerate()
                .map(|(i, name)| Voice { id: i as i32, name })
                .collect();
        }
    }

    KOKORO_VOICES
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, name)| Voice {
            id: i as i32,
            name: (*name).to_string(),
        })
        .collect()
}
