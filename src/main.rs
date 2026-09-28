//! TinyTTS entry point.
//!
//! A tiny, offline, local text-to-speech tool for Windows built on
//! Rust + sherpa-onnx + Kokoro.

#![cfg_attr(
    all(not(debug_assertions), feature = "native"),
    windows_subsystem = "windows"
)]

#[cfg(feature = "native")]
use tinytts::error::{Result, TinyTtsError};
#[cfg(feature = "native")]
use tinytts::paths::Paths;

#[cfg(feature = "native")]
fn run() -> Result<()> {
    use eframe::egui;

    let paths = Paths::from_exe()?;
    tinytts::log::init(&paths.log_file());

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 500.0])
            .with_min_inner_size([480.0, 340.0])
            .with_title("TinyTTS"),
        ..Default::default()
    };

    eframe::run_native(
        "TinyTTS",
        native_options,
        Box::new(|cc| Ok(Box::new(tinytts::app::TinyTtsApp::new(cc, paths)))),
    )
    .map_err(|e| TinyTtsError::Gui(e.to_string()))
}

#[cfg(feature = "native")]
fn main() -> Result<()> {
    run()
}

#[cfg(not(feature = "native"))]
fn main() {
    eprintln!("TinyTTS was compiled without the 'native' feature.");
    eprintln!("Run `cargo test --no-default-features` to execute the core unit tests,");
    eprintln!("or build the full GUI app with `cargo build --release` on Windows");
    eprintln!("with the MSVC Build Tools installed (see README.md).");
}
