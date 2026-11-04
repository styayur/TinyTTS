# Kokoro model provenance

TinyTTS does not ship the Kokoro model in Git or in the application archive.

| Field | Value |
| --- | --- |
| Upstream release | `k2-fsa/sherpa-onnx` assets under tag `tts-models` |
| Asset | `kokoro-multi-lang-v1_0.tar.bz2` |
| Source URL | https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2 |
| Size | 349,886,476 bytes |
| SHA-256 | `60703707bc05ccd48d60e1eb723f38b07d6e474795a0b6a588d7220400e0f7a4` |
| Base model | https://huggingface.co/hexgrad/Kokoro-82M |
| Runtime | sherpa-onnx / ONNX Runtime |

The upstream asset is not immutable merely because its URL is stable. Verify the SHA-256 value above before extraction. If upstream replaces the asset or the digest changes, review the new artifact before updating TinyTTS documentation and releases.

The downloaded archive includes model weights, lexicons, voices, token data, and `espeak-ng-data`. Check [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) before redistributing a full model bundle; component licenses and voice provenance may differ from the TinyTTS application license.

## Reviewed replacement on 3 November 2026

The upstream same-name asset was replaced on 26 October 2026. The former archive was 349,906,910 bytes with SHA-256 `c5f7e2d2caf082bc1d20fb70334a61d99d20b484500aad32e7cf84c128ea3298`; it is a historical identity, not the current download. The reviewed current official asset is ID `677610079`, with the size and digest in the table above. Never silently accept a mismatched download.

The archive README exactly matches the official [v1.0 source](https://github.com/k2-fsa/sherpa-onnx/tree/master/scripts/kokoro/v1.0). Its generator and metadata use 54 voice embeddings of shape `510 x 1 x 256`, float32. The new Spanish `em_santa` is appended at ID 53; IDs 0 through 52 and existing voice preferences retain their names/order. The application fallback follows that mapping. The complete voice table shape and finite values were independently checked before local inference tests.

Model weights are still downloaded separately and are not redistributed in the application. The archive's Apache-2.0 model license, espeak-ng component terms and individual voice provenance obligations remain distinct; no application or component license is changed.
