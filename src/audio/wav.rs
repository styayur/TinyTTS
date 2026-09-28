//! Minimal standard PCM WAV writer (16-bit mono), no external dependency.

use std::path::Path;

use crate::error::{Result, TinyTtsError};

/// Write normalized `f32` mono samples as a 16-bit PCM WAV file.
pub fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> Result<()> {
    if sample_rate == 0 {
        return Err(TinyTtsError::WavSaveFailed(
            "sample rate is zero".to_string(),
        ));
    }

    let data_len = (samples.len() as u32) * 2;
    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);

    // RIFF / WAVE header.
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");

    // fmt chunk (PCM, mono, 16-bit).
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
    bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk.
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for &sample in samples {
        let v = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        bytes.extend_from_slice(&v.to_le_bytes());
    }

    std::fs::write(path, &bytes)
        .map_err(|e| TinyTtsError::WavSaveFailed(format!("{}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("tinytts-wav-{tag}-{}.wav", std::process::id()));
        p
    }

    #[test]
    fn writes_valid_riff_header() {
        let path = temp_path("header");
        let samples = vec![0.0f32, 0.5, -0.5, 1.0];
        write_wav(&path, &samples, 24000).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");

        // 16-bit mono: data payload = samples * 2 bytes.
        let data_len = u32::from_le_bytes(bytes[40..44].try_into().unwrap());
        assert_eq!(data_len, 4 * 2);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn clamps_out_of_range_samples() {
        let path = temp_path("clamp");
        write_wav(&path, &[2.0f32, -2.0, 0.25], 24000).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        // Sample data starts at byte 44.
        let first = i16::from_le_bytes(bytes[44..46].try_into().unwrap());
        let second = i16::from_le_bytes(bytes[46..48].try_into().unwrap());
        assert_eq!(first, 32767);
        assert_eq!(second, -32767);
        let _ = std::fs::remove_file(&path);
    }
}
