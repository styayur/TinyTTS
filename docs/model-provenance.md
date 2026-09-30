# Kokoro model provenance

TinyTTS does not ship the Kokoro model in Git or in the application archive.

| Field | Value |
| --- | --- |
| Upstream release | `k2-fsa/sherpa-onnx` assets under tag `tts-models` |
| Asset | `kokoro-multi-lang-v1_0.tar.bz2` |
| Source URL | https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2 |
| Size | 349,906,910 bytes |
| SHA-256 | `c5f7e2d2caf082bc1d20fb70334a61d99d20b484500aad32e7cf84c128ea3298` |
| Base model | https://huggingface.co/hexgrad/Kokoro-82M |
| Runtime | sherpa-onnx / ONNX Runtime |

The upstream asset is not immutable merely because its URL is stable. Verify the SHA-256 value above before extraction. If upstream replaces the asset or the digest changes, review the new artifact before updating TinyTTS documentation and releases.

The downloaded archive includes model weights, lexicons, voices, token data, and `espeak-ng-data`. Check [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) before redistributing a full model bundle; component licenses and voice provenance may differ from the TinyTTS application license.
