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
