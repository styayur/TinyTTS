TinyTTS
=======

Tiny, offline text-to-speech for Windows.

[![CI](https://github.com/styayur/TinyTTS/actions/workflows/ci.yml/badge.svg)](https://github.com/styayur/TinyTTS/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/styayur/TinyTTS)](https://github.com/styayur/TinyTTS/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

- Local
- No API
- No account
- No Python runtime
- Kokoro + sherpa-onnx
- Chinese / English
- Portable

Usage:

    Paste text → Speak.

TinyTTS does not send your text to any server.

---

What is TinyTTS?
----------------

TinyTTS is a minimal, fully offline text-to-speech desktop tool for Windows.
Download, unzip, double-click `TinyTTS.exe`, paste text and press **Speak**.
It runs entirely on your machine using the
[Kokoro](https://huggingface.co/hexgrad/Kokoro-82M) model through
[sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx). There is no cloud, no
login, no telemetry, no Python, no Node, no Electron and no local HTTP server.

The goal is deliberately small: it is the offline reader that feels like a
missing Windows utility, not another AI voice studio.

Features (MVP)
--------------

- Chinese, English and mixed Chinese/English text
- Long text is segmented automatically and played chunk by chunk
- Speak / Pause / Resume / Stop
- Voice selector (auto-detected from the model)
- Speed 0.5x – 2.0x
- Save WAV (standard 16-bit PCM, no FFmpeg)
- Ctrl+Alt+S global shortcut: read clipboard and speak (Windows)
- Settings persisted next to the executable (`config/config.toml`)
- Bounded audio queue so long text does not grow memory without limit

Requirements
------------

To build from source on Windows you need:

1. **Rust** (stable, MSVC target). Install with
   [rustup](https://rustup.rs) and select the `x86_64-pc-windows-msvc`
   toolchain (this is the default).
2. **Visual Studio Build Tools** (or Visual Studio) with the
   **"Desktop development with C++"** workload, which provides:

   - MSVC v143 (or later) C++ x64/x86 build tools
   - Windows 10/11 SDK

   The `sherpa-onnx` build script downloads a prebuilt native library
   automatically on the first build, so you do not need to compile it yourself,
   but you do need the MSVC linker (`link.exe`).

   > Rust's GNU toolchain (`x86_64-pc-windows-gnu`) is **not** sufficient for
   > the default static linking of the prebuilt sherpa-onnx library, which is
   > built with MSVC.

Verify your toolchain:

    rustc -vV
    # host should be x86_64-pc-windows-msvc

Download the model
------------------

Download the pinned Chinese + English Kokoro model, verify it, and extract it next to the app:

```powershell
mkdir models
curl.exe -L -o kokoro.tar.bz2 https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2
Get-FileHash .\kokoro.tar.bz2 -Algorithm SHA256
# Expected: C5F7E2D2CAF082BC1D20FB70334A61D99D20B484500AAD32E7CF84C128EA3298
tar -xf kokoro.tar.bz2
Remove-Item kokoro.tar.bz2
```

Exact source, size, license, and provenance are recorded in
[docs/model-provenance.md](docs/model-provenance.md). TinyTTS never downloads a
model at runtime.

Expected layout:

    TinyTTS/
    ├─ TinyTTS.exe
    ├─ models/
    │  └─ kokoro/
    │     ├─ model.onnx
    │     ├─ voices.bin
    │     ├─ tokens.txt
    │     ├─ espeak-ng-data/
    │     ├─ lexicon-us-en.txt
    │     ├─ lexicon-zh.txt
    │     └─ dict/
    └─ config/
       └─ config.toml   (created automatically)

If the files are missing, TinyTTS shows *"Kokoro model files were not found"*
together with the exact path it searched. It never downloads models by itself.

Build
-----

    git clone https://github.com/styayur/TinyTTS
    cd TinyTTS
    cargo build --release

The binary is `target/release/TinyTTS.exe`.

Run
---

Copy the executable next to your `models/` folder (or put `models/kokoro` into
`target/release/`), then launch:

    target\release\TinyTTS.exe

On first launch the model is loaded once in the background; the status bar shows
`Loading model...` and then `Ready`.

Core commands
-------------

- **Speak** – segment the text and start playing as soon as the first chunk is ready.
- **Pause / Resume** – pause/resume playback only.
- **Stop** – stop playback, clear the queue and cancel generation.
- **Save WAV** – generate the whole text into a 16-bit mono PCM WAV file.
- **Ctrl+Alt+S** – read the clipboard and speak it, even when minimized.

Tests
-----

    cargo fmt
    cargo clippy --all-targets
    cargo test

The full suite is light: it never loads the real Kokoro model. If you only want
the dependency-free core tests (text segmenter, settings, paths, WAV writer,
mock TTS pipeline) and do not have the native toolchain ready:

    cargo test --no-default-features

This builds with the `native` feature disabled and skips GUI/sherpa-onnx/audio
linking entirely.

Portable ZIP
------------

    cargo build --release
    mkdir dist\TinyTTS
    copy target\release\TinyTTS.exe dist\TinyTTS\
    xcopy /e /i models\kokoro dist\TinyTTS\models\kokoro
    powershell Compress-Archive -Path dist\TinyTTS\* -DestinationPath TinyTTS-win-x64.zip

Unzip `TinyTTS-win-x64.zip` anywhere and run `TinyTTS.exe`. The application
binary is a few tens of MB; the Kokoro model is counted separately (~350 MB).

Download
--------

Download the latest Windows portable ZIP from
[GitHub Releases](https://github.com/styayur/TinyTTS/releases/latest). The
archive includes `TinyTTS.exe`, configuration examples, licenses, and
`SHA256SUMS.txt`. Model files are intentionally separate because they are about
350 MB and have their own provenance and license terms.

Current support is Windows 10/11 x64. TinyTTS does not offer a cloud service,
telemetry, account system, or cross-platform build.

Release and version policy
--------------------------

Releases use semantic versions and tags such as `v0.2.0`. A release is built
only from an exact `vX.Y.Z` tag, repeats formatting/lint/tests, produces a
stable `TinyTTS-windows-x64-vX.Y.Z.zip` filename, and publishes `SHA256SUMS.txt`
alongside it. Maintainers create releases after validation; contributors do not
publish tags or assets.

Configuration
-------------

Copy `config.example.toml` to `config/config.toml` next to the executable.
Available keys:

| key          | default    | meaning                              |
|--------------|------------|--------------------------------------|
| `voice`      | `af_heart` | default voice name                   |
| `speed`      | `1.0`      | playback speed (0.5 – 2.0)           |
| `num_threads`| `2`        | ONNX Runtime threads (1, 2 or 4)     |
| `model_dir`  | *(empty)*  | optional absolute model dir override |

Voice and speed can also be changed in the UI and are saved automatically.

Project layout
--------------

    src/
    ├─ main.rs            entry point (binary)
    ├─ lib.rs             core library
    ├─ app.rs             egui window
    ├─ pipeline.rs        speak pipeline + bounded queue orchestration
    ├─ tts/               TtsEngine trait + sherpa-onnx Kokoro implementation
    ├─ audio/             rodio player, bounded queue, WAV writer
    ├─ text_segmenter.rs  long-text segmentation
    ├─ clipboard.rs       Windows clipboard read
    ├─ hotkey.rs          Ctrl+Alt+S global hotkey (reserved on non-Windows)
    ├─ settings.rs        TOML settings
    ├─ paths.rs           portable path resolution
    ├─ error.rs           error types
    └─ log.rs             tiny file logger (`logs/tinytts.log`)

Contributing
------------

Development setup, validation commands, compatibility expectations, and asset
rules are in [CONTRIBUTING.md](CONTRIBUTING.md). The core/UI/model/release
boundaries are in [docs/architecture.md](docs/architecture.md).

Security
--------

Do not report exploitable issues in a public issue. Follow
[SECURITY.md](SECURITY.md) and use GitHub private vulnerability reporting.

Community
---------

Questions, design discussions, and early feedback are welcome on
[Discord](https://discord.gg/wA2xy6VPK). Discord is an informal community
channel, not an official support or response-time guarantee. Use GitHub Issues
for reproducible bugs and scoped feature requests.

Licenses
--------

TinyTTS itself is MIT. The model and bundled native libraries have their own
licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
