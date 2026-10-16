# Third-Party Notices

TinyTTS application source code is licensed under GPL-3.0-or-later.
Third-party components remain under their respective licences. The list below
is informational and is **not** a claim that every component shares the
TinyTTS application licence.

## Runtime components

### sherpa-onnx
- Project: https://github.com/k2-fsa/sherpa-onnx
- License: Apache-2.0
- Use: TTS inference (the Rust `sherpa-onnx` crate and its prebuilt native
  library, downloaded automatically by its build script).

### ONNX Runtime
- Project: https://github.com/microsoft/onnxruntime
- License: MIT
- Use: bundled inside the sherpa-onnx prebuilt native library.

### Kokoro model (kokoro-multi-lang-v1_0)
- Source: https://huggingface.co/hexgrad/Kokoro-82M
- Base model license: Apache-2.0
- Use: `model.onnx`, `voices.bin`, `tokens.txt` and lexicons. The model files
  are **not** shipped in this repository; users download them separately.
- Note: the multi-language voice pack combines voices from multiple sources.
  Verify the individual voice licenses before redistributing a model bundle.

### espeak-ng-data
- Project: https://github.com/espeak-ng/espeak-ng
- License: GPL-3.0
- Use: phoneme/grapheme data used by the Kokoro frontend at inference time.
- Note: espeak-ng is GPL-3.0. Review your own distribution obligations if you
  ship `espeak-ng-data` alongside a binary.

## Rust crates

The following are linked into the TinyTTS binary (default `native` feature).
They are listed with their licenses as declared by their crates; check each
project for full text.

| crate          | license    | purpose                                  |
|----------------|------------|------------------------------------------|
| `eframe`/`egui`| MIT/Apache-2.0 | GUI                                   |
| `rodio`        | MIT/Apache-2.0 | audio playback                       |
| `rfd`          | MIT        | native file dialogs                    |
| `sherpa-onnx`  | Apache-2.0 | TTS inference                          |
| `serde`        | MIT/Apache-2.0 | serialization                       |
| `toml`         | MIT/Apache-2.0 | config parsing                      |
| `thiserror`    | MIT/Apache-2.0 | error derive                         |
| `windows-sys`  | MIT/Apache-2.0 | Win32 FFI (clipboard, hotkey)        |

Full license texts are available from each project's repository and from the
`LICENSE` files in the Cargo registry cache after a build.

## Disclaimer

License information is provided in good faith based on public metadata at the
time of writing. It is your responsibility to verify licenses and compliance
before distributing binaries that bundle model files or native libraries.