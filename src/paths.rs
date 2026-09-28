//! Portable, executable-relative path resolution.
//!
//! All mutable state (models, config, logs) lives next to `TinyTTS.exe` so the
//! whole application is a self-contained folder that can be unzipped anywhere.

use std::path::{Path, PathBuf};

use crate::error::{Result, TinyTtsError};

/// Absolute paths to every file the Kokoro model needs.
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub model: PathBuf,
    pub voices: PathBuf,
    pub tokens: PathBuf,
    pub data_dir: PathBuf,
    pub dict_dir: Option<PathBuf>,
    /// Comma separated lexicon paths, when present.
    pub lexicon: Option<String>,
    /// Comma separated zh rule FST paths, when present.
    pub rule_fsts: Option<String>,
    /// Optional `voices.txt` listing one speaker name per line.
    pub voices_txt: PathBuf,
}

/// Base-directory aware path resolver.
#[derive(Debug, Clone)]
pub struct Paths {
    base_dir: PathBuf,
}

impl Paths {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Resolve paths relative to the directory that contains the executable.
    pub fn from_exe() -> Result<Self> {
        let exe = std::env::current_exe()
            .map_err(|e| TinyTtsError::Gui(format!("cannot locate executable: {e}")))?;
        let dir = exe.parent().ok_or_else(|| {
            TinyTtsError::Gui("cannot determine executable directory".to_string())
        })?;
        Ok(Self::new(dir.to_path_buf()))
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn model_dir(&self) -> PathBuf {
        self.base_dir.join("models").join("kokoro")
    }

    pub fn config_dir(&self) -> PathBuf {
        self.base_dir.join("config")
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }

    pub fn log_file(&self) -> PathBuf {
        self.logs_dir().join("tinytts.log")
    }

    /// Validate that the Kokoro model files are present and build [`ModelPaths`].
    ///
    /// `override_dir` (typically from settings) replaces the default model dir.
    pub fn model_files(&self, override_dir: Option<&Path>) -> Result<ModelPaths> {
        let dir = match override_dir {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => self.model_dir(),
        };

        let model = dir.join("model.onnx");
        let voices = dir.join("voices.bin");
        let tokens = dir.join("tokens.txt");
        let data_dir = dir.join("espeak-ng-data");
        let dict_dir = dir.join("dict");
        let voices_txt = dir.join("voices.txt");

        let mut missing: Vec<&str> = Vec::new();
        if !model.exists() {
            missing.push("model.onnx");
        }
        if !voices.exists() {
            missing.push("voices.bin");
        }
        if !tokens.exists() {
            missing.push("tokens.txt");
        }
        if !data_dir.is_dir() {
            missing.push("espeak-ng-data/");
        }

        if !missing.is_empty() {
            return Err(TinyTtsError::ModelNotFound(format!(
                "{}\nMissing: {}\nPlease download the Kokoro model into this folder (see README.md).",
                dir.display(),
                missing.join(", ")
            )));
        }

        let dict_dir = dict_dir.is_dir().then_some(dict_dir);
        let lexicon = join_existing(
            &dir,
            &["lexicon-us-en.txt", "lexicon-zh.txt", "lexicon-gb-en.txt"],
        );
        let rule_fsts = join_existing(&dir, &["date-zh.fst", "number-zh.fst", "phone-zh.fst"]);

        Ok(ModelPaths {
            model,
            voices,
            tokens,
            data_dir,
            dict_dir,
            lexicon,
            rule_fsts,
            voices_txt,
        })
    }
}

fn join_existing(dir: &Path, names: &[&str]) -> Option<String> {
    let existing: Vec<String> = names
        .iter()
        .filter(|name| dir.join(name).exists())
        .map(|name| dir.join(name).display().to_string())
        .collect();
    if existing.is_empty() {
        None
    } else {
        Some(existing.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_base(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("tinytts-paths-{tag}-{}", std::process::id()));
        p
    }

    #[test]
    fn layout_is_relative_to_base_dir() {
        let paths = Paths::new("C:/apps/TinyTTS");
        assert_eq!(
            paths.model_dir(),
            Path::new("C:/apps/TinyTTS/models/kokoro")
        );
        assert_eq!(
            paths.config_file(),
            Path::new("C:/apps/TinyTTS/config/config.toml")
        );
        assert_eq!(
            paths.log_file(),
            Path::new("C:/apps/TinyTTS/logs/tinytts.log")
        );
    }

    #[test]
    fn missing_model_returns_model_not_found() {
        let base = temp_base("missing");
        let _ = std::fs::remove_dir_all(&base);
        let paths = Paths::new(&base);
        let err = paths.model_files(None).unwrap_err();
        assert!(matches!(err, TinyTtsError::ModelNotFound(_)));
    }

    #[test]
    fn complete_model_is_detected() {
        let base = temp_base("complete");
        let _ = std::fs::remove_dir_all(&base);
        let paths = Paths::new(&base);
        let dir = paths.model_dir();
        std::fs::create_dir_all(dir.join("espeak-ng-data")).unwrap();
        std::fs::write(dir.join("model.onnx"), b"x").unwrap();
        std::fs::write(dir.join("voices.bin"), b"x").unwrap();
        std::fs::write(dir.join("tokens.txt"), b"x").unwrap();
        std::fs::write(dir.join("lexicon-us-en.txt"), b"x").unwrap();
        std::fs::write(dir.join("lexicon-zh.txt"), b"x").unwrap();

        let mp = paths.model_files(None).unwrap();
        assert!(mp.lexicon.is_some());
        let lex = mp.lexicon.unwrap();
        assert!(lex.contains("lexicon-us-en.txt"));
        assert!(lex.contains("lexicon-zh.txt"));

        let _ = std::fs::remove_dir_all(&base);
    }
}
