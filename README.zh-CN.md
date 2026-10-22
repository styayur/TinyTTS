# TinyTTS

使用 Rust、sherpa-onnx 和 Kokoro 的 Windows 离线文字转语音工具。

[English](README.md) · [架构](docs/architecture.md) · [问题反馈](https://github.com/styayur/TinyTTS/issues)

**源码预览版。** 截至 22-10-2026，没有已发布的 GitHub Release。请从源码构建；存在发布工作流和打包步骤不代表已有下载包。

![TinyTTS 桌面窗口，包含文字输入、音色与播放控制](docs/assets/tinytts.png)

## 功能

支持中文、英文及混合文本，长文本分段，Speak / Pause / Resume / Stop，音色选择，0.5–2.0 倍速及 16-bit PCM WAV 导出。运行时可用 `Ctrl+Alt+S` 读取 Windows 剪贴板。设置保存在 EXE 旁；音频队列有界。

推理在本地运行，无账号、遥测或 HTTP 服务；运行时不需要 Python / Node.js。首次源码构建和手动下载模型需要联网。

## 构建与首次运行

需要 Windows 10/11 x64、stable Rust 的 `x86_64-pc-windows-msvc` 目标，以及 Visual Studio Build Tools 的 **Desktop development with C++** 和 Windows SDK。默认预编译原生库不能使用 GNU Rust 链接；首次原生构建由 sherpa-onnx 构建脚本下载该库。

```powershell
git clone https://github.com/styayur/TinyTTS.git
cd TinyTTS
rustc -vV
cargo build --release
```

确认 `rustc -vV` 显示 MSVC host。程序位于 `target/release/TinyTTS.exe`。进入此目录准备模型：

```powershell
Set-Location ./target/release
```

下载固定版本，**先校验哈希再解压**，并检查归档路径：

```powershell
New-Item -ItemType Directory -Force models | Out-Null
curl.exe -L -o kokoro.tar.bz2 https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2
$expected = 'C5F7E2D2CAF082BC1D20FB70334A61D99D20B484500AAD32E7CF84C128EA3298'
if ((Get-FileHash ./kokoro.tar.bz2 -Algorithm SHA256).Hash -ne $expected) { throw 'Model checksum mismatch' }
tar -tf ./kokoro.tar.bz2 | Select-Object -First 15
tar -xf ./kokoro.tar.bz2 -C ./models
```

归档内含模型目录。将解出的模型目录改名为 `models/kokoro`，不要覆盖已有模型；也可把 `model_dir` 设置为该目录的绝对路径。仅解压到 EXE 旁不会自动形成默认布局。请保留词典等完整模型内容：

```text
TinyTTS.exe
models/kokoro/
  model.onnx
  voices.bin
  tokens.txt
  espeak-ng-data/
  lexicon-us-en.txt
  lexicon-zh.txt
  dict/
config/config.toml   # created when settings are saved
```

归档大小为 349,906,910 字节，各组成部分有独立条款，见[模型来源](docs/model-provenance.md)与[第三方声明](THIRD_PARTY_NOTICES.md)。应用不会自动下载模型；缺失错误会显示搜索路径。

```powershell
.\TinyTTS.exe
```

等待 `Loading model...` 变为 `Ready`，粘贴文字后按 Speak。Pause/Resume 只控制播放；Stop 清空队列并取消生成；Save WAV 将全文生成 mono PCM WAV。全局快捷键会读取剪贴板，使用前检查其中内容。

## 配置

手动配置时，把仓库根目录的 [config.example.toml](config.example.toml) 复制到 EXE 旁的 `config/config.toml`；界面的音色和速度也可直接保存。

| 参数 | 默认值 | 含义 |
| --- | --- | --- |
| `voice` | `af_heart` | 模型中存在的音色 |
| `speed` | `1.0` | 0.5–2.0 倍速 |
| `num_threads` | `2` | 推理线程 1、2 或 4；修改后重启 |
| `model_dir` | 空 | 可选绝对路径，否则用 EXE 旁的 `models/kokoro` |

## 测试与限制

在仓库根目录运行现有 [Windows CI](https://github.com/styayur/TinyTTS/actions/workflows/ci.yml) 的检查：

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
```

`cargo test --no-default-features` 可跳过 GUI、音频和 sherpa-onnx 链接，运行无依赖核心测试。测试使用模拟引擎，不加载真实 Kokoro，不能据此判断音质或真实设备兼容性。应用支持目标为 Windows；不承诺跨平台交付。发音、语言和音色覆盖取决于所选模型。

## 贡献与后续工作

[贡献指南](CONTRIBUTING.md)包含开发及素材要求；通过 [Issues](https://github.com/styayur/TinyTTS/issues) 提交脱敏复现，漏洞按 [SECURITY.md](SECURITY.md) 处理。不要提交模型、生成音频、配置、日志或构建产物。首次正式发布验收及发音/模型改进仍是后续工作；界面音色选择已经实现。源码变更见 [CHANGELOG.md](CHANGELOG.md)。

## 许可证

应用源码为 [GPL-3.0-or-later](LICENSE)。Kokoro、音色、sherpa-onnx、ONNX Runtime 等保留各自条款，见[第三方声明](THIRD_PARTY_NOTICES.md)；应用许可证不会改变模型许可。
