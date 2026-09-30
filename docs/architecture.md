# Architecture

TinyTTS is a small local-first Windows utility. The architecture is deliberately narrower than a voice studio.

## Core domain

- `text_segmenter.rs` splits long mixed Chinese/English text into bounded chunks.
- `tts/engine.rs` defines the `TtsEngine` boundary.
- `pipeline.rs` owns the bounded producer/audio queue, cancellation, progress, and WAV assembly.
- `audio/` owns playback, queueing, and PCM WAV writing.
- `settings.rs` and `paths.rs` resolve portable configuration and model locations.

## UI boundary

`app.rs` is an egui front end. It submits text, voice/speed settings, and cancellation signals to the core pipeline. UI code does not implement synthesis, path discovery, audio queueing, or provider-specific model logic.

## Persistence boundary

Settings live next to the executable in `config/config.toml` for portable use. Logs live under `logs/`. No cloud account, database, or opaque application store is required.

## Model and provider boundary

`sherpa_kokoro.rs` is the only native model integration. It receives explicit local paths from `ModelPaths`; TinyTTS never downloads a model implicitly. Model source, size, checksum, and licensing are recorded in [model-provenance.md](model-provenance.md).

## Extension points

- A new TTS backend implements `TtsEngine`.
- New synthesis controls belong in `GenerationConfig`/pipeline settings, not ad-hoc UI state.
- New audio formats must preserve bounded memory and cancellation behavior.
- Changes to native dependencies require Windows CI, license review, and portable-model documentation.

## Release boundary

Releases are built from exact `vX.Y.Z` tags, repeat formatting/lint/tests, produce a stable portable ZIP name, and publish `SHA256SUMS.txt`. Models remain separate from Git and source archives. Maintainers own tagging and publication.
