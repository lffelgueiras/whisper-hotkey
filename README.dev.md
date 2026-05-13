# Whisper Hotkey — dev guide

## Prerequisites

- macOS 12+ (Apple Silicon) or Windows 10+ (x64)
- Rust 1.78+ (`rustup`)
- Node 20+ and pnpm 10+ (`npm i -g pnpm`)
- macOS: Xcode CLI tools (`xcode-select --install`), `brew install cmake`
- Windows: Visual Studio 2022 Build Tools (C++ workload), CMake

## Install

```bash
pnpm install
```

## Run in dev

```bash
pnpm tauri dev
```

First run will download `whisper-base.bin` (~142 MB) to your app data dir
(`~/Library/Application Support/whisper-hotkey/models/` on macOS,
`%APPDATA%\whisper-hotkey\models\` on Windows).

## Tests

- Rust: `cd src-tauri && cargo test`
- Rust integration (manual): `cd src-tauri && cargo test --test transcribe_smoke -- --ignored`
- Frontend: `pnpm test`
- E2E: `pnpm e2e`

## Common issues

- **macOS Accessibility prompt doesn't appear** — System Settings → Privacy & Security → Accessibility. Add the dev binary manually: `src-tauri/target/debug/whisper-hotkey`.
- **whisper.cpp build fails on Windows** — make sure VS Build Tools include "Desktop development with C++" and CMake is on PATH.
