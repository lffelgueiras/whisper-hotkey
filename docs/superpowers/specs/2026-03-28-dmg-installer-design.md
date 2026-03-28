# Whisper Hotkey .dmg Installer — Design Spec

## Goal

Create a `.dmg` containing a `.pkg` installer that provides a wizard-style installation experience. The `.pkg` copies application files and then opens Terminal to run dependency installation (Homebrew, Python venv, pip packages, ML models) with visible progress.

## Architecture

The build pipeline uses native macOS tools (`pkgbuild`, `productbuild`, `hdiutil`) to produce the final `WhisperHotkey.dmg`. No third-party build tools required.

### Build flow

```
build-dmg.sh
  ├── pkgbuild  → component .pkg (payload + postinstall script)
  ├── productbuild + distribution.xml → WhisperHotkey.pkg (wizard UI)
  └── hdiutil   → WhisperHotkey.dmg
```

### .pkg structure

The `.pkg` installs to two locations:

1. **`/Applications/Whisper Hotkey.app`** — the .app bundle (launcher.sh, Info.plist, icon)
2. **`~/.whisper-hotkey/`** — application files (whisper_hotkey.py, icons/, install.sh)

Since `.pkg` installs as root and `~` is ambiguous, the postinstall script handles copying files to the current user's home directory.

### Postinstall flow

1. The `.pkg` extracts payload to a temporary staging area
2. The `postinstall` script:
   - Copies `whisper_hotkey.py`, `icons/`, `install.sh` to `$HOME/.whisper-hotkey/`
   - Builds the `.app` bundle in `/Applications/Whisper Hotkey.app`
   - Opens Terminal.app running `~/.whisper-hotkey/install.sh --from-pkg`
3. `install.sh --from-pkg`:
   - Skips file copying (already done by pkg)
   - Skips .app bundle creation (already done by pkg)
   - Skips the interactive LLM prompt — always installs the LLM model
   - Installs: Homebrew, Python 3, portaudio, venv, pip packages, ASR model, LLM model
   - Creates default config at `~/.whisper_hotkey/config.json`

## Files

| File | Action | Purpose |
|------|--------|---------|
| `build-dmg.sh` | Create | Build script that produces the .dmg |
| `distribution.xml` | Create | Wizard UI config (title, intro text, no license) |
| `scripts/postinstall` | Create | Pkg postinstall — copies files to $HOME, opens Terminal |
| `install.sh` | Modify | Add `--from-pkg` flag to skip file copy/app build/interactive prompt |

## install.sh changes

Add a `--from-pkg` flag that:
- Sets `FROM_PKG=true`
- Skips steps 10 (copy files) and 11 (build app bundle) — already done by pkg
- Skips the interactive `read -p` prompt — always installs the LLM model
- All other steps run normally (Homebrew, Python, venv, packages, models, config)

## build-dmg.sh

The build script:
1. Creates a staging directory with the payload:
   - `whisper_hotkey.py`
   - `icons/`
   - `install.sh`
   - `launcher.sh`
2. Creates the `.app` bundle structure (same as current install.sh step 11)
3. Runs `pkgbuild` with the staging payload and postinstall script
4. Runs `productbuild` with `distribution.xml` to wrap it in a wizard
5. Creates `.dmg` with `hdiutil create`
6. Cleans up temp files

## distribution.xml

Minimal wizard configuration:
- Title: "Whisper Hotkey"
- Welcome text: "Este instalador vai configurar o Whisper Hotkey no seu Mac. Apos a instalacao, uma janela do Terminal vai abrir para baixar os modelos de IA necessarios (~3.7GB). Isso pode levar alguns minutos dependendo da sua conexao."
- No license screen
- No destination selection (fixed to current user)

## scripts/postinstall

```bash
#!/bin/bash
REAL_HOME=$(eval echo ~$USER)
INSTALL_DIR="$REAL_HOME/.whisper-hotkey"

# Copy payload files to user's home
mkdir -p "$INSTALL_DIR"
cp "$1/whisper_hotkey.py" "$INSTALL_DIR/"
cp -r "$1/icons" "$INSTALL_DIR/"
cp "$1/install.sh" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/install.sh"

# Build .app bundle
APP_DIR="/Applications/Whisper Hotkey.app"
APP_CONTENTS="$APP_DIR/Contents"
mkdir -p "$APP_CONTENTS/MacOS" "$APP_CONTENTS/Resources"
cp "$1/launcher.sh" "$APP_CONTENTS/MacOS/Whisper Hotkey"
chmod +x "$APP_CONTENTS/MacOS/Whisper Hotkey"
# Info.plist and icon generation handled here

# Open Terminal to complete installation
open -a Terminal "$INSTALL_DIR/install.sh" --args --from-pkg
```

## User Experience

1. User receives `WhisperHotkey.dmg`
2. Mounts the DMG, double-clicks `WhisperHotkey.pkg`
3. Wizard: Introducao -> Instalacao -> Conclusao
4. After wizard finishes, Terminal opens automatically
5. Terminal shows colored progress of each installation step
6. After completion, user opens app via Spotlight or Applications
7. First launch prompts for Accessibility and Microphone permissions

## Constraints

- Apple Silicon only (arm64 check in install.sh)
- macOS 12.0+ (Monterey)
- Requires internet for Homebrew, pip packages, and model downloads
- The `.pkg` is unsigned (Gatekeeper may warn; user right-clicks -> Open)
- Total download during install: ~3.7GB (ASR model ~1.2GB + LLM model ~2.5GB)
