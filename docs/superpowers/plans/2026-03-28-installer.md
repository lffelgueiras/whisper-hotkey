# Whisper Hotkey Installer — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a single `install.sh` that installs Whisper Hotkey from scratch on any Apple Silicon Mac, including Homebrew, Python, dependencies, ML models, and the .app bundle.

**Architecture:** A self-contained bash script with helper functions for each install step. The launcher.c is updated to prioritize the venv Python. The script bundles all logic inline — no external config files needed.

**Tech Stack:** Bash, Homebrew, Python 3 venv, pip, huggingface-cli, clang, iconutil

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `install.sh` | Create | Main installer script |
| `launcher.c` | Modify | Add venv Python as first candidate |

---

### Task 1: Update launcher.c to prioritize venv Python

**Files:**
- Modify: `launcher.c:28-35`

- [ ] **Step 1: Add venv path as first candidate**

In `launcher.c`, after the `if (home)` block opens, add the venv path before anaconda:

```c
if (home) {
    snprintf(candidates[n++], 4096, "%s/.whisper-hotkey/venv/bin/python3", home);
    snprintf(candidates[n++], 4096, "%s/anaconda3/bin/python3", home);
    snprintf(candidates[n++], 4096, "%s/miniconda3/bin/python3", home);
    snprintf(candidates[n++], 4096, "%s/miniforge3/bin/python3", home);
}
```

- [ ] **Step 2: Verify it compiles**

```bash
clang -O2 -framework Foundation -o /tmp/test-launcher launcher.c && echo "OK"
```

Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add launcher.c
git commit -m "feat: prioritize whisper-hotkey venv Python in launcher"
```

---

### Task 2: Create install.sh

**Files:**
- Create: `install.sh`

- [ ] **Step 1: Write the complete install.sh script**

```bash
#!/bin/bash
set -euo pipefail

# ── Colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

INSTALL_DIR="$HOME/.whisper-hotkey"
VENV_DIR="$INSTALL_DIR/venv"
APP_NAME="Whisper Hotkey"
APP_DIR="/Applications/$APP_NAME.app"

info()    { echo -e "${BLUE}==>${NC} ${BOLD}$1${NC}"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn()    { echo -e "${YELLOW}⚠${NC} $1"; }
fail()    { echo -e "${RED}✗ $1${NC}"; exit 1; }

# ── Step 1: Verify Apple Silicon ──────────────────────────────────────────────
info "Verificando arquitetura..."
if [ "$(uname -m)" != "arm64" ]; then
    fail "Whisper Hotkey requer Apple Silicon (M1/M2/M3/M4). Este Mac usa $(uname -m)."
fi
success "Apple Silicon detectado"

# ── Step 2: Install Homebrew ──────────────────────────────────────────────────
info "Verificando Homebrew..."
if ! command -v brew &>/dev/null; then
    info "Instalando Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    # Add brew to PATH for this session
    eval "$(/opt/homebrew/bin/brew shellenv)"
    success "Homebrew instalado"
else
    success "Homebrew já instalado"
fi

# ── Step 3: Install Python 3 ─────────────────────────────────────────────────
info "Verificando Python 3..."
if ! brew list python3 &>/dev/null; then
    info "Instalando Python 3 via Homebrew..."
    brew install python3
    success "Python 3 instalado"
else
    success "Python 3 já instalado"
fi

# ── Step 4: Install system dependencies ───────────────────────────────────────
info "Verificando dependencias do sistema..."
if ! brew list portaudio &>/dev/null; then
    info "Instalando PortAudio..."
    brew install portaudio
    success "PortAudio instalado"
else
    success "PortAudio já instalado"
fi

# ── Step 5: Create install directory ──────────────────────────────────────────
info "Criando diretorio de instalacao..."
mkdir -p "$INSTALL_DIR"
success "Diretorio: $INSTALL_DIR"

# ── Step 6: Create virtual environment ────────────────────────────────────────
info "Configurando ambiente Python..."
if [ ! -d "$VENV_DIR" ]; then
    "$(brew --prefix python3)/bin/python3" -m venv "$VENV_DIR"
    success "Ambiente virtual criado"
else
    success "Ambiente virtual já existe"
fi

PIP="$VENV_DIR/bin/pip"
PYTHON="$VENV_DIR/bin/python3"

# Upgrade pip
"$PIP" install --upgrade pip --quiet

# ── Step 7: Install Python packages ──────────────────────────────────────────
info "Instalando pacotes Python (isso pode levar alguns minutos)..."
"$PIP" install --quiet \
    numpy \
    sounddevice \
    pyperclip \
    PySide6 \
    pyobjc-framework-Cocoa \
    pyobjc-framework-Quartz \
    mlx-qwen3-asr \
    mlx-lm \
    transformers
success "Pacotes Python instalados"

# ── Step 8: Download ASR model ────────────────────────────────────────────────
info "Baixando modelo de transcricao Qwen3-ASR-0.6B (~1.2GB)..."
"$VENV_DIR/bin/huggingface-cli" download Qwen/Qwen3-ASR-0.6B --quiet
success "Modelo de transcricao baixado"

# ── Step 9: Ask about LLM post-processing ────────────────────────────────────
INSTALL_LLM=false
echo ""
echo -e "${BOLD}Deseja instalar o modelo de pos-processamento de texto (~2.5GB)?${NC}"
echo "Esse modelo corrige pontuacao, acentuacao e formatacao das transcricoes."
echo "Recomendado para Macs com 16GB+ de RAM."
echo ""
read -p "Instalar modelo de pos-processamento? [s/N] " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Ss]$ ]]; then
    INSTALL_LLM=true
    info "Baixando modelo de pos-processamento Qwen3.5-4B (~2.5GB)..."
    "$VENV_DIR/bin/huggingface-cli" download mlx-community/Qwen3.5-4B-MLX-4bit --quiet
    success "Modelo de pos-processamento baixado"
else
    info "Pulando modelo de pos-processamento"
fi

# ── Step 10: Copy application files ──────────────────────────────────────────
info "Copiando arquivos da aplicacao..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cp "$SCRIPT_DIR/whisper_hotkey.py" "$INSTALL_DIR/"
cp -r "$SCRIPT_DIR/icons" "$INSTALL_DIR/"
success "Arquivos copiados"

# ── Step 11: Build app bundle ─────────────────────────────────────────────────
info "Construindo aplicacao..."

# Compile launcher
clang -O2 -framework Foundation -o "/tmp/whisper-hotkey-launcher" "$SCRIPT_DIR/launcher.c"

# Create .app structure
APP_CONTENTS="$APP_DIR/Contents"
mkdir -p "$APP_CONTENTS/MacOS"
mkdir -p "$APP_CONTENTS/Resources"

# Copy binary
cp "/tmp/whisper-hotkey-launcher" "$APP_CONTENTS/MacOS/Whisper Hotkey"
rm -f "/tmp/whisper-hotkey-launcher"

# Copy Info.plist
cat > "$APP_CONTENTS/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Whisper Hotkey</string>
    <key>CFBundleDisplayName</key>
    <string>Whisper Hotkey</string>
    <key>CFBundleIdentifier</key>
    <string>com.whisperhotkey.app</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleExecutable</key>
    <string>Whisper Hotkey</string>
    <key>CFBundleIconFile</key>
    <string>icon</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSMicrophoneUsageDescription</key>
    <string>Whisper Hotkey needs microphone access to record and transcribe speech.</string>
    <key>NSAppleEventsUsageDescription</key>
    <string>Whisper Hotkey needs automation access to paste transcribed text.</string>
</dict>
</plist>
PLIST

# Generate icon.icns from PNG
if [ -f "$SCRIPT_DIR/icons/whisper.png" ]; then
    ICONSET_DIR="/tmp/whisper-hotkey.iconset"
    mkdir -p "$ICONSET_DIR"
    sips -z 16 16     "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_16x16.png"    &>/dev/null
    sips -z 32 32     "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_16x16@2x.png" &>/dev/null
    sips -z 32 32     "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_32x32.png"    &>/dev/null
    sips -z 64 64     "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_32x32@2x.png" &>/dev/null
    sips -z 128 128   "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_128x128.png"  &>/dev/null
    sips -z 256 256   "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_128x128@2x.png" &>/dev/null
    sips -z 256 256   "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_256x256.png"  &>/dev/null
    sips -z 512 512   "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_256x256@2x.png" &>/dev/null
    sips -z 512 512   "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_512x512.png"  &>/dev/null
    sips -z 1024 1024 "$SCRIPT_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_512x512@2x.png" &>/dev/null
    iconutil -c icns "$ICONSET_DIR" -o "$APP_CONTENTS/Resources/icon.icns"
    rm -rf "$ICONSET_DIR"
    success "Icone gerado"
elif [ -f "$SCRIPT_DIR/Whisper Hotkey.app/Contents/Resources/icon.icns" ]; then
    cp "$SCRIPT_DIR/Whisper Hotkey.app/Contents/Resources/icon.icns" "$APP_CONTENTS/Resources/"
    success "Icone copiado"
fi

# Code sign
codesign --force --deep --sign - "$APP_DIR" 2>/dev/null || true
success "Aplicacao construida em $APP_DIR"

# ── Step 12: Create default config ───────────────────────────────────────────
CONFIG_DIR="$HOME/.whisper_hotkey"
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.json" ]; then
    if [ "$INSTALL_LLM" = true ]; then
        PP_VALUE="true"
    else
        PP_VALUE="false"
    fi
    cat > "$CONFIG_DIR/config.json" << CONF
{
    "model": "Qwen/Qwen3-ASR-0.6B",
    "language": "pt",
    "hotkey": "cmd+shift+space",
    "auto_paste": true,
    "post_process": $PP_VALUE,
    "post_process_model": "mlx-community/Qwen3.5-4B-MLX-4bit",
    "overlay_position": "top-center",
    "history_limit": 50,
    "theme": "auto",
    "vocabulary": [],
    "replacements": []
}
CONF
    success "Configuracao criada"
else
    success "Configuracao existente mantida"
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}${BOLD}  Whisper Hotkey instalado com sucesso!${NC}"
echo -e "${GREEN}${BOLD}════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${BOLD}Como usar:${NC}"
echo -e "  1. Abra o app pelo ${BOLD}Spotlight${NC} (Cmd+Space) ou ${BOLD}Applications${NC}"
echo -e "  2. Pressione ${BOLD}cmd+shift+space${NC} para gravar"
echo -e "  3. Pressione novamente para parar e transcrever"
echo -e "  4. O texto sera colado automaticamente onde o cursor estiver"
echo ""
echo -e "  ${BOLD}Configuracoes:${NC} clique no icone da bandeja > Settings"
echo ""
echo -e "  ${BOLD}Nota:${NC} Na primeira execucao, o macOS pedira permissao"
echo -e "  para Microfone e Acessibilidade. Aceite ambas."
echo ""
```

- [ ] **Step 2: Make it executable**

```bash
chmod +x install.sh
```

- [ ] **Step 3: Verify syntax**

```bash
bash -n install.sh && echo "Syntax OK"
```

Expected: `Syntax OK`

- [ ] **Step 4: Commit**

```bash
git add install.sh
git commit -m "feat: add install.sh for complete macOS installation"
```

---

### Task 3: Update launcher.c path resolution for installed layout

**Files:**
- Modify: `launcher.c:16-23`

The current launcher navigates up 4 levels from the binary to find `whisper_hotkey.py`. This works when the .app is next to the script. For the installed layout (`/Applications/Whisper Hotkey.app` + `~/.whisper-hotkey/whisper_hotkey.py`), the launcher must also check `~/.whisper-hotkey/`.

- [ ] **Step 1: Add fallback path to launcher**

After the existing `snprintf(script, ...)` line, add a fallback check:

```c
    char script[4096];
    snprintf(script, sizeof(script), "%s/whisper_hotkey.py", dir);

    // Fallback: check ~/.whisper-hotkey/ for installed layout
    if (access(script, R_OK) != 0 && home) {
        snprintf(script, sizeof(script), "%s/.whisper-hotkey/whisper_hotkey.py", home);
    }
```

- [ ] **Step 2: Verify it compiles**

```bash
clang -O2 -framework Foundation -o /tmp/test-launcher launcher.c && echo "OK"
```

Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add launcher.c
git commit -m "feat: add fallback path for installed layout in launcher"
```

---

### Task 4: Test the full installation

- [ ] **Step 1: Run install.sh locally to verify the flow**

```bash
bash install.sh
```

Verify:
- Each step shows colored output
- Homebrew/Python/portaudio checks pass (already installed)
- Venv is created at `~/.whisper-hotkey/venv/`
- Python packages install successfully
- ASR model downloads
- LLM prompt appears and works for both yes/no
- App bundle is created at `/Applications/Whisper Hotkey.app`
- Config is created at `~/.whisper_hotkey/config.json`
- App launches from `/Applications/Whisper Hotkey.app`

- [ ] **Step 2: Verify the app works from installed location**

```bash
open "/Applications/Whisper Hotkey.app"
```

Verify the app opens, shows in the tray, and the hotkey works.

- [ ] **Step 3: Final commit with any fixes**

```bash
git add -A
git commit -m "feat: complete installer for Whisper Hotkey"
```
