# Contributing

TinyTTS is a small Windows desktop utility. Keep changes aligned with its core promise: tiny, offline, local-first, and easy to run.

## Development environment

- Windows 10 or 11 x64
- Rust stable with the `x86_64-pc-windows-msvc` target
- Visual Studio Build Tools with Desktop development with C++ and a Windows SDK
- The Kokoro model only when running the GUI; unit tests use mocks

```powershell
git clone https://github.com/styayur/TinyTTS.git
cd TinyTTS
cargo test --no-default-features
cargo test --all
```

For a full GUI run, follow the model layout and provenance instructions in [docs/model-provenance.md](docs/model-provenance.md).

## Before opening a pull request

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
```

- Add regression tests for behaviour changes.
- Include a screenshot or short recording for UI changes.
- Document configuration or persistence compatibility changes.
- Do not commit models, downloaded archives, generated audio, logs, `target/`, or release bundles.
- New third-party models, libraries, fonts, or data require source URL, exact version, license, and redistribution terms.
- Keep the app local-first. Do not add analytics, accounts, cloud uploads, or implicit model downloads without an explicit product decision.

## Pull requests

Clear conventional-style titles are preferred (`fix: ...`, `feat: ...`, `docs: ...`) but not mandatory. Keep each PR focused. Maintainers perform releases; contributors should not create release tags or publish assets.

Security issues must follow [SECURITY.md](SECURITY.md), not the public issue tracker.
