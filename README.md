<p align="center">
  <img src="icons/whisper.png" alt="Whisper Hotkey Icon" width="128"/>
</p>

<h1 align="center">Whisper Hotkey</h1>

<p align="center">
Voice dictation for any app on macOS and Windows. Press a global hotkey, speak, press again — the transcribed text gets pasted at the cursor.
</p>

<p align="center">
100% local. No cloud, no API keys, no subscriptions. Runs on Apple Silicon (Metal) and Windows (CPU).
</p>

<p align="center">
  <em>Based on <a href="https://github.com/dpejoh/whisper-hotkey">whisper-hotkey</a> by <a href="https://github.com/dpejoh">dpejoh</a> (original Windows Python version) and the macOS Python fork by <a href="https://github.com/lffelgueiras">lffelgueiras</a>. This release is a full rewrite in Rust + TypeScript on top of <a href="https://tauri.app">Tauri</a>, shipping a single signed installer per platform.</em>
</p>

---

## Features

- **Global hotkey** — works in any app
- **Local transcription** — [whisper.cpp](https://github.com/ggml-org/whisper.cpp) via [whisper-rs](https://github.com/utilityai/whisper-rs); Metal on macOS, CPU on Windows
- **Optional LLM post-processing** — punctuation/accents fix via [llama.cpp](https://github.com/ggml-org/llama.cpp) (off by default)
- **Recording overlay** — discreet on-screen indicator
- **Transcription history** — searchable, exportable to Markdown
- **Custom vocabulary + replacement rules** (literal or regex)
- **Themes** — system / light / dark

## Install

Grab the latest installer from the [Releases page](../../releases):

- **macOS**: `.dmg` (signed and notarized; Apple Silicon)
- **Windows**: `.exe` NSIS installer (x64)

On first launch, the app walks you through permissions and an ASR model download.

## Development

See [`src-tauri/README.dev.md`](src-tauri/README.dev.md).

```bash
pnpm install
pnpm tauri dev
```

## License

MIT.
