# sherpa-onnx-sys 1.13.8 ZIP build-dependency backport

This is an explicitly local TinyTTS fork of the official Apache-2.0 crate, not a newer native engine release. `BACKPORT.json` records the verified crates.io archive identity and every original file digest; `LICENSE` and all FFI source files are preserved exactly.

The package version is `1.13.8+tinytts.zip9.1`. The code/build changes are the ZIP build dependency from 2 to exactly 9.1.3, an explicit `UPSTREAM_NATIVE_VERSION = "1.13.8"` for native archive names/URLs, and returning `Result` from the build-script entry point instead of panicking. Invalid native inputs still cause a nonzero Cargo failure. The README describes this local fork; the unused original manifest, library lock and cargo archive metadata are omitted. The root application lock is authoritative. No API, FFI or native library is upgraded or relabelled.

The ZIP API occurs only in the upstream iOS XCFramework extraction branch. Windows uses the existing hash-pinned 1.13.8 TAR archive and its original 13 static libraries, including eSpeak fallback. Updating the build dependency removes vulnerable ZIP implementation code without accepting the native 2.x regression that skips unknown English words. This does not assert that the Windows runtime exposed the iOS extraction vulnerability, or claim an actual iOS device test.

Keep the root provenance test, native inference/cancellation/WAV acceptance, native input integrity checks and existing strict native build gates. Remove this fork only after an upstream supported dependency chain and full speech compatibility have been verified.
