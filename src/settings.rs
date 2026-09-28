//! Tiny local settings, persisted as TOML next to the executable.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Result, TinyTtsError};

pub const SPEED_MIN: f32 = 0.5;
pub const SPEED_MAX: f32 = 2.0;
pub const SPEED_DEFAULT: f32 = 1.0;
pub const ALLOWED_THREADS: [u8; 3] = [1, 2, 4];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub voice: String,
    pub speed: f32,
    pub num_threads: u8,
    pub model_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            voice: "af_heart".to_string(),
            speed: SPEED_DEFAULT,
            num_threads: 2,
            model_dir: None,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| TinyTtsError::Settings(format!("cannot read {}: {e}", path.display())))?;
        let settings: Settings = toml::from_str(&text)
            .map_err(|e| TinyTtsError::Settings(format!("cannot parse {}: {e}", path.display())))?;
        Ok(settings.normalized())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self)
            .map_err(|e| TinyTtsError::Settings(format!("cannot serialize settings: {e}")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TinyTtsError::Settings(format!("cannot create {}: {e}", parent.display()))
            })?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, text)
            .map_err(|e| TinyTtsError::Settings(format!("cannot write {}: {e}", tmp.display())))?;
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        std::fs::rename(&tmp, path)
            .map_err(|e| TinyTtsError::Settings(format!("cannot write {}: {e}", path.display())))
    }

    /// Clamp invalid values so a hand-edited config can never break the app.
    pub fn normalized(mut self) -> Self {
        if !self.speed.is_finite() || self.speed < SPEED_MIN {
            self.speed = SPEED_DEFAULT;
        }
        if self.speed > SPEED_MAX {
            self.speed = SPEED_MAX;
        }
        let nearest = ALLOWED_THREADS
            .iter()
            .min_by_key(|&&n| (n as i32 - self.num_threads as i32).abs())
            .copied()
            .unwrap_or(2);
        self.num_threads = nearest;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "tinytts-settings-{tag}-{}.toml",
            std::process::id()
        ));
        p
    }

    #[test]
    fn defaults_are_sane() {
        let s = Settings::default();
        assert_eq!(s.voice, "af_heart");
        assert_eq!(s.speed, 1.0);
        assert_eq!(s.num_threads, 2);
    }

    #[test]
    fn round_trip_preserves_values() {
        let path = temp_path("roundtrip");
        let s = Settings {
            voice: "zf_xiaobei".into(),
            speed: 1.4,
            num_threads: 4,
            model_dir: None,
        };
        s.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();
        assert_eq!(loaded, s);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let path = temp_path("missing");
        let _ = std::fs::remove_file(&path);
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());
    }

    #[test]
    fn normalization_clamps_values() {
        let s = Settings {
            voice: String::new(),
            speed: 9.9,
            num_threads: 3,
            model_dir: None,
        }
        .normalized();
        assert_eq!(s.speed, SPEED_MAX);
        assert!(ALLOWED_THREADS.contains(&s.num_threads));
    }
}
