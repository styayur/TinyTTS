//! Main application window and state management.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;

use crate::audio::wav;
use crate::clipboard;
use crate::error::Result;
use crate::hotkey::GlobalHotkey;
use crate::paths::Paths;
use crate::pipeline::{self, Pipeline};
use crate::settings::{self, Settings};
use crate::tts::{SherpaKokoroEngine, TtsEngine, Voice};

pub struct TinyTtsApp {
    paths: Paths,
    settings: Settings,

    engine: Option<Arc<dyn TtsEngine>>,
    engine_rx: Option<Receiver<Result<Arc<dyn TtsEngine>>>>,
    engine_error: Option<String>,
    voices: Vec<Voice>,
    selected_voice_index: usize,
    speed: f32,

    text: String,

    pipeline: Option<Pipeline>,

    saving: bool,
    save_status: Option<String>,
    save_rx: Option<Receiver<Result<std::path::PathBuf>>>,

    hotkey: Option<GlobalHotkey>,
}

impl TinyTtsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, paths: Paths) -> Self {
        let settings = Settings::load(&paths.config_file()).unwrap_or_else(|e| {
            crate::log::error(format_args!("failed to load settings: {e}"));
            Settings::default()
        });

        let mut app = Self {
            paths,
            settings,
            engine: None,
            engine_rx: None,
            engine_error: None,
            voices: Vec::new(),
            selected_voice_index: 0,
            speed: settings::SPEED_DEFAULT,
            text: String::new(),
            pipeline: None,
            saving: false,
            save_status: None,
            save_rx: None,
            hotkey: None,
        };

        app.speed = app.settings.speed;
        app.start_engine_load();
        app.hotkey = GlobalHotkey::register();
        crate::log::info(format_args!("TinyTTS started"));

        app
    }

    fn start_engine_load(&mut self) {
        let paths = self.paths.clone();
        let settings = self.settings.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("tinytts-load-model".to_string())
            .spawn(move || {
                let result = load_engine(&paths, &settings);
                let _ = tx.send(result);
            })
            .ok();
        self.engine_rx = Some(rx);
    }

    fn poll_background(&mut self, ctx: &egui::Context) {
        // Model load completion.
        if let Some(rx) = &self.engine_rx {
            match rx.try_recv() {
                Ok(Ok(engine)) => {
                    self.voices = engine.voices();
                    self.selected_voice_index = self
                        .voices
                        .iter()
                        .position(|v| v.name == self.settings.voice)
                        .unwrap_or(0);
                    self.engine = Some(engine);
                    self.engine_rx = None;
                    crate::log::info(format_args!("model loaded ({} voices)", self.voices.len()));
                }
                Ok(Err(e)) => {
                    self.engine_error = Some(e.to_string());
                    self.engine_rx = None;
                    crate::log::error(format_args!("model load failed: {e}"));
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.engine_rx = None,
            }
        }

        // Save WAV completion.
        if let Some(rx) = &self.save_rx {
            match rx.try_recv() {
                Ok(Ok(path)) => {
                    self.saving = false;
                    self.save_status = Some(format!("Saved: {}", path.display()));
                    self.save_rx = None;
                }
                Ok(Err(e)) => {
                    self.saving = false;
                    self.save_status = Some(format!("Save failed: {e}"));
                    self.save_rx = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.saving = false;
                    self.save_rx = None;
                }
            }
        }

        // Global hotkey (Ctrl+Alt+S) -> read clipboard -> speak.
        if let Some(hotkey) = &self.hotkey {
            if hotkey.try_recv().is_some() {
                self.handle_hotkey();
            }
        }

        // Finished pipelines release their resources.
        if let Some(pipeline) = &self.pipeline {
            if pipeline.is_finished() {
                self.pipeline = None;
            }
        }

        // Keep the status line fresh while anything is happening.
        if self.pipeline.is_some() || self.engine_rx.is_some() || self.saving {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
    }

    fn handle_hotkey(&mut self) {
        match clipboard::read_text() {
            Ok(Some(text)) => {
                if self.engine.is_some() {
                    self.speak(text);
                } else {
                    self.save_status = Some("Model not ready yet.".to_string());
                }
            }
            Ok(None) => {
                crate::log::info(format_args!("hotkey: clipboard has no text"));
            }
            Err(e) => {
                self.save_status = Some(format!("Clipboard error: {e}"));
                crate::log::error(format_args!("clipboard read failed: {e}"));
            }
        }
    }

    fn speak(&mut self, text: String) {
        let engine = match &self.engine {
            Some(engine) => Arc::clone(engine),
            None => return,
        };
        let voice_id = self
            .voices
            .get(self.selected_voice_index)
            .map(|v| v.id)
            .unwrap_or(0);
        let speed = self.speed;

        // Exactly one pipeline at a time.
        if let Some(mut old) = self.pipeline.take() {
            old.stop();
        }

        match Pipeline::start(engine, text, voice_id, speed) {
            Ok(pipeline) => self.pipeline = Some(pipeline),
            Err(e) => {
                crate::log::error(format_args!("speak failed: {e}"));
                self.save_status = Some(format!("Error: {e}"));
            }
        }

        self.settings.speed = speed;
        if let Some(voice) = self.voices.get(self.selected_voice_index) {
            self.settings.voice = voice.name.clone();
        }
        self.save_settings();
    }

    fn stop(&mut self) {
        if let Some(mut pipeline) = self.pipeline.take() {
            pipeline.stop();
        }
    }

    fn pause(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline.pause();
        }
    }

    fn resume(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline.resume();
        }
    }

    fn save_wav(&mut self) {
        if self.text.trim().is_empty() {
            return;
        }
        let engine = match &self.engine {
            Some(engine) => Arc::clone(engine),
            None => return,
        };
        let voice_id = self
            .voices
            .get(self.selected_voice_index)
            .map(|v| v.id)
            .unwrap_or(0);
        let speed = self.speed;
        let text = self.text.clone();

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WAV audio", &["wav"])
            .set_file_name("tinytts.wav")
            .save_file()
        {
            self.saving = true;
            self.save_status = Some("Saving WAV...".to_string());
            let cancel = Arc::new(AtomicBool::new(false));
            let sample_rate = engine.sample_rate();
            let (tx, rx) = mpsc::channel();
            std::thread::Builder::new()
                .name("tinytts-save".to_string())
                .spawn(move || {
                    let result =
                        pipeline::synthesize_all(&*engine, &text, voice_id, speed, &cancel)
                            .and_then(|samples| {
                                wav::write_wav(&path, &samples, sample_rate).map(|_| path)
                            });
                    let _ = tx.send(result);
                })
                .ok();
            self.save_rx = Some(rx);
        }
    }

    fn save_settings(&self) {
        if let Err(e) = self.settings.save(&self.paths.config_file()) {
            crate::log::error(format_args!("failed to save settings: {e}"));
        }
    }

    fn status_text(&self) -> String {
        if let Some(e) = &self.engine_error {
            return format!("Error: {e}");
        }
        if self.engine.is_none() {
            return "Loading model...".to_string();
        }
        if self.saving {
            return self
                .save_status
                .clone()
                .unwrap_or_else(|| "Saving WAV...".to_string());
        }
        if let Some(pipeline) = &self.pipeline {
            if let Some(error) = pipeline.error() {
                return format!("Error: {error}");
            }
            let done = pipeline.progress_done();
            let total = pipeline.progress_total();
            if pipeline.is_finished() {
                return "Finished".to_string();
            }
            if pipeline.is_paused() {
                return format!("Paused {done}/{total}");
            }
            if done == 0 {
                return "Generating...".to_string();
            }
            return format!("Playing {done}/{total}");
        }
        if let Some(status) = &self.save_status {
            return status.clone();
        }
        "Ready".to_string()
    }

    fn engine_ready(&self) -> bool {
        self.engine.is_some() && self.engine_error.is_none()
    }
}

impl eframe::App for TinyTtsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TinyTTS");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text)
                        .desired_rows(10)
                        .desired_width(f32::INFINITY)
                        .hint_text("Paste text here, then click Speak."),
                );
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Voice:");
                let current = self
                    .voices
                    .get(self.selected_voice_index)
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| "Loading...".to_string());
                let mut voice_changed = false;
                egui::ComboBox::from_id_salt("voice")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for (i, voice) in self.voices.iter().enumerate() {
                            if ui
                                .selectable_value(&mut self.selected_voice_index, i, &voice.name)
                                .changed()
                            {
                                voice_changed = true;
                            }
                        }
                    });
                if voice_changed {
                    if let Some(voice) = self.voices.get(self.selected_voice_index) {
                        self.settings.voice = voice.name.clone();
                    }
                    self.save_settings();
                }

                ui.label("Speed:");
                let speed_response = ui.add(
                    egui::Slider::new(&mut self.speed, settings::SPEED_MIN..=settings::SPEED_MAX)
                        .suffix("x"),
                );
                if speed_response.changed() {
                    self.settings.speed = self.speed;
                    self.save_settings();
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let ready = self.engine_ready() && !self.saving;
                if ui.add_enabled(ready, egui::Button::new("Speak")).clicked() {
                    let text = self.text.clone();
                    self.speak(text);
                }

                let has_pipeline = self.pipeline.is_some();
                let paused = self.pipeline.as_ref().is_some_and(|p| p.is_paused());
                if ui
                    .add_enabled(has_pipeline && !paused, egui::Button::new("Pause"))
                    .clicked()
                {
                    self.pause();
                }
                if ui
                    .add_enabled(has_pipeline && paused, egui::Button::new("Resume"))
                    .clicked()
                {
                    self.resume();
                }
                if ui
                    .add_enabled(has_pipeline, egui::Button::new("Stop"))
                    .clicked()
                {
                    self.stop();
                }
                if ui
                    .add_enabled(ready, egui::Button::new("Save WAV"))
                    .clicked()
                {
                    self.save_wav();
                }
            });

            ui.separator();
            ui.label(self.status_text());
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
        if let Some(mut pipeline) = self.pipeline.take() {
            pipeline.stop();
        }
        crate::log::info(format_args!("TinyTTS exiting"));
    }
}

fn load_engine(paths: &Paths, settings: &Settings) -> Result<Arc<dyn TtsEngine>> {
    let override_dir = settings.model_dir.as_deref().map(Path::new);
    let model = paths.model_files(override_dir)?;
    let engine = SherpaKokoroEngine::new(&model, settings.num_threads)?;
    Ok(Arc::new(engine))
}
