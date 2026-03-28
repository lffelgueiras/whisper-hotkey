# Whisper Hotkey — Installer Design

## Overview

A shell script (`install.sh`) that performs a complete installation of Whisper Hotkey on any Mac with Apple Silicon. The user runs a single command in Terminal and gets a fully working app.

## Target audience

Non-technical macOS users with Apple Silicon Macs (M1/M2/M3/M4). No prior Python, Homebrew, or developer tools required.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Distribution format | Shell script (`install.sh`) | Simplest to create/maintain; handles Python + ML model deps well |
| Python installation | Auto-install via Homebrew | Users may not have Python installed |
| Model download timing | During installation | "Install and it works" experience |
| LLM post-processing model | Optional, user chooses during install | Not all Macs have enough RAM (8GB vs 16GB+) |

## Installation flow

### Step 1: Verify Apple Silicon

Abort with clear message if `uname -m != arm64`. MLX only works on Apple Silicon.

### Step 2: Install Homebrew

Check if `brew` exists. If not, install via the official Homebrew install script.

### Step 3: Install Python 3

Run `brew install python3` if `python3` is not available.

### Step 4: Install system dependencies

Run `brew install portaudio` (required by sounddevice/PortAudio for audio recording).

### Step 5: Create virtual environment

Create a dedicated virtualenv at `~/.whisper-hotkey/venv/` to isolate dependencies.

### Step 6: Install Python packages

Using pip in the venv, install:
- numpy
- sounddevice
- pyperclip
- PySide6
- pyobjc-framework-Cocoa
- pyobjc-framework-Quartz
- mlx-qwen3-asr
- mlx-lm
- transformers

### Step 7: Download ASR model

Download `Qwen/Qwen3-ASR-0.6B` (~1.2GB) via `huggingface-cli download`. This is the core speech-to-text model and is always required.

### Step 8: Ask about LLM post-processing

Prompt the user:
> "Deseja instalar o modelo de pos-processamento de texto (~2.5GB)? Recomendado para Macs com 16GB+ de RAM. [s/N]"

If yes, download `mlx-community/Qwen3.5-4B-MLX-4bit`.
If no, set `post_process: false` in the default config.

### Step 9: Copy application files

Copy `whisper_hotkey.py` and `icons/` to `~/.whisper-hotkey/`.

### Step 10: Build app bundle

- Compile `launcher.c` with `clang -O2 -framework Foundation`
- Assemble `.app` bundle structure (Info.plist, icon, compiled binary)
- The launcher prioritizes the venv Python (`~/.whisper-hotkey/venv/bin/python3`)
- Copy or symlink the `.app` to `/Applications/Whisper Hotkey.app`

### Step 11: Final message

Display:
- Success confirmation
- How to open the app (from Applications or Spotlight)
- Default hotkey (`cmd+shift+space`)
- How to change settings

## Installed file structure

```
~/.whisper-hotkey/
├── venv/                    # Python virtualenv with all dependencies
├── whisper_hotkey.py        # Main application script
├── icons/                   # Application icons
└── config.json              # Created on first launch

/Applications/Whisper Hotkey.app/
├── Contents/
│   ├── Info.plist
│   ├── MacOS/Whisper Hotkey  # Compiled C launcher
│   └── Resources/icon.icns
```

Models are cached by Hugging Face Hub at `~/.cache/huggingface/hub/`.

## Error handling

- Each step checks for success before proceeding
- On failure, display a clear message indicating what went wrong and how to fix it
- The script is idempotent — safe to re-run if it fails partway through
- Skip steps that are already completed (Homebrew already installed, packages already present, etc.)

## Launcher modification

The `launcher.c` candidate list is updated to prioritize the venv Python:

```c
// Priority: whisper-hotkey venv > anaconda > homebrew > system
snprintf(candidates[n++], 4096, "%s/.whisper-hotkey/venv/bin/python3", home);
snprintf(candidates[n++], 4096, "%s/anaconda3/bin/python3", home);
// ... existing candidates
```

## Out of scope

- Auto-update mechanism
- Uninstaller (can be added later)
- Intel Mac support (MLX is Apple Silicon only)
- Linux/Windows support
