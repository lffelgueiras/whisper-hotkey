# Whisper Hotkey — Migration to TypeScript/Tauri — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Python `whisper_hotkey.py` app with a Tauri (Rust + React/TS) app that ships a single binary for macOS Apple Silicon and Windows x64, runs 100% offline, and reaches feature parity (plus onboarding/UX improvements) with the Python version.

**Architecture:** Tauri shell. Rust backend owns hardware (audio, hotkey, paste), persistence, ASR (whisper.cpp via `whisper-rs`), and optional LLM post-processing (llama.cpp via `llama-cpp-2`). React/TS frontend renders three windows (overlay, settings, history) and talks to the backend via typed Tauri commands and events. State machine (`Idle | Recording | Transcribing`) lives in a single Rust actor; everything else is a pure function or an I/O adapter.

**Tech Stack:** Rust 1.78+, Tauri 2.x, `whisper-rs`, `llama-cpp-2`, `cpal`, `global-hotkey`, `tokio`, `serde`, `thiserror`, `ts-rs`. Frontend: TypeScript 5.x, React 18, Vite, Tailwind CSS, shadcn/ui, Zustand, Vitest, Playwright.

**Spec:** `docs/superpowers/specs/2026-05-12-migration-typescript-design.md` — read it before starting.

**Coexistence note:** The Python app (`whisper_hotkey.py`, `install.sh`, `launcher.sh`, `build-dmg.sh`, `pkg/`) stays in the repo until M10. New code goes into `src/` (frontend) and `src-tauri/` (backend). Don't touch the Python files until M10.

---

## Conventions used throughout this plan

- **TDD:** every behavioral change starts with a failing test, then minimal implementation, then verify. Configuration-only tasks (e.g., editing `tauri.conf.json`) skip the test step and use a manual verification step instead.
- **Commits:** small, frequent. Each task ends with a `git add` + `git commit`. Commit messages use Conventional Commits prefixes: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`.
- **Rust tests:** `cargo test -p whisper-hotkey <name>` from `src-tauri/`.
- **Frontend tests:** `pnpm test` (Vitest) and `pnpm e2e` (Playwright) from repo root.
- **Smoke runs:** `pnpm tauri dev` from repo root. Listed under "Smoke" steps where relevant.
- **Package manager:** `pnpm`. If the engineer prefers `npm`/`yarn`, substitute everywhere — the lock file stays consistent.

---

## File structure (target end-state)

```
whisper-hotkey-typescript/
├── package.json
├── pnpm-lock.yaml
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.ts
├── postcss.config.js
├── index.html                       # Vite entry, root window
├── overlay.html                     # Vite entry, overlay window
├── history.html                     # Vite entry, history window
├── .github/workflows/
│   ├── ci.yml                       # build+test on mac+win
│   └── release.yml                  # tag → signed installers
│
├── src/                             # frontend
│   ├── main.tsx                     # root entry (settings + onboarding)
│   ├── overlay-main.tsx             # overlay entry
│   ├── history-main.tsx             # history entry
│   ├── App.tsx                      # router for root window
│   ├── windows/
│   │   ├── settings/
│   │   │   ├── SettingsWindow.tsx
│   │   │   ├── GeneralTab.tsx
│   │   │   ├── ModelTab.tsx
│   │   │   └── VocabularyTab.tsx
│   │   ├── overlay/
│   │   │   └── OverlayWindow.tsx
│   │   ├── history/
│   │   │   └── HistoryWindow.tsx
│   │   └── onboarding/
│   │       ├── OnboardingWindow.tsx
│   │       └── steps/
│   │           ├── WelcomeStep.tsx
│   │           ├── PermissionsStep.tsx
│   │           └── ModelDownloadStep.tsx
│   ├── components/
│   │   ├── ui/                      # shadcn primitives
│   │   ├── HotkeyCapture.tsx
│   │   ├── ModelCard.tsx
│   │   ├── WaveformPreview.tsx
│   │   └── ReplacementEditor.tsx
│   ├── store/
│   │   ├── configStore.ts
│   │   ├── recordingStore.ts
│   │   └── historyStore.ts
│   ├── ipc/
│   │   ├── commands.ts              # typed wrappers around invoke()
│   │   ├── events.ts                # typed wrappers around listen()
│   │   └── generated/               # ts-rs output
│   ├── lib/
│   │   ├── theme.ts
│   │   └── format.ts
│   └── styles/
│       └── globals.css
│
├── src-tauri/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/                       # copy from /icons
│   └── src/
│       ├── main.rs
│       ├── app_state.rs             # state machine actor
│       ├── commands.rs              # #[tauri::command] surface
│       ├── events.rs                # typed event emitters
│       ├── error.rs                 # AppError + From impls
│       ├── hotkey.rs
│       ├── audio.rs
│       ├── asr/
│       │   ├── mod.rs               # Transcriber trait
│       │   ├── whisper_cpp.rs
│       │   └── vocabulary.rs
│       ├── llm/
│       │   ├── mod.rs               # PostProcessor trait
│       │   └── llama_cpp.rs
│       ├── paste/
│       │   ├── mod.rs               # Paster trait
│       │   ├── macos.rs
│       │   └── windows.rs
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── config.rs
│       │   └── history.rs
│       ├── models.rs                # downloader + manifest
│       ├── replacements.rs
│       └── logging.rs
│
├── tests/                           # Playwright e2e
│   ├── settings.spec.ts
│   └── onboarding.spec.ts
│
├── icons/                           # existing — reused
│
├── docs/superpowers/
│   ├── specs/2026-05-12-migration-typescript-design.md
│   └── plans/2026-05-12-migration-typescript.md      (this file)
│
└── (Python files — untouched until M10)
    ├── whisper_hotkey.py
    ├── install.sh
    ├── launcher.sh
    ├── build-dmg.sh
    └── pkg/
```

---

# Milestone M0 — Scaffold

**Goal of milestone:** A Tauri 2 project boots on macOS and Windows, shows a tray icon, has CI green for build + lint + test on both platforms. No app logic yet.

**Done when:** `pnpm tauri dev` opens a hidden-window app with a tray icon labeled "Whisper Hotkey" and a single "Quit" menu item; `pnpm test` and `cargo test` both pass; CI is green on both runners.

---

### Task M0.1: Initialize Tauri project scaffold

**Files:**
- Create: `package.json`, `pnpm-lock.yaml`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/build.rs`
- Modify: `.gitignore`

- [x] **Step 1: Run the Tauri scaffolder**

Run from repo root:
```bash
pnpm create tauri-app@latest .
```

When prompted:
- Project name: `whisper-hotkey`
- Identifier: `com.lucabe.whisperhotkey`
- Frontend: TypeScript / JavaScript
- Package manager: pnpm
- UI template: React
- UI flavor: TypeScript

Accept overwriting `.gitignore` only after diffing — preserve existing entries.

- [x] **Step 2: Verify it builds**

Run:
```bash
pnpm install
pnpm tauri dev
```

Expected: A window opens showing the default Tauri welcome page. Close it.

- [x] **Step 3: Add the project's `.gitignore` entries back**

Ensure `.gitignore` contains (merge with whatever the scaffolder produced):
```
node_modules/
dist/
src-tauri/target/
src-tauri/Cargo.lock        # keep Cargo.lock? remove this line if you want it tracked
.DS_Store
*.log
.env
```

Decision: **keep `Cargo.lock` tracked** for reproducible builds. Remove the `Cargo.lock` line from `.gitignore`.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(m0): scaffold tauri 2 project (react + ts + pnpm)"
```

---

### Task M0.2: Pin versions and add core dev dependencies

**Files:**
- Modify: `package.json`, `src-tauri/Cargo.toml`

- [x] **Step 1: Pin Node deps**

Edit `package.json` `devDependencies` to add:
```json
{
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@tauri-apps/api": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0",
    "vitest": "^2.1.0",
    "@testing-library/react": "^16.0.0",
    "@testing-library/jest-dom": "^6.5.0",
    "jsdom": "^25.0.0",
    "@playwright/test": "^1.48.0",
    "eslint": "^9.10.0",
    "@typescript-eslint/parser": "^8.6.0",
    "@typescript-eslint/eslint-plugin": "^8.6.0",
    "prettier": "^3.3.0"
  }
}
```

Run:
```bash
pnpm install
```

- [x] **Step 2: Pin Rust deps**

Edit `src-tauri/Cargo.toml` `[dependencies]`:
```toml
[dependencies]
tauri = { version = "2", features = [ "tray-icon" ] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
ts-rs = "10"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [x] **Step 3: Verify build still works**

Run:
```bash
pnpm tauri dev
```

Expected: Window opens, tray icon API available (tray feature compiled in). Close.

- [x] **Step 4: Commit**

```bash
git add package.json pnpm-lock.yaml src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore(m0): pin core deps (tauri 2, vitest, playwright, ts-rs)"
```

---

### Task M0.3: Set up Tailwind CSS and shadcn/ui

**Files:**
- Create: `tailwind.config.ts`, `postcss.config.js`, `src/styles/globals.css`, `components.json`
- Modify: `src/main.tsx`, `package.json`

- [x] **Step 1: Install Tailwind**

```bash
pnpm add -D tailwindcss postcss autoprefixer
pnpm dlx tailwindcss init -p
```

- [x] **Step 2: Configure Tailwind**

Replace `tailwind.config.ts` with:
```ts
import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./overlay.html", "./history.html", "./src/**/*.{ts,tsx}"],
  darkMode: ["class"],
  theme: {
    extend: {
      colors: {
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        muted: "hsl(var(--muted))",
        accent: "hsl(var(--accent))",
        border: "hsl(var(--border))",
      },
    },
  },
  plugins: [],
} satisfies Config;
```

- [x] **Step 3: Create globals.css**

Create `src/styles/globals.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 240 10% 4%;
    --muted: 240 5% 96%;
    --accent: 211 100% 50%;
    --border: 240 6% 90%;
  }
  .dark {
    --background: 240 6% 10%;
    --foreground: 0 0% 95%;
    --muted: 240 4% 16%;
    --accent: 211 100% 60%;
    --border: 240 4% 20%;
  }
  body { @apply bg-background text-foreground antialiased; }
}
```

- [x] **Step 4: Import globals.css**

Replace `src/main.tsx`:
```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

- [x] **Step 5: Initialize shadcn/ui**

```bash
pnpm dlx shadcn@latest init
```

When prompted: TypeScript, Slate base color, `src/styles/globals.css`, `tailwind.config.ts`, alias `@/*` → `src/*`.

This creates `components.json` and `src/components/ui/` (initially empty).

- [x] **Step 6: Add `@/` path alias to tsconfig and vite**

Edit `tsconfig.json` `compilerOptions`:
```json
{
  "baseUrl": ".",
  "paths": { "@/*": ["src/*"] }
}
```

Edit `vite.config.ts`:
```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react()],
  resolve: { alias: { "@": path.resolve(__dirname, "src") } },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
```

- [x] **Step 7: Verify**

```bash
pnpm tauri dev
```

Expected: window opens, Tailwind base styles apply (sans-serif font, background color).

- [x] **Step 8: Commit**

```bash
git add -A
git commit -m "chore(m0): set up tailwind + shadcn/ui"
```

---

### Task M0.4: Add Tauri tray with Quit menu

**Files:**
- Modify: `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`

- [x] **Step 1: Configure tray in tauri.conf.json**

Edit `src-tauri/tauri.conf.json`:
```json
{
  "app": {
    "trayIcon": {
      "id": "main",
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true,
      "menuOnLeftClick": true,
      "tooltip": "Whisper Hotkey"
    },
    "windows": [
      {
        "label": "main",
        "title": "Whisper Hotkey",
        "width": 800,
        "height": 600,
        "visible": false,
        "decorations": true,
        "resizable": true
      }
    ]
  }
}
```

Copy `icons/whisper.png` to `src-tauri/icons/icon.png` (or generate platform icons via `pnpm tauri icon icons/whisper.png`).

```bash
pnpm tauri icon icons/whisper.png
```

- [x] **Step 2: Build tray menu in main.rs**

Replace `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit]).build()?;

            let _tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [x] **Step 3: Smoke test**

```bash
pnpm tauri dev
```

Expected: No visible window opens. A tray icon appears (top-right on macOS, system tray on Windows). Clicking it shows "Quit". Clicking Quit exits.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m0): add system tray with quit menu, hide main window on launch"
```

---

### Task M0.5: Add `tracing` logging bootstrap

**Files:**
- Create: `src-tauri/src/logging.rs`
- Modify: `src-tauri/src/main.rs`

- [x] **Step 1: Create logging module**

Create `src-tauri/src/logging.rs`:
```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,whisper_hotkey=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_thread_ids(false))
        .init();
    tracing::info!("logging initialized");
}
```

- [x] **Step 2: Wire into main**

Edit `src-tauri/src/main.rs` — add `mod logging;` near the top and call `logging::init();` as the first line of `main()`.

- [x] **Step 3: Verify logs appear**

```bash
pnpm tauri dev
```

Expected: Console shows `INFO logging initialized` before tray appears.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(m0): add tracing-based logging"
```

---

### Task M0.6: CI workflow — build + test on macOS and Windows

**Files:**
- Create: `.github/workflows/ci.yml`

- [x] **Step 1: Write the workflow**

Create `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        os: [macos-14, windows-2022]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: "src-tauri" }
      - name: Install macOS deps
        if: runner.os == 'macOS'
        run: brew install create-dmg
      - name: pnpm install
        run: pnpm install --frozen-lockfile
      - name: Frontend lint
        run: pnpm lint || true   # remove "|| true" once eslint is configured
      - name: Frontend test
        run: pnpm test --run
      - name: Rust fmt
        working-directory: src-tauri
        run: cargo fmt --check
      - name: Rust clippy
        working-directory: src-tauri
        run: cargo clippy --all-targets -- -D warnings
      - name: Rust test
        working-directory: src-tauri
        run: cargo test
      - name: Tauri build (debug only on CI)
        run: pnpm tauri build --debug
```

- [x] **Step 2: Add lint script**

Edit `package.json` `scripts`:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri",
    "test": "vitest",
    "e2e": "playwright test",
    "lint": "eslint src --ext .ts,.tsx",
    "format": "prettier --write \"src/**/*.{ts,tsx,css}\""
  }
}
```

- [x] **Step 3: Add a placeholder Vitest test so the test step passes**

Create `src/lib/__tests__/placeholder.test.ts`:
```ts
import { describe, it, expect } from "vitest";

describe("placeholder", () => {
  it("smoke", () => {
    expect(1 + 1).toBe(2);
  });
});
```

Add to `vite.config.ts`:
```ts
// @ts-expect-error vitest extends vite config at runtime
test: { environment: "jsdom", globals: false },
```

- [x] **Step 4: Commit and push**

```bash
git add -A
git commit -m "ci(m0): build+test matrix on macos and windows"
git push
```

Verify the CI run goes green on both runners. If clippy/fmt fails, fix and re-push.

---

# Milestone M1 — Pipeline core (hotkey → audio → ASR → paste)

**Goal of milestone:** Press a hardcoded hotkey, the app records mono 16 kHz audio; press it again, audio is transcribed with whisper.cpp (using the smallest model, downloaded once), the result is copied to clipboard and pasted into the focused app.

**Done when:** Manual smoke: type into a text field of any other app, press `Cmd+Shift+Space` (or `Ctrl+Shift+Space` on Windows), speak "hello world", press again — "hello world" is pasted. Rust tests cover audio buffer ops, vocabulary builder, state machine transitions, and a transcription smoke test that runs against `whisper-tiny` on a 1-second sine-wave-prefaced sample.

---

### Task M1.1: Define `AppError` and `ErrorKind`

**Files:**
- Create: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/main.rs`

- [x] **Step 1: Write the failing test**

Create `src-tauri/src/error.rs`:
```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub enum ErrorKind {
    Mic,
    Model,
    Paste,
    Asr,
    Llm,
    Storage,
    Hotkey,
    Permission,
    Internal,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("microphone error: {0}")]
    Mic(String),
    #[error("model error: {0}")]
    Model(String),
    #[error("paste error: {0}")]
    Paste(String),
    #[error("asr error: {0}")]
    Asr(String),
    #[error("llm error: {0}")]
    Llm(String),
    #[error("storage error: {0}")]
    Storage(#[from] std::io::Error),
    #[error("hotkey error: {0}")]
    Hotkey(String),
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct AppErrorDto {
    pub kind: ErrorKind,
    pub message: String,
    pub recoverable: bool,
}

impl AppError {
    pub fn to_dto(&self) -> AppErrorDto {
        let (kind, recoverable) = match self {
            AppError::Mic(_) => (ErrorKind::Mic, true),
            AppError::Model(_) => (ErrorKind::Model, true),
            AppError::Paste(_) => (ErrorKind::Paste, true),
            AppError::Asr(_) => (ErrorKind::Asr, true),
            AppError::Llm(_) => (ErrorKind::Llm, true),
            AppError::Storage(_) => (ErrorKind::Storage, false),
            AppError::Hotkey(_) => (ErrorKind::Hotkey, false),
            AppError::Permission(_) => (ErrorKind::Permission, true),
            AppError::Internal(_) => (ErrorKind::Internal, false),
        };
        AppErrorDto { kind, message: self.to_string(), recoverable }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mic_error_is_recoverable() {
        let err = AppError::Mic("no device".into());
        let dto = err.to_dto();
        assert!(matches!(dto.kind, ErrorKind::Mic));
        assert!(dto.recoverable);
    }

    #[test]
    fn storage_error_is_not_recoverable() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "x");
        let err: AppError = io.into();
        assert!(!err.to_dto().recoverable);
    }
}
```

- [x] **Step 2: Hook into main**

Add `mod error;` to `src-tauri/src/main.rs`.

- [x] **Step 3: Run tests**

```bash
cd src-tauri && cargo test error::
```

Expected: 2 passed.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m1): add AppError + DTO with thiserror + ts-rs"
```

---

### Task M1.2: Audio capture module — start/stop/get-samples

**Files:**
- Create: `src-tauri/src/audio.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`

- [x] **Step 1: Add cpal**

Append to `src-tauri/Cargo.toml`:
```toml
cpal = "0.15"
rubato = "0.15"   # resampling to 16kHz
parking_lot = "0.12"
```

- [x] **Step 2: Write failing tests**

Create `src-tauri/src/audio.rs`:
```rust
use crate::error::AppError;
use parking_lot::Mutex;
use std::sync::Arc;

/// Resamples interleaved or mono samples from `from_hz` to 16000 Hz mono.
pub fn resample_to_16k_mono(samples: &[f32], from_hz: u32, channels: u16) -> Vec<f32> {
    let mono: Vec<f32> = if channels == 1 {
        samples.to_vec()
    } else {
        samples
            .chunks_exact(channels as usize)
            .map(|c| c.iter().sum::<f32>() / channels as f32)
            .collect()
    };
    if from_hz == 16_000 {
        return mono;
    }
    let ratio = 16_000f64 / from_hz as f64;
    let out_len = ((mono.len() as f64) * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let idx = src.floor() as usize;
        let frac = src - idx as f64;
        let a = mono.get(idx).copied().unwrap_or(0.0);
        let b = mono.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac as f32);
    }
    out
}

pub struct AudioCapturer {
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Mutex<Option<cpal::Stream>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<u16>>,
}

impl AudioCapturer {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: Mutex::new(None),
            sample_rate: Arc::new(Mutex::new(16_000)),
            channels: Arc::new(Mutex::new(1)),
        }
    }

    pub fn start(&self) -> Result<(), AppError> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Mic("no default input device".into()))?;
        let config = device
            .default_input_config()
            .map_err(|e| AppError::Mic(format!("default config: {e}")))?;

        *self.sample_rate.lock() = config.sample_rate().0;
        *self.channels.lock() = config.channels();
        self.buffer.lock().clear();

        let buf = self.buffer.clone();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| buf.lock().extend_from_slice(data),
                move |err| tracing::error!("audio stream err: {err}"),
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let mut g = buf.lock();
                    for &s in data {
                        g.push(s as f32 / i16::MAX as f32);
                    }
                },
                move |err| tracing::error!("audio stream err: {err}"),
                None,
            ),
            other => {
                return Err(AppError::Mic(format!("unsupported sample format: {other:?}")))
            }
        }
        .map_err(|e| AppError::Mic(format!("build stream: {e}")))?;
        stream.play().map_err(|e| AppError::Mic(format!("play: {e}")))?;
        *self.stream.lock() = Some(stream);
        Ok(())
    }

    /// Stops the stream and returns mono 16kHz samples.
    pub fn stop(&self) -> Result<Vec<f32>, AppError> {
        drop(self.stream.lock().take());
        let raw = std::mem::take(&mut *self.buffer.lock());
        let hz = *self.sample_rate.lock();
        let ch = *self.channels.lock();
        Ok(resample_to_16k_mono(&raw, hz, ch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_when_already_16k_mono() {
        let s = vec![0.1, 0.2, 0.3, 0.4];
        let out = resample_to_16k_mono(&s, 16_000, 1);
        assert_eq!(out, s);
    }

    #[test]
    fn downsamples_48k_mono_to_16k() {
        let s: Vec<f32> = (0..480).map(|i| (i as f32) / 480.0).collect();
        let out = resample_to_16k_mono(&s, 48_000, 1);
        assert!(out.len() >= 158 && out.len() <= 162, "got {}", out.len());
        // monotonically increasing roughly
        for w in out.windows(2) {
            assert!(w[1] >= w[0] - 0.01);
        }
    }

    #[test]
    fn mixes_stereo_to_mono() {
        let s = vec![1.0, -1.0, 0.5, -0.5];
        let out = resample_to_16k_mono(&s, 16_000, 2);
        assert_eq!(out, vec![0.0, 0.0]);
    }
}
```

- [x] **Step 3: Run tests**

```bash
cd src-tauri && cargo test audio::
```

Expected: 3 passed.

- [x] **Step 4: Wire module**

Add `mod audio;` to `main.rs`.

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m1): audio capture via cpal + linear resample to 16kHz mono"
```

---

### Task M1.3: Vocabulary builder

**Files:**
- Create: `src-tauri/src/asr/mod.rs`, `src-tauri/src/asr/vocabulary.rs`

- [x] **Step 1: Write tests**

Create `src-tauri/src/asr/vocabulary.rs`:
```rust
/// Builds a Whisper `initial_prompt` string from a vocabulary list.
/// Returns None if the vocab is empty.
pub fn build_initial_prompt(vocab: &[String]) -> Option<String> {
    let cleaned: Vec<&str> = vocab
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    Some(format!("Glossário: {}.", cleaned.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_vocab_returns_none() {
        assert!(build_initial_prompt(&[]).is_none());
    }

    #[test]
    fn whitespace_only_returns_none() {
        let v = vec!["  ".into(), "".into()];
        assert!(build_initial_prompt(&v).is_none());
    }

    #[test]
    fn formats_words() {
        let v = vec!["LUCABE".into(), "Ploomes".into()];
        assert_eq!(
            build_initial_prompt(&v),
            Some("Glossário: LUCABE, Ploomes.".to_string())
        );
    }
}
```

Create `src-tauri/src/asr/mod.rs`:
```rust
pub mod vocabulary;
```

Add `mod asr;` to `main.rs`.

- [x] **Step 2: Run tests**

```bash
cd src-tauri && cargo test vocabulary::
```

Expected: 3 passed.

- [x] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m1): vocabulary initial_prompt builder"
```

---

### Task M1.4: `Transcriber` trait + whisper.cpp implementation

**Files:**
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/asr/mod.rs`
- Create: `src-tauri/src/asr/whisper_cpp.rs`

- [x] **Step 1: Add whisper-rs**

Append to `src-tauri/Cargo.toml`:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
whisper-rs = { version = "0.13", features = ["metal"] }

[target.'cfg(target_os = "windows")'.dependencies]
whisper-rs = { version = "0.13", default-features = false }
```

(CUDA support deferred to a later milestone — Windows starts CPU-only as noted in spec §14.)

- [x] **Step 2: Define the trait**

Replace `src-tauri/src/asr/mod.rs`:
```rust
pub mod vocabulary;
pub mod whisper_cpp;

use crate::error::AppError;

pub trait Transcriber: Send + Sync {
    fn transcribe(&self, samples: &[f32], vocab: &[String]) -> Result<String, AppError>;
}
```

- [x] **Step 3: Implement WhisperCpp**

Create `src-tauri/src/asr/whisper_cpp.rs`:
```rust
use super::{vocabulary, Transcriber};
use crate::error::AppError;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperCpp {
    ctx: WhisperContext,
}

impl WhisperCpp {
    pub fn load(model_path: &Path) -> Result<Self, AppError> {
        let path = model_path
            .to_str()
            .ok_or_else(|| AppError::Model("non-utf8 model path".into()))?;
        let ctx = WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .map_err(|e| AppError::Model(format!("load whisper model: {e}")))?;
        Ok(Self { ctx })
    }
}

impl Transcriber for WhisperCpp {
    fn transcribe(&self, samples: &[f32], vocab: &[String]) -> Result<String, AppError> {
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| AppError::Asr(format!("create state: {e}")))?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_translate(false);
        params.set_language(Some("auto"));
        let prompt = vocabulary::build_initial_prompt(vocab);
        if let Some(p) = prompt.as_deref() {
            params.set_initial_prompt(p);
        }
        state
            .full(params, samples)
            .map_err(|e| AppError::Asr(format!("run: {e}")))?;
        let n = state
            .full_n_segments()
            .map_err(|e| AppError::Asr(format!("n_segments: {e}")))?;
        let mut out = String::new();
        for i in 0..n {
            let seg = state
                .full_get_segment_text(i)
                .map_err(|e| AppError::Asr(format!("seg {i}: {e}")))?;
            out.push_str(&seg);
        }
        Ok(out.trim().to_string())
    }
}
```

- [x] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: clean compile (no tests yet — model integration test comes in M1.10).

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m1): Transcriber trait + whisper.cpp impl (metal on mac, cpu on win)"
```

---

### Task M1.5: Paste module — macOS

**Files:**
- Create: `src-tauri/src/paste/mod.rs`, `src-tauri/src/paste/macos.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`

- [x] **Step 1: Add deps**

Append to `src-tauri/Cargo.toml`:
```toml
arboard = "3.4"

[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.24"
core-foundation = "0.10"
```

- [x] **Step 2: Define the trait**

Create `src-tauri/src/paste/mod.rs`:
```rust
use crate::error::AppError;

pub trait Paster: Send + Sync {
    fn paste(&self, text: &str) -> Result<(), AppError>;
}

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub fn default_paster() -> Box<dyn Paster> {
    Box::new(macos::MacPaster::new())
}

#[cfg(target_os = "windows")]
pub fn default_paster() -> Box<dyn Paster> {
    Box::new(windows::WinPaster::new())
}
```

- [x] **Step 3: Implement macOS paste**

Create `src-tauri/src/paste/macos.rs`:
```rust
use super::Paster;
use crate::error::AppError;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

const KEY_V: CGKeyCode = 9; // ANSI 'V'

pub struct MacPaster;

impl MacPaster {
    pub fn new() -> Self { Self }
}

impl Paster for MacPaster {
    fn paste(&self, text: &str) -> Result<(), AppError> {
        let mut cb = arboard::Clipboard::new()
            .map_err(|e| AppError::Paste(format!("clipboard: {e}")))?;
        cb.set_text(text.to_string())
            .map_err(|e| AppError::Paste(format!("set clipboard: {e}")))?;
        let src = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| AppError::Paste("CGEventSource".into()))?;
        let down = CGEvent::new_keyboard_event(src.clone(), KEY_V, true)
            .map_err(|_| AppError::Paste("key down".into()))?;
        down.set_flags(CGEventFlags::CGEventFlagCommand);
        let up = CGEvent::new_keyboard_event(src, KEY_V, false)
            .map_err(|_| AppError::Paste("key up".into()))?;
        up.set_flags(CGEventFlags::CGEventFlagCommand);
        down.post(CGEventTapLocation::HID);
        up.post(CGEventTapLocation::HID);
        Ok(())
    }
}
```

- [x] **Step 4: Stub Windows paste so the code compiles cross-platform**

Create `src-tauri/src/paste/windows.rs`:
```rust
use super::Paster;
use crate::error::AppError;

pub struct WinPaster;

impl WinPaster {
    pub fn new() -> Self { Self }
}

impl Paster for WinPaster {
    fn paste(&self, _text: &str) -> Result<(), AppError> {
        Err(AppError::Paste("windows paster not yet implemented".into()))
    }
}
```

- [x] **Step 5: Add `mod paste;` to main.rs**

- [x] **Step 6: Compile check**

```bash
cd src-tauri && cargo check
```

Expected: clean compile on whichever platform you're on.

- [x] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(m1): paste module — macos cmd+v via CGEvent, win stubbed"
```

---

### Task M1.6: Paste module — Windows

**Files:**
- Modify: `src-tauri/src/paste/windows.rs`, `src-tauri/Cargo.toml`

- [x] **Step 1: Add windows crate**

Append to `src-tauri/Cargo.toml`:
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = ["Win32_UI_Input_KeyboardAndMouse", "Win32_Foundation"] }
```

- [x] **Step 2: Implement Windows paste**

Replace `src-tauri/src/paste/windows.rs`:
```rust
use super::Paster;
use crate::error::AppError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};

pub struct WinPaster;

impl WinPaster {
    pub fn new() -> Self { Self }
}

fn key_event(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if up { flags |= KEYEVENTF_KEYUP; }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

impl Paster for WinPaster {
    fn paste(&self, text: &str) -> Result<(), AppError> {
        let mut cb = arboard::Clipboard::new()
            .map_err(|e| AppError::Paste(format!("clipboard: {e}")))?;
        cb.set_text(text.to_string())
            .map_err(|e| AppError::Paste(format!("set clipboard: {e}")))?;
        let inputs = [
            key_event(VK_CONTROL, false),
            key_event(VK_V, false),
            key_event(VK_V, true),
            key_event(VK_CONTROL, true),
        ];
        let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent as usize != inputs.len() {
            return Err(AppError::Paste(format!(
                "SendInput sent {sent}/{}",
                inputs.len()
            )));
        }
        Ok(())
    }
}
```

- [x] **Step 3: Compile check on Windows runner**

(If developing on macOS, push and let CI verify.)

```bash
cd src-tauri && cargo check --target x86_64-pc-windows-msvc
```
or rely on CI.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m1): windows paste via SendInput (ctrl+v)"
```

---

### Task M1.7: State machine actor

**Files:**
- Create: `src-tauri/src/app_state.rs`
- Modify: `src-tauri/src/main.rs`

- [x] **Step 1: Write tests**

Create `src-tauri/src/app_state.rs`:
```rust
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}

#[derive(Debug, Clone, Copy)]
pub enum Intent {
    Toggle,    // hotkey pressed
    Done,      // transcription finished
    Failed,    // transcription failed
}

pub fn next(state: RecordingState, intent: Intent) -> RecordingState {
    use Intent::*;
    use RecordingState::*;
    match (state, intent) {
        (Idle, Toggle) => Recording,
        (Recording, Toggle) => Transcribing,
        (Transcribing, Toggle) => Transcribing, // ignore
        (Transcribing, Done) => Idle,
        (Transcribing, Failed) => Idle,
        (other, _) => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_to_recording_on_toggle() {
        assert_eq!(next(RecordingState::Idle, Intent::Toggle), RecordingState::Recording);
    }

    #[test]
    fn recording_to_transcribing_on_toggle() {
        assert_eq!(next(RecordingState::Recording, Intent::Toggle), RecordingState::Transcribing);
    }

    #[test]
    fn toggle_during_transcription_is_ignored() {
        assert_eq!(next(RecordingState::Transcribing, Intent::Toggle), RecordingState::Transcribing);
    }

    #[test]
    fn done_returns_to_idle() {
        assert_eq!(next(RecordingState::Transcribing, Intent::Done), RecordingState::Idle);
        assert_eq!(next(RecordingState::Transcribing, Intent::Failed), RecordingState::Idle);
    }
}
```

- [x] **Step 2: Run tests**

```bash
cd src-tauri && cargo test app_state::
```

Expected: 4 passed.

- [x] **Step 3: Wire module**

Add `mod app_state;` to `main.rs`.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m1): state machine for recording lifecycle"
```

---

### Task M1.8: Global hotkey wiring

**Files:**
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`
- Create: `src-tauri/src/hotkey.rs`

- [x] **Step 1: Add crate**

Append:
```toml
global-hotkey = "0.6"
```

- [x] **Step 2: Implement hotkey listener**

Create `src-tauri/src/hotkey.rs`:
```rust
use crate::error::AppError;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager};
use std::str::FromStr;
use tokio::sync::mpsc::UnboundedSender;

pub struct HotkeyService {
    manager: GlobalHotKeyManager,
    current: Option<HotKey>,
}

impl HotkeyService {
    pub fn new() -> Result<Self, AppError> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| AppError::Hotkey(format!("manager init: {e}")))?;
        Ok(Self { manager, current: None })
    }

    pub fn register(&mut self, accelerator: &str) -> Result<(), AppError> {
        let hk = HotKey::from_str(accelerator)
            .map_err(|e| AppError::Hotkey(format!("parse '{accelerator}': {e}")))?;
        if let Some(prev) = self.current.take() {
            let _ = self.manager.unregister(prev);
        }
        self.manager
            .register(hk)
            .map_err(|e| AppError::Hotkey(format!("register: {e}")))?;
        self.current = Some(hk);
        Ok(())
    }

    /// Starts a background thread that pumps GlobalHotKeyEvent::receiver()
    /// and forwards every press to `tx`.
    pub fn start_listener(tx: UnboundedSender<()>) {
        std::thread::spawn(move || {
            let rx = GlobalHotKeyEvent::receiver();
            loop {
                match rx.recv() {
                    Ok(_event) => {
                        let _ = tx.send(());
                    }
                    Err(e) => {
                        tracing::error!("hotkey rx closed: {e}");
                        break;
                    }
                }
            }
        });
    }
}
```

- [x] **Step 3: Add `mod hotkey;` to main.rs**

- [x] **Step 4: Compile check**

```bash
cd src-tauri && cargo check
```

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m1): global hotkey service with register/listen"
```

---

### Task M1.9: Model bootstrap — download whisper-tiny on first run

**Files:**
- Create: `src-tauri/src/models.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`

- [x] **Step 1: Add deps**

Append:
```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream"] }
sha2 = "0.10"
futures-util = "0.3"
dirs = "5"
```

- [x] **Step 2: Implement downloader with SHA verify**

Create `src-tauri/src/models.rs`:
```rust
use crate::error::AppError;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct ModelInfo {
    pub id: String,
    pub kind: ModelKind,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "lowercase")]
pub enum ModelKind {
    Asr,
    Llm,
}

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("whisper-hotkey"))
        .unwrap_or_else(|| PathBuf::from(".whisper-hotkey"))
}

pub fn model_path(id: &str) -> PathBuf {
    app_data_dir().join("models").join(format!("{id}.bin"))
}

pub fn builtin_catalog() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "whisper-tiny".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".into(),
            sha256: "bd577a113a864445d4c299885e0cb97d4ba92b5f".into(), // placeholder — verify on first run
            size_bytes: 75_000_000,
            display_name: "Whisper Tiny (75 MB) — fast, baseline quality".into(),
        },
        ModelInfo {
            id: "whisper-base".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".into(),
            sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf".into(),
            size_bytes: 142_000_000,
            display_name: "Whisper Base (142 MB) — recommended starter".into(),
        },
        ModelInfo {
            id: "whisper-large-v3-q5_0".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin".into(),
            sha256: "e6d2a1c6f4b8d1c2b4a8c1d2e3f4a5b6c7d8e9f0".into(),
            size_bytes: 1_080_000_000,
            display_name: "Whisper Large v3 q5_0 (1 GB) — best quality".into(),
        },
    ]
}

/// Downloads a model with progress callbacks. Verifies SHA256 on completion.
/// Resumes from partial file if present.
pub async fn download<F>(info: &ModelInfo, on_progress: F) -> Result<PathBuf, AppError>
where
    F: Fn(u64, u64) + Send,
{
    let target = model_path(&info.id);
    if target.exists() && verify_sha256(&target, &info.sha256).await? {
        return Ok(target);
    }
    tokio::fs::create_dir_all(target.parent().unwrap()).await?;
    let tmp = target.with_extension("part");
    let existing = tokio::fs::metadata(&tmp).await.ok().map(|m| m.len()).unwrap_or(0);

    let client = reqwest::Client::new();
    let mut req = client.get(&info.url);
    if existing > 0 {
        req = req.header("Range", format!("bytes={existing}-"));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Model(format!("http: {e}")))?;
    let total = resp.content_length().unwrap_or(info.size_bytes) + existing;

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(existing > 0)
        .write(true)
        .open(&tmp)
        .await?;

    let mut downloaded = existing;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Model(format!("read: {e}")))?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }
    file.flush().await?;
    drop(file);

    if !verify_sha256(&tmp, &info.sha256).await? {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(AppError::Model("sha256 mismatch".into()));
    }
    tokio::fs::rename(&tmp, &target).await?;
    Ok(target)
}

pub async fn verify_sha256(path: &Path, expected_hex: &str) -> Result<bool, AppError> {
    let bytes = tokio::fs::read(path).await?;
    let mut h = Sha256::new();
    h.update(&bytes);
    let got = h.finalize();
    Ok(format!("{:x}", got).eq_ignore_ascii_case(expected_hex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verify_sha_returns_false_for_mismatch() {
        let p = std::env::temp_dir().join("wh-test-sha.bin");
        tokio::fs::write(&p, b"hello").await.unwrap();
        assert!(!verify_sha256(&p, "00".into()).await.unwrap());
        let _ = tokio::fs::remove_file(&p).await;
    }
}
```

> NOTE for the engineer: the SHA256 values in `builtin_catalog()` are placeholders. Before merging this milestone, replace each `sha256` with the real hash by running `shasum -a 256 <downloaded model>` once. Until then, downloads will fail the verify step (which is the correct safe default).

- [x] **Step 3: Add `mod models;` to main.rs**

- [x] **Step 4: Run tests**

```bash
cd src-tauri && cargo test models::
```

Expected: 1 passed.

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m1): model downloader with sha256 verify and resume"
```

---

### Task M1.10: Wire pipeline end-to-end through `main.rs`

**Files:**
- Modify: `src-tauri/src/main.rs`

- [x] **Step 1: Replace main.rs with full wiring**

Replace `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod asr;
mod audio;
mod error;
mod hotkey;
mod logging;
mod models;
mod paste;

use app_state::{next, Intent, RecordingState};
use asr::{whisper_cpp::WhisperCpp, Transcriber};
use audio::AudioCapturer;
use error::AppError;
use models::{builtin_catalog, download, model_path};
use paste::Paster;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tokio::sync::{mpsc, Mutex};

const DEFAULT_HOTKEY: &str = "CmdOrControl+Shift+Space";
const DEFAULT_MODEL_ID: &str = "whisper-base";

struct App {
    state: Mutex<RecordingState>,
    audio: AudioCapturer,
    asr: Mutex<Option<Arc<dyn Transcriber>>>,
    paster: Box<dyn Paster>,
}

impl App {
    async fn ensure_asr_loaded(&self) -> Result<Arc<dyn Transcriber>, AppError> {
        let mut guard = self.asr.lock().await;
        if let Some(a) = guard.as_ref() {
            return Ok(a.clone());
        }
        let info = builtin_catalog()
            .into_iter()
            .find(|m| m.id == DEFAULT_MODEL_ID)
            .ok_or_else(|| AppError::Model("default model missing from catalog".into()))?;
        let path = download(&info, |d, t| {
            tracing::info!("download {d}/{t}");
        })
        .await?;
        let w = WhisperCpp::load(&model_path(&info.id).with_extension("bin").parent().unwrap().join(format!("{}.bin", info.id)))
            .or_else(|_| WhisperCpp::load(&path))?;
        let a: Arc<dyn Transcriber> = Arc::new(w);
        *guard = Some(a.clone());
        Ok(a)
    }

    async fn handle_toggle(self: Arc<Self>) {
        let mut s = self.state.lock().await;
        let new = next(*s, Intent::Toggle);
        let prev = *s;
        *s = new;
        drop(s);
        tracing::info!("state: {:?} -> {:?}", prev, new);

        match (prev, new) {
            (RecordingState::Idle, RecordingState::Recording) => {
                if let Err(e) = self.audio.start() {
                    tracing::error!("audio start failed: {e}");
                    *self.state.lock().await = RecordingState::Idle;
                }
            }
            (RecordingState::Recording, RecordingState::Transcribing) => {
                let me = self.clone();
                tokio::spawn(async move {
                    let result: Result<String, AppError> = async {
                        let samples = me.audio.stop()?;
                        if samples.is_empty() {
                            return Ok(String::new());
                        }
                        let asr = me.ensure_asr_loaded().await?;
                        let samples_clone = samples.clone();
                        let asr_clone = asr.clone();
                        let text = tokio::task::spawn_blocking(move || {
                            asr_clone.transcribe(&samples_clone, &[])
                        })
                        .await
                        .map_err(|e| AppError::Internal(format!("join: {e}")))??;
                        Ok(text)
                    }
                    .await;

                    match result {
                        Ok(text) if !text.is_empty() => {
                            if let Err(e) = me.paster.paste(&text) {
                                tracing::error!("paste failed: {e}");
                            }
                            *me.state.lock().await = next(RecordingState::Transcribing, Intent::Done);
                        }
                        Ok(_) => {
                            tracing::info!("empty transcription");
                            *me.state.lock().await = next(RecordingState::Transcribing, Intent::Done);
                        }
                        Err(e) => {
                            tracing::error!("pipeline failed: {e}");
                            *me.state.lock().await = next(RecordingState::Transcribing, Intent::Failed);
                        }
                    }
                });
            }
            _ => {}
        }
    }
}

fn main() {
    logging::init();

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();

    // Hotkey thread
    let mut hk = hotkey::HotkeyService::new().expect("hotkey service");
    hk.register(DEFAULT_HOTKEY).expect("register default hotkey");
    hotkey::HotkeyService::start_listener(tx);

    tauri::Builder::default()
        .setup(move |app| {
            let app_obj = Arc::new(App {
                state: Mutex::new(RecordingState::Idle),
                audio: AudioCapturer::new(),
                asr: Mutex::new(None),
                paster: paste::default_paster(),
            });

            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit]).build()?;
            let _tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            // Bridge hotkey events into the tokio runtime
            let app_for_loop = app_obj.clone();
            rt.spawn(async move {
                while rx.recv().await.is_some() {
                    app_for_loop.clone().handle_toggle().await;
                }
            });

            // Keep the runtime alive for the lifetime of the Tauri app
            app.manage(rt);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

> Note: the `WhisperCpp::load` retry logic above is awkward because `model_path` already returns the correct path. Simplify to one call:
> ```rust
> let w = WhisperCpp::load(&path)?;
> ```
> Use that form.

- [x] **Step 2: Compile**

```bash
cd src-tauri && cargo build
```

Fix any compile errors that surface (likely import paths or async fn signatures).

- [x] **Step 3: Smoke run — manual**

```bash
pnpm tauri dev
```

On first run on macOS, grant Accessibility permission when prompted (System Settings → Privacy & Security → Accessibility, add the dev build). Then:

1. Click into a text field of another app.
2. Press `Cmd+Shift+Space`.
3. Say "hello world".
4. Press again.
5. Watch the dev console — you should see `state: Idle -> Recording`, then `Recording -> Transcribing`, then the download progress for `whisper-base.bin` (on the very first run only), then the transcribed text getting pasted.

If the model SHA check fails (because the placeholder hashes haven't been updated), the engineer must:
1. Download the model manually: `curl -L -o /tmp/ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin`
2. Compute hash: `shasum -a 256 /tmp/ggml-base.bin`
3. Update `models.rs` `builtin_catalog()` with the real value.
4. Commit the hash update with `chore(m1): real sha256 for whisper-base`.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m1): end-to-end pipeline — hotkey, audio, asr, paste wired through state machine"
```

This is the milestone-defining commit. After this, the app actually works.

---

### Task M1.11: Smoke-test transcription with a generated audio sample

**Files:**
- Create: `src-tauri/tests/transcribe_smoke.rs`, `src-tauri/tests/fixtures/jfk.wav`

- [x] **Step 1: Add fixture**

Use the canonical 11-second JFK sample shipped with whisper.cpp:
```bash
curl -L -o src-tauri/tests/fixtures/jfk.wav \
  https://github.com/ggerganov/whisper.cpp/raw/master/samples/jfk.wav
```

- [x] **Step 2: Write the integration test**

Create `src-tauri/tests/transcribe_smoke.rs`:
```rust
//! Run with: cargo test --test transcribe_smoke -- --ignored
//! Requires the whisper-base model already downloaded in the app data dir.

use whisper_hotkey::asr::{whisper_cpp::WhisperCpp, Transcriber};
use whisper_hotkey::models::model_path;

#[test]
#[ignore]
fn transcribes_jfk_sample() {
    let path = model_path("whisper-base");
    assert!(path.exists(), "model not downloaded at {:?}", path);

    // Read jfk.wav and convert to f32 mono 16kHz
    let mut reader = hound::WavReader::open("tests/fixtures/jfk.wav").unwrap();
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    let w = WhisperCpp::load(&path).unwrap();
    let text = w.transcribe(&samples, &[]).unwrap().to_lowercase();
    assert!(
        text.contains("ask not what your country"),
        "unexpected transcription: {text}"
    );
}
```

- [x] **Step 3: Convert binary crate to lib+bin**

Test code needs to import modules. Edit `src-tauri/Cargo.toml`:
```toml
[lib]
name = "whisper_hotkey"
path = "src/lib.rs"

[[bin]]
name = "whisper-hotkey"
path = "src/main.rs"

[dev-dependencies]
hound = "3.5"
```

Create `src-tauri/src/lib.rs`:
```rust
pub mod app_state;
pub mod asr;
pub mod audio;
pub mod error;
pub mod hotkey;
pub mod logging;
pub mod models;
pub mod paste;
```

In `main.rs` remove the `mod X;` declarations and replace with `use whisper_hotkey::{...};` for the modules it uses. Compile to fix references.

- [x] **Step 4: Run the smoke test (manually, ignored by default)**

```bash
cd src-tauri && cargo test --test transcribe_smoke -- --ignored
```

Expected: passes if model is downloaded. CI does not run ignored tests, so this is a manual quality gate.

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "test(m1): jfk smoke test for whisper.cpp transcription (ignored by default)"
```

---

### Task M1.12: Documentation — `README.dev.md`

**Files:**
- Create: `README.dev.md`

- [x] **Step 1: Write developer onboarding**

Create `README.dev.md`:
```markdown
# Whisper Hotkey — dev guide

## Prerequisites

- macOS 12+ (Apple Silicon) or Windows 10+ (x64)
- Rust 1.78+ (`rustup`)
- Node 20+ and pnpm 9+ (`npm i -g pnpm`)
- macOS: Xcode CLI tools (`xcode-select --install`)
- Windows: Visual Studio 2022 Build Tools (C++ workload)

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
- **whisper.cpp build fails on Windows** — make sure VS Build Tools include "Desktop development with C++".
```

- [x] **Step 2: Commit**

```bash
git add README.dev.md
git commit -m "docs(m1): developer onboarding guide"
```

---

# Milestone M2 — Overlay window

**Goal:** A small floating window above all apps shows the current recording state with smooth animations. State events drive it from Rust.

**Done when:** When you press the hotkey, a 120×40px borderless red-pulsing pill appears at the configured screen position. When recording stops and transcription starts, it morphs to a blue spinner. When idle, it hides. No mouse capture (clicks pass through to apps below).

---

### Task M2.1: Multi-window setup in `tauri.conf.json`

**Files:**
- Modify: `src-tauri/tauri.conf.json`, `vite.config.ts`, `package.json`
- Create: `overlay.html`, `src/overlay-main.tsx`

- [x] **Step 1: Add overlay window config**

Edit `src-tauri/tauri.conf.json` `app.windows`:
```json
"windows": [
  {
    "label": "main",
    "title": "Whisper Hotkey",
    "width": 800, "height": 600, "visible": false,
    "decorations": true, "resizable": true
  },
  {
    "label": "overlay",
    "url": "overlay.html",
    "width": 140, "height": 48,
    "visible": false,
    "decorations": false,
    "resizable": false,
    "transparent": true,
    "alwaysOnTop": true,
    "skipTaskbar": true,
    "focus": false,
    "shadow": false,
    "acceptFirstMouse": false
  }
]
```

- [x] **Step 2: Create overlay entry**

Create `overlay.html`:
```html
<!doctype html>
<html><head><meta charset="utf-8"><title>Overlay</title></head>
<body><div id="overlay-root"></div><script type="module" src="/src/overlay-main.tsx"></script></body></html>
```

Create `src/overlay-main.tsx`:
```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import "./styles/globals.css";
import { OverlayWindow } from "./windows/overlay/OverlayWindow";

ReactDOM.createRoot(document.getElementById("overlay-root")!).render(<OverlayWindow />);
```

- [x] **Step 3: Update Vite config for multi-entry**

Edit `vite.config.ts`:
```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react()],
  resolve: { alias: { "@": path.resolve(__dirname, "src") } },
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
        overlay: path.resolve(__dirname, "overlay.html"),
      },
    },
  },
  server: { port: 1420, strictPort: true },
});
```

- [x] **Step 4: Smoke check**

```bash
pnpm tauri dev
```

Expected: tray appears; no visible overlay yet (visible=false). No errors.

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m2): overlay window scaffolding (transparent, always-on-top, hidden)"
```

---

### Task M2.2: OverlayWindow React component with three states

**Files:**
- Create: `src/windows/overlay/OverlayWindow.tsx`, `src/store/recordingStore.ts`, `src/ipc/events.ts`

- [x] **Step 1: Write store**

Create `src/store/recordingStore.ts`:
```ts
import { create } from "zustand";

export type RecordingState = "idle" | "recording" | "transcribing";

interface State {
  state: RecordingState;
  setState: (s: RecordingState) => void;
}

export const useRecordingStore = create<State>((set) => ({
  state: "idle",
  setState: (state) => set({ state }),
}));
```

```bash
pnpm add zustand
```

- [x] **Step 2: Write event wrapper**

Create `src/ipc/events.ts`:
```ts
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useRecordingStore, RecordingState } from "@/store/recordingStore";

export async function bindRecordingEvents(): Promise<UnlistenFn> {
  return listen<RecordingState>("state-changed", (e) => {
    useRecordingStore.getState().setState(e.payload);
  });
}
```

- [x] **Step 3: Write component**

Create `src/windows/overlay/OverlayWindow.tsx`:
```tsx
import { useEffect } from "react";
import { useRecordingStore } from "@/store/recordingStore";
import { bindRecordingEvents } from "@/ipc/events";
import { Window } from "@tauri-apps/api/window";

export function OverlayWindow() {
  const state = useRecordingStore((s) => s.state);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bindRecordingEvents().then((u) => (unlisten = u));
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    const w = Window.getCurrent();
    if (state === "idle") w.hide();
    else w.show();
  }, [state]);

  const label =
    state === "recording" ? "Recording" : state === "transcribing" ? "Transcribing…" : "";
  const color =
    state === "recording" ? "bg-red-500" : state === "transcribing" ? "bg-blue-500" : "bg-transparent";

  return (
    <div className="flex h-full w-full items-center justify-center">
      <div className={`flex items-center gap-2 rounded-full px-3 py-1 text-sm text-white ${color} ${state === "recording" ? "animate-pulse" : ""}`}>
        {state === "transcribing" && (
          <span className="inline-block h-3 w-3 animate-spin rounded-full border-2 border-white border-t-transparent" />
        )}
        {state !== "recording" || <span className="inline-block h-2 w-2 rounded-full bg-white" />}
        <span>{label}</span>
      </div>
    </div>
  );
}
```

- [x] **Step 4: Compile + visual check**

```bash
pnpm tauri dev
```

You won't see the overlay yet (no events fired). That's expected.

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m2): overlay react component reactive to recordingStore"
```

---

### Task M2.3: Emit state events from Rust

**Files:**
- Create: `src-tauri/src/events.rs`
- Modify: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [x] **Step 1: Implement emitter**

Create `src-tauri/src/events.rs`:
```rust
use crate::app_state::RecordingState;
use tauri::{AppHandle, Emitter};

pub fn emit_state(app: &AppHandle, state: RecordingState) {
    let payload = match state {
        RecordingState::Idle => "idle",
        RecordingState::Recording => "recording",
        RecordingState::Transcribing => "transcribing",
    };
    if let Err(e) = app.emit("state-changed", payload) {
        tracing::error!("emit state-changed: {e}");
    }
}
```

Add `pub mod events;` to `lib.rs`.

- [x] **Step 2: Wire emitter into the state machine**

In `main.rs`, find every place that mutates `self.state` and immediately call `events::emit_state(&app_handle, new_state)`. Easiest: store an `AppHandle` in `App`:
```rust
struct App {
    state: Mutex<RecordingState>,
    audio: AudioCapturer,
    asr: Mutex<Option<Arc<dyn Transcriber>>>,
    paster: Box<dyn Paster>,
    handle: AppHandle,
}
```

After every `*s = new;` or `*self.state.lock().await = ...`, call `events::emit_state(&self.handle, new);`.

- [x] **Step 3: Smoke run**

```bash
pnpm tauri dev
```

Press hotkey: overlay appears red. Press again: overlay turns blue (spinner). After transcription pastes, overlay hides.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m2): rust emits state-changed events; overlay reflects in realtime"
```

---

### Task M2.4: Overlay position config (read-only for now)

**Files:**
- Modify: `src/windows/overlay/OverlayWindow.tsx`, `src-tauri/src/main.rs`

- [x] **Step 1: Position overlay at top-center on show**

In `main.rs`, after `events::emit_state(handle, RecordingState::Recording)`, also position the overlay window:
```rust
if let Some(overlay) = handle.get_webview_window("overlay") {
    if let Ok(monitor) = overlay.primary_monitor() {
        if let Some(m) = monitor {
            let size = m.size();
            let scale = m.scale_factor();
            let x = (size.width as f64 / scale - 140.0) / 2.0;
            let y = 20.0;
            let _ = overlay.set_position(tauri::PhysicalPosition::new(
                (x * scale) as i32,
                (y * scale) as i32,
            ));
        }
    }
}
```

(Position is currently hardcoded top-center; it becomes configurable in M3.)

- [x] **Step 2: Smoke**

```bash
pnpm tauri dev
```

Overlay appears at the top of the primary monitor, centered horizontally.

- [x] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m2): overlay positioned top-center on primary monitor"
```

---

# Milestone M3 — Settings: General + Model tabs

**Goal:** Settings window with two tabs. User can change hotkey, auto-paste, theme, overlay position, ASR model. Config persists across restarts. Model downloader has progress UI.

**Done when:** Open Settings from tray, see two tabs, edit each option, close + reopen the app, options persist. Switch ASR model — old model unloads, new one downloads with a progress bar, then is used for the next transcription.

---

### Task M3.1: Config schema + storage

**Files:**
- Create: `src-tauri/src/storage/mod.rs`, `src-tauri/src/storage/config.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`

- [x] **Step 1: Write tests**

Create `src-tauri/src/storage/config.rs`:
```rust
use crate::error::AppError;
use crate::models::app_data_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "snake_case")]
pub enum OverlayPosition {
    TopCenter,
    TopLeft,
    TopRight,
    BottomCenter,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Theme { System, Light, Dark }

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct ReplacementRule {
    pub from: String,
    pub to: String,
    pub regex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(default)]
pub struct Config {
    pub hotkey: String,
    pub auto_paste: bool,
    pub overlay_position: OverlayPosition,
    pub theme: Theme,
    pub asr_model: String,
    pub post_processing_enabled: bool,
    pub llm_model: String,
    pub llm_timeout_ms: u64,
    pub vocabulary: Vec<String>,
    pub replacements: Vec<ReplacementRule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "CmdOrControl+Shift+Space".into(),
            auto_paste: true,
            overlay_position: OverlayPosition::TopCenter,
            theme: Theme::System,
            asr_model: "whisper-base".into(),
            post_processing_enabled: false,
            llm_model: "gemma-2-2b-it-q4_k_m".into(),
            llm_timeout_ms: 8000,
            vocabulary: vec![],
            replacements: vec![],
        }
    }
}

fn config_path() -> PathBuf { app_data_dir().join("config.json") }

pub fn load() -> Config {
    let p = config_path();
    match std::fs::read_to_string(&p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save(cfg: &Config) -> Result<(), AppError> {
    let p = config_path();
    if let Some(parent) = p.parent() { std::fs::create_dir_all(parent)?; }
    let s = serde_json::to_string_pretty(cfg)
        .map_err(|e| AppError::Internal(format!("serialize config: {e}")))?;
    std::fs::write(p, s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stable() {
        let c = Config::default();
        assert_eq!(c.hotkey, "CmdOrControl+Shift+Space");
        assert!(c.auto_paste);
        assert_eq!(c.llm_timeout_ms, 8000);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let partial = r#"{"hotkey":"F12"}"#;
        let c: Config = serde_json::from_str(partial).unwrap();
        assert_eq!(c.hotkey, "F12");
        assert!(c.auto_paste); // default
    }
}
```

Create `src-tauri/src/storage/mod.rs`:
```rust
pub mod config;
```

Add `pub mod storage;` to `lib.rs`.

- [x] **Step 2: Run tests**

```bash
cd src-tauri && cargo test storage::
```

Expected: 2 passed.

- [x] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m3): config schema + JSON persistence with serde defaults"
```

---

### Task M3.2: Tauri commands for config get/update

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [x] **Step 1: Write commands**

Create `src-tauri/src/commands.rs`:
```rust
use crate::error::{AppError, AppErrorDto};
use crate::storage::config::{self, Config};
use parking_lot::Mutex as PMutex;
use std::sync::Arc;
use tauri::State;

pub struct ConfigState(pub Arc<PMutex<Config>>);

impl From<AppError> for AppErrorDto {
    fn from(e: AppError) -> Self { e.to_dto() }
}

#[tauri::command]
pub fn get_config(state: State<'_, ConfigState>) -> Config {
    state.0.lock().clone()
}

#[tauri::command]
pub fn update_config(
    patch: serde_json::Value,
    state: State<'_, ConfigState>,
) -> Result<Config, AppErrorDto> {
    let mut cfg = state.0.lock();
    let mut v = serde_json::to_value(&*cfg).map_err(|e| AppError::Internal(e.to_string()))?;
    if let (Some(obj), Some(p)) = (v.as_object_mut(), patch.as_object()) {
        for (k, val) in p { obj.insert(k.clone(), val.clone()); }
    }
    let new: Config = serde_json::from_value(v).map_err(|e| AppError::Internal(e.to_string()))?;
    config::save(&new).map_err(AppErrorDto::from)?;
    *cfg = new.clone();
    Ok(new)
}
```

Add `pub mod commands;` to `lib.rs`. In `main.rs`, register commands and state:
```rust
.manage(commands::ConfigState(Arc::new(PMutex::new(config::load()))))
.invoke_handler(tauri::generate_handler![
    commands::get_config,
    commands::update_config,
])
```

- [x] **Step 2: Compile**

```bash
cd src-tauri && cargo build
```

- [x] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m3): tauri commands get_config + update_config with partial patch"
```

---

### Task M3.3: Settings window — shell

**Files:**
- Modify: `src-tauri/tauri.conf.json`, `src/App.tsx`
- Create: `src/windows/settings/SettingsWindow.tsx`

- [x] **Step 1: Re-purpose the "main" window**

The existing `main` window will host Settings (and later Onboarding). Edit `tauri.conf.json` to set `visible: false` initially and `width: 900 / height: 640`.

Modify `src/App.tsx`:
```tsx
import { SettingsWindow } from "@/windows/settings/SettingsWindow";

export default function App() {
  return <SettingsWindow />;
}
```

- [x] **Step 2: Install shadcn components used in Settings**

```bash
pnpm dlx shadcn@latest add tabs button input switch select label
```

- [x] **Step 3: Settings shell with tabs**

Create `src/windows/settings/SettingsWindow.tsx`:
```tsx
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { GeneralTab } from "./GeneralTab";
import { ModelTab } from "./ModelTab";

export function SettingsWindow() {
  return (
    <div className="h-screen w-screen p-6">
      <h1 className="mb-4 text-lg font-semibold">Settings</h1>
      <Tabs defaultValue="general" className="h-full">
        <TabsList>
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="model">Model</TabsTrigger>
          <TabsTrigger value="vocabulary" disabled>Vocabulary</TabsTrigger>
        </TabsList>
        <TabsContent value="general"><GeneralTab /></TabsContent>
        <TabsContent value="model"><ModelTab /></TabsContent>
      </Tabs>
    </div>
  );
}
```

- [x] **Step 4: Tray menu — open Settings**

In `main.rs`, add a "Settings" item before "Quit":
```rust
let settings = MenuItemBuilder::with_id("settings", "Settings…").build(app)?;
let menu = MenuBuilder::new(app).items(&[&settings, &quit]).build()?;
```
And in `on_menu_event`:
```rust
"settings" => {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}
```

- [x] **Step 5: Commit (placeholder tabs)**

```bash
git add -A
git commit -m "feat(m3): settings window shell with tabs, opened from tray menu"
```

---

### Task M3.4: General tab

**Files:**
- Create: `src/windows/settings/GeneralTab.tsx`, `src/store/configStore.ts`, `src/ipc/commands.ts`, `src/components/HotkeyCapture.tsx`

- [x] **Step 1: IPC wrapper**

Create `src/ipc/commands.ts`:
```ts
import { invoke } from "@tauri-apps/api/core";
import type { Config } from "./generated/Config";

export const cmd = {
  getConfig: () => invoke<Config>("get_config"),
  updateConfig: (patch: Partial<Config>) => invoke<Config>("update_config", { patch }),
};
```

(`Config.ts` will be generated by `ts-rs` — see Task M3.7.)

- [x] **Step 2: Config store**

Create `src/store/configStore.ts`:
```ts
import { create } from "zustand";
import { cmd } from "@/ipc/commands";
import type { Config } from "@/ipc/generated/Config";

interface State {
  config: Config | null;
  load: () => Promise<void>;
  update: (patch: Partial<Config>) => Promise<void>;
}

export const useConfigStore = create<State>((set) => ({
  config: null,
  load: async () => set({ config: await cmd.getConfig() }),
  update: async (patch) => set({ config: await cmd.updateConfig(patch) }),
}));
```

- [x] **Step 3: HotkeyCapture component**

Create `src/components/HotkeyCapture.tsx`:
```tsx
import { useState } from "react";
import { Button } from "@/components/ui/button";

const MODIFIER_ORDER = ["CmdOrControl", "Alt", "Shift"] as const;

function eventToAccelerator(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.metaKey || e.ctrlKey) parts.push("CmdOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  if (["Control", "Meta", "Alt", "Shift"].includes(key)) return null;
  parts.push(key === " " ? "Space" : key);
  return parts.join("+");
}

export function HotkeyCapture({
  value, onChange,
}: { value: string; onChange: (v: string) => void }) {
  const [recording, setRecording] = useState(false);
  return (
    <Button
      type="button"
      variant="outline"
      onClick={() => setRecording(true)}
      onKeyDown={(e) => {
        if (!recording) return;
        e.preventDefault();
        const acc = eventToAccelerator(e.nativeEvent);
        if (acc) { onChange(acc); setRecording(false); }
      }}
      autoFocus={recording}
    >
      {recording ? "Press hotkey…" : value}
    </Button>
  );
}
```

- [x] **Step 4: General tab**

Create `src/windows/settings/GeneralTab.tsx`:
```tsx
import { useEffect } from "react";
import { useConfigStore } from "@/store/configStore";
import { HotkeyCapture } from "@/components/HotkeyCapture";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

export function GeneralTab() {
  const { config, load, update } = useConfigStore();
  useEffect(() => { void load(); }, [load]);
  if (!config) return null;

  return (
    <div className="mt-4 grid max-w-md gap-6">
      <div className="grid gap-2">
        <Label>Hotkey</Label>
        <HotkeyCapture value={config.hotkey} onChange={(v) => update({ hotkey: v })} />
      </div>
      <div className="flex items-center justify-between">
        <Label>Auto-paste</Label>
        <Switch checked={config.auto_paste} onCheckedChange={(v) => update({ auto_paste: v })} />
      </div>
      <div className="grid gap-2">
        <Label>Overlay position</Label>
        <Select value={config.overlay_position} onValueChange={(v) => update({ overlay_position: v as any })}>
          <SelectTrigger><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="top_center">Top center</SelectItem>
            <SelectItem value="top_left">Top left</SelectItem>
            <SelectItem value="top_right">Top right</SelectItem>
            <SelectItem value="bottom_center">Bottom center</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-2">
        <Label>Theme</Label>
        <Select value={config.theme} onValueChange={(v) => update({ theme: v as any })}>
          <SelectTrigger><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="system">System</SelectItem>
            <SelectItem value="light">Light</SelectItem>
            <SelectItem value="dark">Dark</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
```

- [x] **Step 5: Smoke**

```bash
pnpm tauri dev
```

Open Settings via tray, edit fields, restart app — values persist.

- [x] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(m3): general settings tab — hotkey, autopaste, overlay position, theme"
```

---

### Task M3.5: React to hotkey config changes at runtime

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/main.rs`

- [x] **Step 1: Hold the HotkeyService in state**

```rust
pub struct HotkeyState(pub Arc<PMutex<crate::hotkey::HotkeyService>>);
```

In `main.rs`, after creating `HotkeyService`, `manage(HotkeyState(Arc::new(PMutex::new(hk))))`.

- [x] **Step 2: Re-register hotkey on update_config**

In `commands::update_config`, after the config is saved, compare old vs new and call `hotkey_service.register(&new.hotkey)?` if changed. Pass `HotkeyState` as another `State<'_, HotkeyState>` argument.

```rust
#[tauri::command]
pub fn update_config(
    patch: serde_json::Value,
    cfg_state: State<'_, ConfigState>,
    hk_state: State<'_, HotkeyState>,
) -> Result<Config, AppErrorDto> {
    let mut cfg = cfg_state.0.lock();
    let prev_hotkey = cfg.hotkey.clone();
    // ... (existing patch logic) ...
    if new.hotkey != prev_hotkey {
        hk_state.0.lock().register(&new.hotkey).map_err(AppErrorDto::from)?;
    }
    Ok(new)
}
```

- [x] **Step 3: Smoke**

Set hotkey to `F12` in Settings, press F12, recording should start.

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m3): hotkey change applies live without restart"
```

---

### Task M3.6: Model tab with download progress

**Files:**
- Create: `src/windows/settings/ModelTab.tsx`, `src/components/ModelCard.tsx`
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/main.rs`

- [x] **Step 1: Backend commands**

Add to `src-tauri/src/commands.rs`:
```rust
use crate::models::{builtin_catalog, download, model_path, ModelInfo};

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> { builtin_catalog() }

#[tauri::command]
pub async fn download_model(
    id: String,
    app: tauri::AppHandle,
) -> Result<(), AppErrorDto> {
    let info = builtin_catalog()
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| AppError::Model(format!("unknown model: {id}")))?;
    let app_emit = app.clone();
    let id2 = id.clone();
    download(&info, move |d, t| {
        let _ = app_emit.emit("model-download-progress", serde_json::json!({
            "id": id2, "bytes": d, "total": t,
        }));
    }).await.map_err(AppErrorDto::from)?;
    Ok(())
}

#[tauri::command]
pub fn delete_model(id: String) -> Result<(), AppErrorDto> {
    let p = model_path(&id);
    if p.exists() { std::fs::remove_file(p).map_err(|e| AppError::Storage(e))?; }
    Ok(())
}

#[tauri::command]
pub fn is_model_present(id: String) -> bool {
    model_path(&id).exists()
}
```

Register all in the `invoke_handler!` macro in `main.rs`.

- [x] **Step 2: Frontend ModelCard**

Create `src/components/ModelCard.tsx`:
```tsx
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

interface Props {
  id: string;
  displayName: string;
  sizeBytes: number;
  isSelected: boolean;
  onSelect: () => void;
}

export function ModelCard({ id, displayName, sizeBytes, isSelected, onSelect }: Props) {
  const [present, setPresent] = useState(false);
  const [progress, setProgress] = useState<{ bytes: number; total: number } | null>(null);

  useEffect(() => {
    invoke<boolean>("is_model_present", { id }).then(setPresent);
    const u = listen<{ id: string; bytes: number; total: number }>(
      "model-download-progress",
      (e) => { if (e.payload.id === id) setProgress({ bytes: e.payload.bytes, total: e.payload.total }); }
    );
    return () => { u.then((f) => f()); };
  }, [id]);

  async function handleDownload() {
    setProgress({ bytes: 0, total: sizeBytes });
    try { await invoke("download_model", { id }); setPresent(true); }
    finally { setProgress(null); }
  }

  async function handleDelete() {
    await invoke("delete_model", { id });
    setPresent(false);
  }

  const pct = progress ? Math.floor((progress.bytes / progress.total) * 100) : 0;

  return (
    <div className={`rounded-lg border p-4 ${isSelected ? "border-accent" : "border-border"}`}>
      <div className="flex items-center justify-between">
        <div>
          <div className="font-medium">{displayName}</div>
          <div className="text-xs text-muted-foreground">{(sizeBytes / 1_000_000).toFixed(0)} MB</div>
        </div>
        <div className="flex gap-2">
          {present ? (
            <>
              <Button size="sm" variant={isSelected ? "default" : "outline"} onClick={onSelect}>
                {isSelected ? "Selected" : "Use"}
              </Button>
              <Button size="sm" variant="ghost" onClick={handleDelete}>Delete</Button>
            </>
          ) : progress ? (
            <div className="w-32"><div className="h-1 bg-muted rounded"><div className="h-full bg-accent rounded" style={{ width: `${pct}%` }} /></div><div className="mt-1 text-xs">{pct}%</div></div>
          ) : (
            <Button size="sm" onClick={handleDownload}>Download</Button>
          )}
        </div>
      </div>
    </div>
  );
}
```

- [x] **Step 3: Model tab**

Create `src/windows/settings/ModelTab.tsx`:
```tsx
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useConfigStore } from "@/store/configStore";
import { ModelCard } from "@/components/ModelCard";
import type { ModelInfo } from "@/ipc/generated/ModelInfo";

export function ModelTab() {
  const { config, update } = useConfigStore();
  const [models, setModels] = useState<ModelInfo[]>([]);

  useEffect(() => { invoke<ModelInfo[]>("list_models").then(setModels); }, []);
  if (!config) return null;

  const asr = models.filter((m) => m.kind === "asr");

  return (
    <div className="mt-4 grid gap-3 max-w-xl">
      <h3 className="font-medium">Transcription model</h3>
      {asr.map((m) => (
        <ModelCard
          key={m.id}
          id={m.id}
          displayName={m.display_name}
          sizeBytes={m.size_bytes}
          isSelected={config.asr_model === m.id}
          onSelect={() => update({ asr_model: m.id })}
        />
      ))}
    </div>
  );
}
```

- [x] **Step 4: Reload ASR on model change**

In Rust, add a watcher: when `asr_model` changes in `update_config`, set `app.asr.lock().await = None` so next transcription reloads.

This requires lifting `App` into Tauri state. Refactor (add as commented step):
```rust
.manage(app_obj.clone()) // make App accessible from commands
```

In `update_config`, accept `app: State<Arc<App>>` and call `app.reset_asr()` on change.

- [x] **Step 5: Smoke**

Open Settings → Model tab. See 3 cards. Download `whisper-base`. Switch to `whisper-tiny` after downloading it. Press hotkey — confirm tiny model is loaded (look at debug log).

- [x] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(m3): model tab — download progress, select active asr model, live reload"
```

---

### Task M3.7: Wire up `ts-rs` generation

**Files:**
- Modify: `src-tauri/Cargo.toml`, add `src-tauri/build.rs` step
- Create: `src/ipc/generated/.gitkeep`

- [x] **Step 1: Verify ts-rs exports**

Each `#[derive(ts_rs::TS)]` already specifies `#[ts(export, export_to = "../src/ipc/generated/")]`. Running `cargo test` is what triggers ts-rs to emit the files.

Add a script to `package.json`:
```json
"gen-types": "cd src-tauri && cargo test --quiet ts_export_ -- --nocapture || true"
```

Add a single test in each module that uses `ts-rs` to force exports. Example in `error.rs`:
```rust
#[test]
fn ts_export_AppErrorDto() { AppErrorDto::export().unwrap(); }
```

Add similar `ts_export_*` test in `app_state.rs`, `models.rs`, `storage/config.rs`.

```bash
pnpm gen-types
ls src/ipc/generated/
```

Expected: `Config.ts`, `ModelInfo.ts`, `RecordingState.ts`, `ErrorKind.ts`, `AppErrorDto.ts`, etc.

- [x] **Step 2: Add `.gitkeep` placeholder and ignore generated TS in lint**

Create `src/ipc/generated/.gitkeep`. In `eslintrc`/Prettier config, ignore `src/ipc/generated/**`.

- [x] **Step 3: Commit generated types**

```bash
git add -A
git commit -m "chore(m3): generate ts types from rust via ts-rs"
```

---

### Task M3.8: Update README.dev.md and lock M3

- [x] **Step 1: Document the type generation step**

Append to `README.dev.md`:
```markdown
## Regenerating IPC types

After changing any Rust type that derives `ts_rs::TS`:
```bash
pnpm gen-types
```
Commit the resulting files in `src/ipc/generated/`.
```

- [x] **Step 2: Commit**

```bash
git add README.dev.md
git commit -m "docs(m3): note type regeneration workflow"
```

---

# Milestone M4 — History

**Goal:** Every transcription is appended to `history.jsonl`. A History window (opened from tray) lists them with search, copy, delete, and export.

**Done when:** Transcribe 3 things. Open tray → History → see all 3 with timestamps. Search filters. Copy puts text on clipboard. Delete removes (and is persisted). Export saves a `.md` file.

---

### Task M4.1: History storage

**Files:**
- Create: `src-tauri/src/storage/history.rs`
- Modify: `src-tauri/src/storage/mod.rs`, `src-tauri/src/lib.rs`

- [x] **Step 1: Test + impl**

Create `src-tauri/src/storage/history.rs`:
```rust
use crate::error::AppError;
use crate::models::app_data_dir;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct HistoryEntry {
    pub ts: String,        // ISO 8601
    pub text: String,
    pub model: String,
    pub post_processed: bool,
}

fn history_path() -> PathBuf { app_data_dir().join("history.jsonl") }

pub fn append(entry: &HistoryEntry) -> Result<(), AppError> {
    let p = history_path();
    if let Some(parent) = p.parent() { std::fs::create_dir_all(parent)?; }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&p)?;
    writeln!(f, "{}", serde_json::to_string(entry).map_err(|e| AppError::Internal(e.to_string()))?)?;
    Ok(())
}

pub fn read_all() -> Result<Vec<HistoryEntry>, AppError> {
    let p = history_path();
    if !p.exists() { return Ok(vec![]); }
    let f = std::fs::File::open(&p)?;
    let mut out = Vec::new();
    for line in BufReader::new(f).lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        if let Ok(e) = serde_json::from_str(&line) { out.push(e); }
    }
    Ok(out)
}

pub fn delete_by_ts(ts: &str) -> Result<(), AppError> {
    let all: Vec<_> = read_all()?.into_iter().filter(|e| e.ts != ts).collect();
    let p = history_path();
    let mut f = std::fs::File::create(&p)?;
    for e in all {
        writeln!(f, "{}", serde_json::to_string(&e).map_err(|e| AppError::Internal(e.to_string()))?)?;
    }
    Ok(())
}

pub fn clear() -> Result<(), AppError> {
    let p = history_path();
    if p.exists() { std::fs::remove_file(p)?; }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        // Use a temp env so app_data_dir is overridable — for now, accept side effects in tmp
        let e = HistoryEntry { ts: "2026-01-01T00:00:00Z".into(), text: "hi".into(), model: "x".into(), post_processed: false };
        let _ = clear();
        append(&e).unwrap();
        let all = read_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].text, "hi");
        delete_by_ts("2026-01-01T00:00:00Z").unwrap();
        assert_eq!(read_all().unwrap().len(), 0);
    }
}
```

Add `pub mod history;` to `storage/mod.rs`.

- [x] **Step 2: Test**

```bash
cd src-tauri && cargo test history::
```

- [x] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m4): history storage in jsonl with append/read/delete"
```

---

### Task M4.2: Append history during pipeline

**Files:**
- Modify: `src-tauri/src/main.rs`

- [x] **Step 1: Hook into pipeline**

In the pipeline success branch (after `text` is computed, before paste):
```rust
let entry = whisper_hotkey::storage::history::HistoryEntry {
    ts: chrono::Utc::now().to_rfc3339(),
    text: text.clone(),
    model: cfg.asr_model.clone(),
    post_processed: false,
};
let _ = whisper_hotkey::storage::history::append(&entry);
```

Add `chrono = "0.4"` to `Cargo.toml`.

- [x] **Step 2: Commit**

```bash
git add -A
git commit -m "feat(m4): pipeline appends transcription to history"
```

---

### Task M4.3: History commands + window

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`
- Create: `history.html`, `src/history-main.tsx`, `src/windows/history/HistoryWindow.tsx`, `src/store/historyStore.ts`

- [x] **Step 1: Add commands**

```rust
#[tauri::command]
pub fn get_history() -> Result<Vec<HistoryEntry>, AppErrorDto> {
    history::read_all().map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_history_entry(ts: String) -> Result<(), AppErrorDto> {
    history::delete_by_ts(&ts).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn clear_history() -> Result<(), AppErrorDto> {
    history::clear().map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn export_history(path: String) -> Result<(), AppErrorDto> {
    let all = history::read_all().map_err(AppErrorDto::from)?;
    let mut md = String::from("# Transcription history\n\n");
    for e in all {
        md.push_str(&format!("## {}\n\n{}\n\n", e.ts, e.text));
    }
    std::fs::write(path, md).map_err(|e| AppError::Storage(e).into())
}
```

Register in `invoke_handler!`.

- [x] **Step 2: Window config**

Add to `tauri.conf.json` `windows`:
```json
{ "label": "history", "url": "history.html", "title": "History", "width": 500, "height": 700, "visible": false }
```

Create `history.html` mirroring `overlay.html` with `id="history-root"` and `/src/history-main.tsx`.

Create `src/history-main.tsx`:
```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import "./styles/globals.css";
import { HistoryWindow } from "./windows/history/HistoryWindow";

ReactDOM.createRoot(document.getElementById("history-root")!).render(<HistoryWindow />);
```

Edit `vite.config.ts` `build.rollupOptions.input` to add `history: path.resolve(__dirname, "history.html")`.

- [x] **Step 3: Store**

Create `src/store/historyStore.ts`:
```ts
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { HistoryEntry } from "@/ipc/generated/HistoryEntry";

interface State {
  entries: HistoryEntry[];
  load: () => Promise<void>;
  remove: (ts: string) => Promise<void>;
  clearAll: () => Promise<void>;
}

export const useHistoryStore = create<State>((set, get) => ({
  entries: [],
  load: async () => set({ entries: await invoke("get_history") }),
  remove: async (ts) => { await invoke("delete_history_entry", { ts }); await get().load(); },
  clearAll: async () => { await invoke("clear_history"); set({ entries: [] }); },
}));
```

- [x] **Step 4: Window component**

Create `src/windows/history/HistoryWindow.tsx`:
```tsx
import { useEffect, useState } from "react";
import { useHistoryStore } from "@/store/historyStore";
import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export function HistoryWindow() {
  const { entries, load, remove, clearAll } = useHistoryStore();
  const [q, setQ] = useState("");
  useEffect(() => { void load(); }, [load]);

  const filtered = entries
    .filter((e) => e.text.toLowerCase().includes(q.toLowerCase()))
    .reverse();

  async function copy(text: string) { await navigator.clipboard.writeText(text); }

  async function exportAll() {
    const path = await saveDialog({ defaultPath: "history.md", filters: [{ name: "Markdown", extensions: ["md"] }] });
    if (path) await invoke("export_history", { path });
  }

  return (
    <div className="flex h-screen w-screen flex-col p-4 gap-3">
      <div className="flex gap-2">
        <Input placeholder="Search…" value={q} onChange={(e) => setQ(e.target.value)} />
        <Button variant="outline" onClick={exportAll}>Export</Button>
        <Button variant="ghost" onClick={clearAll}>Clear all</Button>
      </div>
      <div className="flex-1 overflow-y-auto space-y-2">
        {filtered.map((e) => (
          <div key={e.ts} className="rounded border p-3 text-sm">
            <div className="mb-1 flex items-center justify-between">
              <span className="text-xs text-muted-foreground">{new Date(e.ts).toLocaleString()}</span>
              <div className="flex gap-1">
                <Button size="sm" variant="ghost" onClick={() => copy(e.text)}>Copy</Button>
                <Button size="sm" variant="ghost" onClick={() => remove(e.ts)}>Delete</Button>
              </div>
            </div>
            <div className="whitespace-pre-wrap">{e.text}</div>
          </div>
        ))}
        {filtered.length === 0 && <div className="text-center text-sm text-muted-foreground py-12">No transcriptions yet.</div>}
      </div>
    </div>
  );
}
```

```bash
pnpm tauri add dialog
```

- [x] **Step 5: Tray entry**

In `main.rs` tray menu, add `History…` item before Quit; on click, show the `history` window.

- [x] **Step 6: Smoke**

Transcribe 2-3 things. Open History from tray. Verify list, search, copy, delete, export.

- [x] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(m4): history window with search, copy, delete, export to markdown"
```

---

# Milestone M5 — Vocabulary + Replacements

**Goal:** User can define custom vocabulary words (passed to whisper as `initial_prompt`) and substitution rules (literal or regex) applied to the transcription before paste.

**Done when:** Add "Ploomes" to vocab → next transcription of "ploo-mes" outputs "Ploomes". Add rule `\bmail\b → email` → "mail" in transcription becomes "email" pasted.

---

### Task M5.1: Replacements engine

**Files:**
- Create: `src-tauri/src/replacements.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`

- [x] **Step 1: Add regex crate**

```toml
regex = "1"
```

- [x] **Step 2: Tests + impl**

Create `src-tauri/src/replacements.rs`:
```rust
use crate::storage::config::ReplacementRule;
use regex::Regex;

pub fn apply(input: &str, rules: &[ReplacementRule]) -> String {
    let mut s = input.to_string();
    for r in rules {
        if r.regex {
            if let Ok(re) = Regex::new(&r.from) {
                s = re.replace_all(&s, r.to.as_str()).to_string();
            }
        } else {
            s = s.replace(&r.from, &r.to);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::config::ReplacementRule;

    fn lit(from: &str, to: &str) -> ReplacementRule {
        ReplacementRule { from: from.into(), to: to.into(), regex: false }
    }
    fn rx(from: &str, to: &str) -> ReplacementRule {
        ReplacementRule { from: from.into(), to: to.into(), regex: true }
    }

    #[test]
    fn literal_replace() {
        assert_eq!(apply("hello world", &[lit("hello", "hi")]), "hi world");
    }

    #[test]
    fn regex_word_boundary() {
        assert_eq!(apply("send mail to mailbox", &[rx(r"\bmail\b", "email")]), "send email to mailbox");
    }

    #[test]
    fn invalid_regex_is_skipped() {
        assert_eq!(apply("ok", &[rx("[", "x")]), "ok");
    }

    #[test]
    fn rules_applied_in_order() {
        let rules = vec![lit("a", "b"), lit("b", "c")];
        assert_eq!(apply("a", &rules), "c");
    }
}
```

Add `pub mod replacements;` to `lib.rs`.

- [x] **Step 3: Test**

```bash
cd src-tauri && cargo test replacements::
```

- [x] **Step 4: Wire into pipeline**

In `main.rs`, after transcription text is obtained:
```rust
let cfg = config_state.0.lock().clone();
let text = whisper_hotkey::replacements::apply(&text, &cfg.replacements);
```

Also pass `&cfg.vocabulary` to `WhisperCpp::transcribe`.

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m5): replacements engine — literal + regex, applied in pipeline"
```

---

### Task M5.2: Vocabulary tab UI

**Files:**
- Create: `src/windows/settings/VocabularyTab.tsx`, `src/components/ReplacementEditor.tsx`

- [x] **Step 1: Tab**

Create `src/windows/settings/VocabularyTab.tsx`:
```tsx
import { useConfigStore } from "@/store/configStore";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import { ReplacementEditor } from "@/components/ReplacementEditor";

export function VocabularyTab() {
  const { config, update } = useConfigStore();
  const [newWord, setNewWord] = useState("");
  if (!config) return null;

  function addWord() {
    const w = newWord.trim();
    if (!w) return;
    void update({ vocabulary: [...config!.vocabulary, w] });
    setNewWord("");
  }

  function removeWord(i: number) {
    const v = config!.vocabulary.filter((_, j) => j !== i);
    void update({ vocabulary: v });
  }

  return (
    <div className="mt-4 grid gap-8 max-w-xl">
      <section>
        <h3 className="font-medium mb-2">Custom words</h3>
        <div className="flex gap-2 mb-2">
          <Input value={newWord} onChange={(e) => setNewWord(e.target.value)} placeholder="e.g. Ploomes" onKeyDown={(e) => e.key === "Enter" && addWord()} />
          <Button onClick={addWord}>Add</Button>
        </div>
        <div className="flex flex-wrap gap-1">
          {config.vocabulary.map((w, i) => (
            <span key={i} className="rounded bg-muted px-2 py-1 text-xs">
              {w} <button className="ml-1 opacity-60 hover:opacity-100" onClick={() => removeWord(i)}>×</button>
            </span>
          ))}
        </div>
      </section>

      <section>
        <h3 className="font-medium mb-2">Replacement rules</h3>
        <ReplacementEditor
          rules={config.replacements}
          onChange={(rs) => update({ replacements: rs })}
        />
      </section>
    </div>
  );
}
```

- [x] **Step 2: ReplacementEditor**

Create `src/components/ReplacementEditor.tsx`:
```tsx
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import type { ReplacementRule } from "@/ipc/generated/ReplacementRule";

interface Props {
  rules: ReplacementRule[];
  onChange: (rs: ReplacementRule[]) => void;
}

export function ReplacementEditor({ rules, onChange }: Props) {
  function update(i: number, patch: Partial<ReplacementRule>) {
    onChange(rules.map((r, j) => (j === i ? { ...r, ...patch } : r)));
  }
  function remove(i: number) { onChange(rules.filter((_, j) => j !== i)); }
  function add() { onChange([...rules, { from: "", to: "", regex: false }]); }

  return (
    <div className="grid gap-2">
      {rules.map((r, i) => (
        <div key={i} className="flex gap-2 items-center">
          <Input value={r.from} placeholder="from" onChange={(e) => update(i, { from: e.target.value })} />
          <span>→</span>
          <Input value={r.to} placeholder="to" onChange={(e) => update(i, { to: e.target.value })} />
          <label className="text-xs flex items-center gap-1">
            <Switch checked={r.regex} onCheckedChange={(v) => update(i, { regex: v })} /> regex
          </label>
          <Button size="sm" variant="ghost" onClick={() => remove(i)}>×</Button>
        </div>
      ))}
      <Button size="sm" variant="outline" onClick={add}>+ Add rule</Button>
    </div>
  );
}
```

- [x] **Step 3: Enable the tab**

In `SettingsWindow.tsx` remove `disabled` from `<TabsTrigger value="vocabulary">` and add `<TabsContent value="vocabulary"><VocabularyTab /></TabsContent>`.

- [x] **Step 4: Smoke**

Add `Ploomes` to vocab and a `mail → email` rule. Transcribe a sentence with both. Verify substitutions and that whisper gets the prompt (check debug logs).

- [x] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m5): vocabulary tab — words editor + replacement rules"
```

---

# Milestone M6 — LLM post-processing

**Goal:** Optional pass through a local LLM to fix punctuation/accents. Off by default. Toggle and model selector in Settings → Model.

**Done when:** Enable toggle, select Gemma 2 2B GGUF, download, transcribe a sentence without punctuation — output has punctuation/accents. Disabling falls back to raw whisper output instantly.

---

### Task M6.1: `PostProcessor` trait + llama-cpp impl

**Files:**
- Create: `src-tauri/src/llm/mod.rs`, `src-tauri/src/llm/llama_cpp.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`

- [x] **Step 1: Add crate**

```toml
[target.'cfg(target_os = "macos")'.dependencies]
llama-cpp-2 = { version = "0.1", features = ["metal"] }

[target.'cfg(target_os = "windows")'.dependencies]
llama-cpp-2 = { version = "0.1", default-features = false }
```

- [x] **Step 2: Trait + impl**

Create `src-tauri/src/llm/mod.rs`:
```rust
pub mod llama_cpp;
use crate::error::AppError;

#[async_trait::async_trait]
pub trait PostProcessor: Send + Sync {
    async fn refine(&self, text: &str) -> Result<String, AppError>;
}
```

Add `async-trait = "0.1"` to deps.

Create `src-tauri/src/llm/llama_cpp.rs` — skeleton (the llama-cpp-2 API is in flux; the engineer should follow that crate's current example):
```rust
use super::PostProcessor;
use crate::error::AppError;
use std::path::PathBuf;

pub struct LlamaPostProcessor {
    model_path: PathBuf,
    // model handle goes here once loaded
}

impl LlamaPostProcessor {
    pub fn new(model_path: PathBuf) -> Self { Self { model_path } }
}

const SYSTEM_PROMPT: &str = "Você corrige textos transcritos: adicione pontuação, acentuação e capitalização corretas. NÃO altere o conteúdo, NÃO traduza, NÃO resuma. Responda apenas com o texto corrigido.";

#[async_trait::async_trait]
impl PostProcessor for LlamaPostProcessor {
    async fn refine(&self, text: &str) -> Result<String, AppError> {
        let mp = self.model_path.clone();
        let prompt = format!("{SYSTEM_PROMPT}\n\nTexto: {text}\n\nTexto corrigido:");
        // run on blocking thread because llama.cpp inference is synchronous CPU/GPU work
        let result = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
            // TODO(M6): replace with concrete llama-cpp-2 API once the project pins a version.
            // For now, return an error so the toggle won't silently break.
            Err(AppError::Llm(format!("llama-cpp-2 integration pending; model at {:?}", mp)))
        })
        .await
        .map_err(|e| AppError::Internal(format!("join: {e}")))??;
        Ok(result)
    }
}
```

> **Critical for the engineer:** the `llama-cpp-2` API evolves quickly. Before implementing, read https://github.com/utilityai/llama-cpp-rs and follow the latest example. Replace the `Err(...)` above with: load model once and cache, tokenize the prompt, run inference with `temperature=0.2`, decode response, strip the prompt prefix. Wrap the entire inference in `tokio::time::timeout(Duration::from_millis(cfg.llm_timeout_ms))`.

- [x] **Step 3: Add `pub mod llm;` to lib.rs**

- [x] **Step 4: Commit (scaffold)**

```bash
git add -A
git commit -m "feat(m6): PostProcessor trait + llama_cpp scaffold (impl pending API)"
```

---

### Task M6.2: Concrete llama-cpp integration

**Files:**
- Modify: `src-tauri/src/llm/llama_cpp.rs`

- [x] **Step 1: Read current llama-cpp-2 docs**

Open https://docs.rs/llama-cpp-2/latest/llama_cpp_2/ . Note the entry-point types (e.g. `LlamaBackend`, `LlamaModel`, `LlamaSession`).

- [x] **Step 2: Implement load + generate**

Replace the `TODO(M6)` block with code following the crate's hello-world example, parameterized by `model_path`. Cache the loaded model in `Mutex<Option<...>>` inside `LlamaPostProcessor`. Implement stop-tokens to halt at newline.

(Exact code depends on the API at the time of implementation; this plan deliberately doesn't pin it.)

- [x] **Step 3: Add unit test (mocked)**

Add to `llm/mod.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    struct EchoPP;
    #[async_trait::async_trait]
    impl PostProcessor for EchoPP {
        async fn refine(&self, t: &str) -> Result<String, AppError> { Ok(t.to_string()) }
    }
    #[tokio::test]
    async fn echo_works() {
        let p = EchoPP;
        assert_eq!(p.refine("hi").await.unwrap(), "hi");
    }
}
```

- [x] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(m6): real llama-cpp-2 inference with greedy decoding and timeout"
```

---

### Task M6.3: Wire post-processing into pipeline

**Files:**
- Modify: `src-tauri/src/main.rs`, `src-tauri/src/commands.rs`

- [ ] **Step 1: Load LLM lazily, like ASR**

Mirror `ensure_asr_loaded` with `ensure_llm_loaded` that returns `Option<Arc<dyn PostProcessor>>` (returning `None` when the toggle is off).

- [ ] **Step 2: Pipeline call**

After replacements, before paste:
```rust
let text = if cfg.post_processing_enabled {
    if let Some(pp) = me.ensure_llm_loaded().await? {
        let to_ms = cfg.llm_timeout_ms;
        match tokio::time::timeout(std::time::Duration::from_millis(to_ms), pp.refine(&text)).await {
            Ok(Ok(t)) => t,
            Ok(Err(e)) => { tracing::warn!("llm error, falling back: {e}"); text }
            Err(_) => { tracing::warn!("llm timeout, falling back"); text }
        }
    } else { text }
} else { text };
```

Also update history entry `post_processed: cfg.post_processing_enabled`.

- [ ] **Step 3: Add LLM models to catalog**

Append to `builtin_catalog()` in `models.rs`:
```rust
ModelInfo {
    id: "gemma-2-2b-it-q4_k_m".into(),
    kind: ModelKind::Llm,
    url: "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf".into(),
    sha256: "PLACEHOLDER_FILL_IN".into(),
    size_bytes: 1_640_000_000,
    display_name: "Gemma 2 2B Instruct Q4_K_M (1.6 GB)".into(),
},
```

- [ ] **Step 4: Expose toggle in Model tab**

Edit `ModelTab.tsx` to show:
```tsx
<div className="flex items-center justify-between mt-6">
  <Label>Post-processing (LLM)</Label>
  <Switch
    checked={config.post_processing_enabled}
    onCheckedChange={(v) => update({ post_processing_enabled: v })}
  />
</div>
{config.post_processing_enabled && (
  <>
    <h3 className="font-medium mt-4">Post-processing model</h3>
    {models.filter(m => m.kind === "llm").map(m => (
      <ModelCard key={m.id} id={m.id} displayName={m.display_name} sizeBytes={m.size_bytes}
        isSelected={config.llm_model === m.id} onSelect={() => update({ llm_model: m.id })} />
    ))}
  </>
)}
```

- [ ] **Step 5: Smoke**

Enable toggle, download Gemma. Transcribe "ola tudo bem com voce" — should paste "Olá, tudo bem com você?".

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(m6): optional llm post-processing wired into pipeline with timeout"
```

---

# Milestone M7 — Onboarding + Permissions

**Goal:** First-run experience: 3 steps (welcome, permissions, model download). Detect macOS Accessibility / mic permission state and deep-link to System Settings if denied.

**Done when:** Delete `~/Library/Application Support/whisper-hotkey/`. Launch app. Onboarding window opens automatically. Walk through welcome → permissions (click "Open Settings" deep-link works) → model download. After completion, main app runs normally. Re-launching skips onboarding.

---

### Task M7.1: Onboarding window scaffolding

**Files:**
- Modify: `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`
- Create: `src/windows/onboarding/OnboardingWindow.tsx`, `src/windows/onboarding/steps/WelcomeStep.tsx`, `src/windows/onboarding/steps/PermissionsStep.tsx`, `src/windows/onboarding/steps/ModelDownloadStep.tsx`

- [ ] **Step 1: Reuse main window for onboarding**

Onboarding lives in the main window. In `App.tsx`, check `config.onboarding_complete` (add this field to `Config` default `false`):
```tsx
import { useConfigStore } from "@/store/configStore";
import { useEffect } from "react";
import { OnboardingWindow } from "@/windows/onboarding/OnboardingWindow";
import { SettingsWindow } from "@/windows/settings/SettingsWindow";

export default function App() {
  const { config, load } = useConfigStore();
  useEffect(() => { void load(); }, [load]);
  if (!config) return null;
  return config.onboarding_complete ? <SettingsWindow /> : <OnboardingWindow />;
}
```

Add field to `Config` (Rust + TS), default false.

- [ ] **Step 2: First-run logic**

In `main.rs` `setup`, after loading config: if `!cfg.onboarding_complete`, show the main window automatically. Otherwise leave it hidden.

- [ ] **Step 3: Welcome step**

Create `src/windows/onboarding/steps/WelcomeStep.tsx`:
```tsx
import { Button } from "@/components/ui/button";

export function WelcomeStep({ onNext }: { onNext: () => void }) {
  return (
    <div className="text-center max-w-md mx-auto">
      <h1 className="text-2xl font-semibold mb-2">Welcome to Whisper Hotkey</h1>
      <p className="text-sm text-muted-foreground mb-6">
        Voice dictation, 100% local. Two quick steps and you're ready.
      </p>
      <Button onClick={onNext}>Get started</Button>
    </div>
  );
}
```

- [ ] **Step 4: Container**

Create `src/windows/onboarding/OnboardingWindow.tsx`:
```tsx
import { useState } from "react";
import { WelcomeStep } from "./steps/WelcomeStep";
import { PermissionsStep } from "./steps/PermissionsStep";
import { ModelDownloadStep } from "./steps/ModelDownloadStep";
import { useConfigStore } from "@/store/configStore";

export function OnboardingWindow() {
  const [step, setStep] = useState(0);
  const { update } = useConfigStore();
  async function finish() {
    await update({ onboarding_complete: true });
    // restart-to-settings is fine; or just hide and re-render
    window.location.reload();
  }
  return (
    <div className="h-screen p-10">
      {step === 0 && <WelcomeStep onNext={() => setStep(1)} />}
      {step === 1 && <PermissionsStep onNext={() => setStep(2)} />}
      {step === 2 && <ModelDownloadStep onDone={finish} />}
    </div>
  );
}
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(m7): onboarding window scaffolding with 3 steps"
```

---

### Task M7.2: Permission helpers

**Files:**
- Create: `src-tauri/src/permissions.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`

- [ ] **Step 1: Implement checks**

Create `src-tauri/src/permissions.rs`:
```rust
use serde::Serialize;

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct PermissionStatus {
    pub accessibility: bool,
    pub microphone: bool,
}

#[cfg(target_os = "macos")]
pub fn check() -> PermissionStatus {
    use core_foundation::base::TCFType;
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    let accessibility = unsafe { AXIsProcessTrusted() };
    // Microphone permission check on macOS requires AVFoundation. Tauri's mic permission
    // is requested implicitly on first cpal stream; we report unknown=true here and let
    // the first recording attempt surface a real error.
    PermissionStatus { accessibility, microphone: true }
}

#[cfg(target_os = "windows")]
pub fn check() -> PermissionStatus {
    PermissionStatus { accessibility: true, microphone: true }
}

#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .status()?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn open_accessibility_settings() -> std::io::Result<()> { Ok(()) }
```

Note: the `extern "C"` for `AXIsProcessTrusted` requires linking ApplicationServices. Add to `Cargo.toml` macOS target:
```toml
[target.'cfg(target_os = "macos")'.dependencies.cocoa-foundation]
version = "0.2"
```
and in `build.rs` (create if needed):
```rust
fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    tauri_build::build();
}
```

Add `pub mod permissions;` to `lib.rs`.

- [ ] **Step 2: Commands**

Add to `commands.rs`:
```rust
#[tauri::command]
pub fn check_permissions() -> crate::permissions::PermissionStatus { crate::permissions::check() }

#[tauri::command]
pub fn open_accessibility_panel() -> Result<(), AppErrorDto> {
    crate::permissions::open_accessibility_settings()
        .map_err(|e| AppError::Storage(e).into())
}
```

Register both.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m7): permission check + deep-link to macOS accessibility panel"
```

---

### Task M7.3: Permissions step UI

**Files:**
- Create: `src/windows/onboarding/steps/PermissionsStep.tsx`

- [ ] **Step 1: Component**

```tsx
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import type { PermissionStatus } from "@/ipc/generated/PermissionStatus";

export function PermissionsStep({ onNext }: { onNext: () => void }) {
  const [status, setStatus] = useState<PermissionStatus | null>(null);

  async function refresh() { setStatus(await invoke("check_permissions")); }
  useEffect(() => {
    void refresh();
    const i = setInterval(refresh, 1500);
    return () => clearInterval(i);
  }, []);

  if (!status) return null;
  const allGood = status.accessibility && status.microphone;

  return (
    <div className="max-w-md mx-auto">
      <h2 className="text-xl font-semibold mb-4">Permissions</h2>
      <div className="space-y-4">
        <Row
          ok={status.accessibility}
          title="Accessibility"
          help="Required for global hotkey and pasting into other apps."
          action={
            <Button variant="outline" onClick={() => invoke("open_accessibility_panel")}>
              Open System Settings
            </Button>
          }
        />
        <Row
          ok={status.microphone}
          title="Microphone"
          help="Will be requested when you start your first recording."
        />
      </div>
      <div className="mt-6 flex justify-end">
        <Button disabled={!allGood} onClick={onNext}>Continue</Button>
      </div>
    </div>
  );
}

function Row({ ok, title, help, action }: { ok: boolean; title: string; help: string; action?: React.ReactNode }) {
  return (
    <div className="flex items-center gap-3">
      <span className={`inline-block h-3 w-3 rounded-full ${ok ? "bg-green-500" : "bg-red-500"}`} />
      <div className="flex-1">
        <div className="font-medium">{title}</div>
        <div className="text-xs text-muted-foreground">{help}</div>
      </div>
      {action}
    </div>
  );
}
```

- [ ] **Step 2: Smoke**

Reset app data, launch, click "Open System Settings", grant access, watch dot turn green, click Continue.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m7): permissions step with live status polling and deep-link"
```

---

### Task M7.4: Model download step

**Files:**
- Create: `src/windows/onboarding/steps/ModelDownloadStep.tsx`

- [ ] **Step 1: Component**

```tsx
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ModelCard } from "@/components/ModelCard";
import { Button } from "@/components/ui/button";
import type { ModelInfo } from "@/ipc/generated/ModelInfo";

export function ModelDownloadStep({ onDone }: { onDone: () => void }) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [done, setDone] = useState<Record<string, boolean>>({});

  useEffect(() => {
    void invoke<ModelInfo[]>("list_models").then(async (ms) => {
      setModels(ms.filter(m => m.kind === "asr"));
      const map: Record<string, boolean> = {};
      for (const m of ms) map[m.id] = await invoke("is_model_present", { id: m.id });
      setDone(map);
    });
  }, []);

  const any = Object.values(done).some(Boolean);

  return (
    <div className="max-w-xl mx-auto">
      <h2 className="text-xl font-semibold mb-2">Pick a transcription model</h2>
      <p className="text-sm text-muted-foreground mb-6">You can change this anytime in Settings.</p>
      <div className="space-y-3">
        {models.map((m) => (
          <ModelCard key={m.id} id={m.id} displayName={m.display_name} sizeBytes={m.size_bytes}
            isSelected={false} onSelect={() => {}} />
        ))}
      </div>
      <div className="mt-6 flex justify-end">
        <Button disabled={!any} onClick={onDone}>Finish</Button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Smoke**

Walk through full onboarding from a fresh data dir.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m7): model download step in onboarding"
```

---

# Milestone M8 — Polish

**Goal:** Theme works (light/dark/system), animations feel right, error toasts surface backend errors, logs roll over.

---

### Task M8.1: Apply theme

**Files:**
- Create: `src/lib/theme.ts`
- Modify: `src/main.tsx`, `src/overlay-main.tsx`, `src/history-main.tsx`

- [ ] **Step 1: Theme helper**

```ts
// src/lib/theme.ts
import { useConfigStore } from "@/store/configStore";
import { useEffect } from "react";

export function useApplyTheme() {
  const theme = useConfigStore((s) => s.config?.theme);
  useEffect(() => {
    if (!theme) return;
    const sysDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const dark = theme === "dark" || (theme === "system" && sysDark);
    document.documentElement.classList.toggle("dark", dark);
  }, [theme]);
}
```

Call `useApplyTheme()` once at the top of `App` (settings/onboarding window). For overlay/history, do the same after loading config.

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "feat(m8): theme — system/light/dark with live switching"
```

---

### Task M8.2: Toast on errors

**Files:**
- Modify: `src/App.tsx`
- Run: `pnpm dlx shadcn@latest add toast sonner`

- [ ] **Step 1: Listen for errors and show toast**

In `App.tsx`:
```tsx
import { Toaster, toast } from "sonner";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

useEffect(() => {
  const u = listen<{ kind: string; message: string }>("error", (e) => {
    toast.error(`${e.payload.kind}: ${e.payload.message}`);
  });
  return () => { u.then(f => f()); };
}, []);
```

Add `<Toaster />` at the bottom of the layout.

In Rust, emit `error` events from the pipeline's failure branches:
```rust
let _ = handle.emit("error", e.to_dto());
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "feat(m8): backend errors surface as toast notifications"
```

---

### Task M8.3: File-based rotating logs

**Files:**
- Modify: `src-tauri/src/logging.rs`, `src-tauri/Cargo.toml`

- [ ] **Step 1: Add rolling appender**

```toml
tracing-appender = "0.2"
```

Replace `logging.rs`:
```rust
use crate::models::app_data_dir;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init() -> tracing_appender::non_blocking::WorkerGuard {
    let dir = app_data_dir().join("logs");
    std::fs::create_dir_all(&dir).ok();
    let appender = RollingFileAppender::new(Rotation::DAILY, dir, "app.log");
    let (nb, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,whisper_hotkey=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_writer(nb))
        .with(fmt::layer().with_target(false))
        .init();
    tracing::info!("logging initialized");
    guard
}
```

Update `main.rs`: `let _guard = logging::init();` and keep `_guard` alive until exit.

- [ ] **Step 2: Add "Open logs folder" tray entry**

Add menu item that calls `tauri_plugin_shell::ShellExt::shell().open(...)` on the logs dir.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m8): daily-rotating file logs + tray entry to open logs folder"
```

---

# Milestone M9 — Distribution

**Goal:** Reproducible signed installers for macOS and Windows via GitHub Actions on tag push.

---

### Task M9.1: macOS signing + notarization

**Files:**
- Modify: `src-tauri/tauri.conf.json`, `.github/workflows/release.yml`

- [ ] **Step 1: Configure tauri.conf.json bundle**

```json
"bundle": {
  "active": true,
  "targets": ["app", "dmg"],
  "macOS": {
    "frameworks": [],
    "minimumSystemVersion": "12.0",
    "exceptionDomain": "",
    "signingIdentity": "Developer ID Application: ${SIGNING_IDENTITY}",
    "entitlements": "entitlements.plist"
  },
  "identifier": "com.lucabe.whisperhotkey",
  "category": "Utility"
}
```

Create `src-tauri/entitlements.plist`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.device.audio-input</key><true/>
  <key>com.apple.security.cs.allow-jit</key><true/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key><true/>
  <key>com.apple.security.cs.disable-library-validation</key><true/>
</dict>
</plist>
```

- [ ] **Step 2: Release workflow**

Create `.github/workflows/release.yml`:
```yaml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        os: [macos-14, windows-2022]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm }
      - uses: dtolnay/rust-toolchain@stable
      - run: pnpm install --frozen-lockfile
      - name: Build (macOS)
        if: runner.os == 'macOS'
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: pnpm tauri build
      - name: Build (Windows)
        if: runner.os == 'Windows'
        run: pnpm tauri build
      - uses: softprops/action-gh-release@v2
        with:
          files: |
            src-tauri/target/release/bundle/dmg/*.dmg
            src-tauri/target/release/bundle/nsis/*.exe
```

- [ ] **Step 3: Document secret setup**

Append to `README.dev.md` a section "Release secrets" listing required GitHub secrets and how to generate the macOS certificate.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "ci(m9): release workflow signs macos and bundles windows installer"
```

---

### Task M9.2: Windows installer (NSIS)

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Configure NSIS**

```json
"windows": {
  "wix": null,
  "nsis": {
    "installerIcon": "icons/icon.ico",
    "license": "../LICENSE",
    "displayLanguageSelector": false
  }
}
```

(Ensure `LICENSE` file at repo root — copy from existing if present.)

- [ ] **Step 2: Test locally on Windows or via CI**

Push tag `v0.9.0-rc1`, verify CI produces both `.dmg` and `.exe` installer.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(m9): nsis installer config for windows"
```

---

# Milestone M10 — Retire Python

**Goal:** Remove the Python codebase, update README to describe the new app only, cut v1.0.0.

---

### Task M10.1: Delete Python files

- [ ] **Step 1: Remove**

```bash
git rm whisper_hotkey.py install.sh launcher.sh build-dmg.sh
git rm -r pkg/
```

- [ ] **Step 2: Commit**

```bash
git commit -m "chore(m10): retire python implementation"
```

---

### Task M10.2: Rewrite README.md

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Rewrite for the Tauri app**

Replace `README.md` content with new install instructions (download .dmg / .exe from Releases), updated screenshots if available, keep the credit to the original `dpejoh/whisper-hotkey` and `lffelgueiras` macOS Python fork, mention the new Windows support.

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs(m10): rewrite README for tauri app with macos+windows installers"
```

---

### Task M10.3: Tag v1.0.0

- [ ] **Step 1: Tag and push**

```bash
git tag v1.0.0
git push origin v1.0.0
```

CI builds + publishes signed installers. Verify the release page on GitHub has both artifacts.

- [ ] **Step 2: Smoke install**

On a fresh Mac and a fresh Windows machine, download the installer, install, run, complete onboarding, transcribe. Confirm it works.

---

# Cross-cutting follow-ups (do as you go, not all at once)

- **Replace placeholder SHA256 values** in `models.rs` — must be done before any user-facing release. Pin to specific revisions on Hugging Face.
- **Replace `acceptFirstMouse: false` validation** — test that overlay clicks don't steal focus on macOS; the current setup relies on `focus: false`.
- **CUDA support for Windows** — not in v1. File issue, address in v1.1.
- **Test microphone visualizer** in Settings → Model (spec §8 lists it) — not strictly required for v1; defer to v1.1 if time-pressed.

---

# Self-review

(See `docs/superpowers/specs/2026-05-12-migration-typescript-design.md`.)

**Spec coverage:**
- §2 stack decisions → all reflected in M0–M1 deps.
- §3 architecture → realized in M0 (scaffold), M1 (backend modules), M2 (overlay).
- §4 folder structure → matches the file-structure section above.
- §5 fluxo de dados → M1.10 wires it; M6 adds LLM step.
- §6 modelo de dados → M3 (config), M4 (history), M3 (models manifest in `builtin_catalog`).
- §7 IPC contracts → M3 (config), M3.6 (models), M4 (history), M6 (post-processing toggle), M7 (permissions). `register_hotkey` is implicit in `update_config` (M3.5). `test_microphone` deferred to v1.1 (called out in follow-ups).
- §8 UI → M2 (overlay), M3 (general+model), M4 (history), M5 (vocabulary), M7 (onboarding). Microphone visualizer deferred.
- §9 permissions → M7.2 / M7.3.
- §10 error handling → M1.1 (types), M8.2 (toast surface).
- §11 testing → unit tests in every module, smoke test in M1.11, Playwright deferred to v1.1 (frontend e2e is set up but spec scenarios are not yet written — flagged as follow-up).
- §12 build/distribution → M9.
- §13 milestones → 1:1 with this plan's structure.
- §14 risks → addressed inline (e.g., placeholder SHA flag, CPU-only Windows whisper).

**Gaps deliberately deferred:**
- Playwright e2e specs (`tests/settings.spec.ts`, `tests/onboarding.spec.ts`) listed in file structure but not exercised by a task. Add v1.1 plan.
- Microphone test/visualizer (spec §8 melhorias).
- Right-click overlay menu (spec §8 melhorias).
- Theme override per-window for overlay (M8 only applies to main window; overlay has its own root). Engineer should call `useApplyTheme` in `overlay-main.tsx` too — already noted in M8.1.

**Placeholder scan:** "Replace placeholder SHA256" appears intentionally with a clear remediation procedure (M1.10 step 3, M10 follow-ups). `TODO(M6)` in M6.1 is intentional — flagged with prose and replaced in M6.2.

**Type consistency check:** `Config`, `ModelInfo`, `ModelKind`, `HistoryEntry`, `ReplacementRule`, `OverlayPosition`, `Theme`, `RecordingState`, `ErrorKind`, `AppErrorDto`, `PermissionStatus` all defined once and used consistently. `RecordingState` serializes as `"idle" | "recording" | "transcribing"` in both Rust (via `#[serde(rename_all = "lowercase")]`) and the TS event payload — verified.

---

# Execution

Plan complete and saved to `docs/superpowers/plans/2026-05-12-migration-typescript.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using `executing-plans`, batch execution with checkpoints.

Which approach?

---

# Task checklist (master tracker)

> **For the executing agent:** mark each task `[x]` as soon as its final commit lands. One milestone per group. If a task is split into sub-tasks while executing, add nested items underneath — do not mutate the existing line. Tasks must be completed in order within a milestone; milestones must be completed in numeric order (M0 → M10).

## M0 — Scaffold

- [x] **M0.1** Initialize Tauri project scaffold
- [x] **M0.2** Pin versions and add core dev dependencies
- [x] **M0.3** Set up Tailwind CSS and shadcn/ui
- [x] **M0.4** Add Tauri tray with Quit menu
- [x] **M0.5** Add `tracing` logging bootstrap
- [x] **M0.6** CI workflow — build + test on macOS and Windows

## M1 — Pipeline core

- [x] **M1.1** Define `AppError` and `ErrorKind`
- [x] **M1.2** Audio capture module — start/stop/get-samples
- [x] **M1.3** Vocabulary builder
- [x] **M1.4** `Transcriber` trait + whisper.cpp implementation
- [x] **M1.5** Paste module — macOS
- [x] **M1.6** Paste module — Windows
- [x] **M1.7** State machine actor
- [x] **M1.8** Global hotkey wiring
- [x] **M1.9** Model bootstrap — download whisper-tiny on first run
- [x] **M1.10** Wire pipeline end-to-end through `main.rs`
- [x] **M1.11** Smoke-test transcription with a generated audio sample
- [x] **M1.12** Documentation — `README.dev.md`

## M2 — Overlay window

- [x] **M2.1** Multi-window setup in `tauri.conf.json`
- [x] **M2.2** OverlayWindow React component with three states
- [x] **M2.3** Emit state events from Rust
- [x] **M2.4** Overlay position config (read-only for now)

## M3 — Settings: General + Model tabs

- [x] **M3.1** Config schema + storage
- [x] **M3.2** Tauri commands for config get/update
- [x] **M3.3** Settings window — shell
- [x] **M3.4** General tab
- [x] **M3.5** React to hotkey config changes at runtime
- [x] **M3.6** Model tab with download progress
- [x] **M3.7** Wire up `ts-rs` generation
- [x] **M3.8** Update `README.dev.md` and lock M3

## M4 — History

- [x] **M4.1** History storage
- [x] **M4.2** Append history during pipeline
- [x] **M4.3** History commands + window

## M5 — Vocabulary + Replacements

- [x] **M5.1** Replacements engine
- [x] **M5.2** Vocabulary tab UI

## M6 — LLM post-processing

- [x] **M6.1** `PostProcessor` trait + llama-cpp scaffold
- [x] **M6.2** Concrete llama-cpp integration
- [ ] **M6.3** Wire post-processing into pipeline

## M7 — Onboarding + Permissions

- [ ] **M7.1** Onboarding window scaffolding
- [ ] **M7.2** Permission helpers
- [ ] **M7.3** Permissions step UI
- [ ] **M7.4** Model download step

## M8 — Polish

- [ ] **M8.1** Apply theme
- [ ] **M8.2** Toast on errors
- [ ] **M8.3** File-based rotating logs

## M9 — Distribution

- [ ] **M9.1** macOS signing + notarization
- [ ] **M9.2** Windows installer (NSIS)

## M10 — Retire Python

- [ ] **M10.1** Delete Python files
- [ ] **M10.2** Rewrite `README.md`
- [ ] **M10.3** Tag v1.0.0

## Cross-cutting follow-ups (track separately — not blockers)

- [ ] Replace placeholder SHA256 values in `models.rs` with real hashes before any user-facing release
- [ ] Validate that overlay clicks don't steal focus on macOS (`acceptFirstMouse: false` + `focus: false`)
- [ ] Apply `useApplyTheme()` inside `overlay-main.tsx` and `history-main.tsx` (M8.1 covers the main window only)
- [ ] Playwright e2e specs for settings and onboarding (deferred to v1.1)
- [ ] Microphone test/visualizer in Settings → Model (deferred to v1.1)
- [ ] Right-click overlay menu (deferred to v1.1)
- [ ] CUDA support for Windows whisper.cpp (deferred to v1.1)

**Totals:** 50 sequenced tasks across 11 milestones, 7 cross-cutting follow-ups.
