//! Minimal, dependency-free file logger.
//!
//! TinyTTS deliberately avoids a heavyweight logging stack. Messages are
//! appended to `<exe_dir>/logs/tinytts.log`.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

/// Initialize the logger. Creates parent directories and opens the log file
/// in append mode. Failures are ignored so logging can never crash the app.
pub fn init(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = OpenOptions::new().create(true).append(true).open(path).ok();
    let _ = LOG_FILE.set(Mutex::new(file));
}

pub fn info(args: std::fmt::Arguments<'_>) {
    write_line("INFO", args);
}

pub fn error(args: std::fmt::Arguments<'_>) {
    write_line("ERROR", args);
}

fn write_line(level: &str, args: std::fmt::Arguments<'_>) {
    let line = format!("[{level}] {args}");
    if let Some(slot) = LOG_FILE.get() {
        if let Ok(mut guard) = slot.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = writeln!(file, "{line}");
                let _ = file.flush();
            }
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log::info(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log::error(format_args!($($arg)*)) };
}
