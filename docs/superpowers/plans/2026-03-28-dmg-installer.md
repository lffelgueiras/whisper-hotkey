# .dmg Installer — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a `.dmg` containing a `.pkg` installer with wizard UI that installs Whisper Hotkey and opens Terminal to download dependencies and ML models.

**Architecture:** `build-dmg.sh` uses `pkgbuild` + `productbuild` + `hdiutil` to produce `WhisperHotkey.dmg`. The `.pkg` payload is extracted by a `postinstall` script that copies files to `~/.whisper-hotkey/`, builds the `.app` bundle, and opens Terminal running `install.sh --from-pkg`.

**Tech Stack:** bash, pkgbuild, productbuild, hdiutil, sips, iconutil, codesign

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `install.sh` | Modify | Add `--from-pkg` flag to skip file copy, app build, and interactive prompt |
| `pkg/postinstall` | Create | Pkg postinstall script — copies payload to `$HOME`, builds .app, opens Terminal |
| `pkg/distribution.xml` | Create | Wizard UI configuration (title, welcome text) |
| `build-dmg.sh` | Create | Build script that produces `WhisperHotkey.dmg` |

---

### Task 1: Add `--from-pkg` flag to install.sh

**Files:**
- Modify: `install.sh:1-249`

- [ ] **Step 1: Add flag parsing and banner at the top of install.sh**

After `NC='\033[0m'` (line 10), before the `INSTALL_DIR` line (line 12), add:

```bash
# ── Parse flags ──────────────────────────────────────────────────────────────
FROM_PKG=false
for arg in "$@"; do
    case "$arg" in
        --from-pkg) FROM_PKG=true ;;
    esac
done
```

- [ ] **Step 2: Wrap steps 10 and 11 with FROM_PKG guard**

Find step 10 (line 126, `info "Copiando arquivos da aplicacao..."`). Wrap steps 10 and 11 in a conditional:

Before step 10, add:
```bash
if [ "$FROM_PKG" = false ]; then
```

After step 11's last line (line 200, `success "Aplicacao construida em $APP_DIR"`), add:
```bash
fi
```

This skips file copying and app bundle building when called from the pkg installer.

- [ ] **Step 3: Replace interactive LLM prompt with FROM_PKG auto-yes**

Replace the current step 9 block (lines 108-123) with:

```bash
if [ "$FROM_PKG" = true ]; then
    INSTALL_LLM=true
else
    echo ""
    echo -e "${BOLD}Deseja instalar o modelo de pos-processamento de texto (~2.5GB)?${NC}"
    echo "Esse modelo corrige pontuacao, acentuacao e formatacao das transcricoes."
    echo "Recomendado para Macs com 16GB+ de RAM."
    echo ""
    read -p "Instalar modelo de pos-processamento? [s/N] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Ss]$ ]]; then
        INSTALL_LLM=true
    fi
fi
```

- [ ] **Step 4: Verify syntax**

```bash
bash -n install.sh && echo "Syntax OK"
```

Expected: `Syntax OK`

- [ ] **Step 5: Commit**

```bash
git add install.sh
git commit -m "feat: add --from-pkg flag to install.sh for pkg installer support"
```

---

### Task 2: Create pkg/postinstall script

**Files:**
- Create: `pkg/postinstall`

- [ ] **Step 1: Create the postinstall script**

```bash
#!/bin/bash
# Whisper Hotkey .pkg postinstall script
# Runs as root after pkg extraction. Copies files to user home and builds .app.

set -euo pipefail

# Get the real user (not root) — installer sets USER and HOME
REAL_USER="${USER}"
REAL_HOME="${HOME}"
# Fallback: if running as root, resolve from SUDO_USER or console user
if [ "$REAL_USER" = "root" ] || [ -z "$REAL_HOME" ] || [ "$REAL_HOME" = "/var/root" ]; then
    REAL_USER=$(stat -f "%Su" /dev/console)
    REAL_HOME=$(eval echo "~$REAL_USER")
fi

INSTALL_DIR="$REAL_HOME/.whisper-hotkey"
PKG_PAYLOAD="$1"  # First arg is the package payload directory

# ── Copy application files to ~/.whisper-hotkey/ ─────────────────────────────
mkdir -p "$INSTALL_DIR"
cp "$PKG_PAYLOAD/whisper_hotkey.py" "$INSTALL_DIR/"
cp -r "$PKG_PAYLOAD/icons" "$INSTALL_DIR/"
cp "$PKG_PAYLOAD/install.sh" "$INSTALL_DIR/"
cp "$PKG_PAYLOAD/launcher.sh" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/install.sh"
chmod +x "$INSTALL_DIR/launcher.sh"

# Fix ownership (postinstall runs as root)
chown -R "$REAL_USER" "$INSTALL_DIR"

# ── Build .app bundle in /Applications ───────────────────────────────────────
APP_DIR="/Applications/Whisper Hotkey.app"
APP_CONTENTS="$APP_DIR/Contents"
mkdir -p "$APP_CONTENTS/MacOS"
mkdir -p "$APP_CONTENTS/Resources"

# Copy launcher
cp "$INSTALL_DIR/launcher.sh" "$APP_CONTENTS/MacOS/Whisper Hotkey"
chmod +x "$APP_CONTENTS/MacOS/Whisper Hotkey"

# Write Info.plist
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
if [ -f "$INSTALL_DIR/icons/whisper.png" ]; then
    ICONSET_DIR="/tmp/whisper-hotkey.iconset"
    mkdir -p "$ICONSET_DIR"
    sips -z 16 16     "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_16x16.png"    &>/dev/null
    sips -z 32 32     "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_16x16@2x.png" &>/dev/null
    sips -z 32 32     "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_32x32.png"    &>/dev/null
    sips -z 64 64     "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_32x32@2x.png" &>/dev/null
    sips -z 128 128   "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_128x128.png"  &>/dev/null
    sips -z 256 256   "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_128x128@2x.png" &>/dev/null
    sips -z 256 256   "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_256x256.png"  &>/dev/null
    sips -z 512 512   "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_256x256@2x.png" &>/dev/null
    sips -z 512 512   "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_512x512.png"  &>/dev/null
    sips -z 1024 1024 "$INSTALL_DIR/icons/whisper.png" --out "$ICONSET_DIR/icon_512x512@2x.png" &>/dev/null
    iconutil -c icns "$ICONSET_DIR" -o "$APP_CONTENTS/Resources/icon.icns"
    rm -rf "$ICONSET_DIR"
fi

# Code sign (ad-hoc)
codesign --force --deep --sign - "$APP_DIR" 2>/dev/null || true

# ── Open Terminal to complete dependency installation ────────────────────────
# Use osascript to open Terminal as the real user (not root)
su "$REAL_USER" -c "open -a Terminal \"$INSTALL_DIR/install.sh\" --args --from-pkg"

exit 0
```

- [ ] **Step 2: Make it executable**

```bash
chmod +x pkg/postinstall
```

- [ ] **Step 3: Verify syntax**

```bash
bash -n pkg/postinstall && echo "Syntax OK"
```

Expected: `Syntax OK`

- [ ] **Step 4: Commit**

```bash
git add pkg/postinstall
git commit -m "feat: add pkg postinstall script for .dmg installer"
```

---

### Task 3: Create pkg/distribution.xml

**Files:**
- Create: `pkg/distribution.xml`

- [ ] **Step 1: Write distribution.xml**

```xml
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>Whisper Hotkey</title>
    <welcome language="pt" mime-type="text/html"><![CDATA[
        <html>
        <body style="font-family: -apple-system, Helvetica, Arial, sans-serif; font-size: 14px; line-height: 1.6;">
        <h2>Bem-vindo ao Whisper Hotkey</h2>
        <p>Este instalador vai configurar o <strong>Whisper Hotkey</strong> no seu Mac.</p>
        <p>Apos a instalacao, uma janela do <strong>Terminal</strong> vai abrir automaticamente para baixar os modelos de IA necessarios (~3.7GB). Isso pode levar alguns minutos dependendo da sua conexao.</p>
        <p><strong>Requisitos:</strong></p>
        <ul>
            <li>Apple Silicon (M1/M2/M3/M4)</li>
            <li>macOS 12.0 ou superior</li>
            <li>Conexao com a internet</li>
            <li>~5GB de espaco livre</li>
        </ul>
        </body>
        </html>
    ]]></welcome>
    <welcome mime-type="text/html"><![CDATA[
        <html>
        <body style="font-family: -apple-system, Helvetica, Arial, sans-serif; font-size: 14px; line-height: 1.6;">
        <h2>Welcome to Whisper Hotkey</h2>
        <p>This installer will set up <strong>Whisper Hotkey</strong> on your Mac.</p>
        <p>After installation, a <strong>Terminal</strong> window will open automatically to download the required AI models (~3.7GB). This may take a few minutes depending on your connection.</p>
        <p><strong>Requirements:</strong></p>
        <ul>
            <li>Apple Silicon (M1/M2/M3/M4)</li>
            <li>macOS 12.0 or later</li>
            <li>Internet connection</li>
            <li>~5GB free disk space</li>
        </ul>
        </body>
        </html>
    ]]></welcome>
    <options customize="never" require-scripts="false" hostArchitectures="arm64"/>
    <domains enable_anywhere="false" enable_currentUserHome="true" enable_localSystem="false"/>
    <pkg-ref id="com.whisperhotkey.app"/>
    <choices-outline>
        <line choice="default">
            <line choice="com.whisperhotkey.app"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="com.whisperhotkey.app" visible="false">
        <pkg-ref id="com.whisperhotkey.app"/>
    </choice>
    <pkg-ref id="com.whisperhotkey.app" version="1.0" onConclusion="none">#WhisperHotkey-component.pkg</pkg-ref>
</installer-gui-script>
```

- [ ] **Step 2: Verify XML is well-formed**

```bash
xmllint --noout pkg/distribution.xml && echo "XML OK"
```

Expected: `XML OK`

- [ ] **Step 3: Commit**

```bash
git add pkg/distribution.xml
git commit -m "feat: add distribution.xml for pkg wizard UI"
```

---

### Task 4: Create build-dmg.sh

**Files:**
- Create: `build-dmg.sh`

- [ ] **Step 1: Write the build script**

```bash
#!/bin/bash
set -euo pipefail

# ── Colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${GREEN}==>${NC} ${BOLD}$1${NC}"; }
fail()    { echo -e "${RED}✗ $1${NC}"; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="/tmp/whisper-hotkey-build"
PKG_ID="com.whisperhotkey.app"
VERSION="1.0"
OUTPUT_DIR="$SCRIPT_DIR/dist"

echo ""
echo -e "${BOLD}Building Whisper Hotkey .dmg${NC}"
echo ""

# ── Clean previous build ────────────────────────────────────────────────────
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/payload"
mkdir -p "$BUILD_DIR/scripts"
mkdir -p "$OUTPUT_DIR"

# ── Stage payload files ─────────────────────────────────────────────────────
info "Staging payload files..."
cp "$SCRIPT_DIR/whisper_hotkey.py" "$BUILD_DIR/payload/"
cp -r "$SCRIPT_DIR/icons" "$BUILD_DIR/payload/"
cp "$SCRIPT_DIR/install.sh" "$BUILD_DIR/payload/"
cp "$SCRIPT_DIR/launcher.sh" "$BUILD_DIR/payload/"

# ── Stage postinstall script ────────────────────────────────────────────────
info "Staging postinstall script..."
cp "$SCRIPT_DIR/pkg/postinstall" "$BUILD_DIR/scripts/postinstall"
chmod +x "$BUILD_DIR/scripts/postinstall"

# ── Build component .pkg ────────────────────────────────────────────────────
info "Building component package..."
pkgbuild \
    --identifier "$PKG_ID" \
    --version "$VERSION" \
    --root "$BUILD_DIR/payload" \
    --install-location "/tmp/whisper-hotkey-payload" \
    --scripts "$BUILD_DIR/scripts" \
    "$BUILD_DIR/WhisperHotkey-component.pkg"

# ── Build product .pkg with distribution ────────────────────────────────────
info "Building product package with wizard..."
productbuild \
    --distribution "$SCRIPT_DIR/pkg/distribution.xml" \
    --package-path "$BUILD_DIR" \
    "$BUILD_DIR/WhisperHotkey.pkg"

# ── Create .dmg ─────────────────────────────────────────────────────────────
info "Creating DMG..."
DMG_STAGING="$BUILD_DIR/dmg-staging"
mkdir -p "$DMG_STAGING"
cp "$BUILD_DIR/WhisperHotkey.pkg" "$DMG_STAGING/"

# Remove old DMG if it exists
rm -f "$OUTPUT_DIR/WhisperHotkey.dmg"

hdiutil create \
    -volname "Whisper Hotkey" \
    -srcfolder "$DMG_STAGING" \
    -ov \
    -format UDZO \
    "$OUTPUT_DIR/WhisperHotkey.dmg"

# ── Clean up ────────────────────────────────────────────────────────────────
rm -rf "$BUILD_DIR"

echo ""
echo -e "${GREEN}${BOLD}DMG created: $OUTPUT_DIR/WhisperHotkey.dmg${NC}"
echo ""
```

- [ ] **Step 2: Make it executable**

```bash
chmod +x build-dmg.sh
```

- [ ] **Step 3: Verify syntax**

```bash
bash -n build-dmg.sh && echo "Syntax OK"
```

Expected: `Syntax OK`

- [ ] **Step 4: Commit**

```bash
git add build-dmg.sh
git commit -m "feat: add build-dmg.sh to produce .dmg installer"
```

---

### Task 5: Build and test the .dmg

- [ ] **Step 1: Run the build**

```bash
cd /Users/lffelgueiras/Dev/whisper-hotkey && bash build-dmg.sh
```

Expected: `DMG created: .../dist/WhisperHotkey.dmg`

- [ ] **Step 2: Verify DMG contents**

```bash
hdiutil attach dist/WhisperHotkey.dmg && ls "/Volumes/Whisper Hotkey/" && hdiutil detach "/Volumes/Whisper Hotkey"
```

Expected: `WhisperHotkey.pkg` listed inside the volume

- [ ] **Step 3: Add dist/ to .gitignore**

Append to `.gitignore` (create if not exists):

```
dist/
```

- [ ] **Step 4: Commit**

```bash
git add build-dmg.sh pkg/ .gitignore
git commit -m "feat: complete .dmg installer build pipeline"
```
