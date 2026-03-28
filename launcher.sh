#!/bin/bash
# Whisper Hotkey launcher — runs Python directly to preserve accessibility permissions

HOME_DIR="$HOME"
SCRIPT="$HOME_DIR/.whisper-hotkey/whisper_hotkey.py"

# Fallback: check next to .app
if [ ! -f "$SCRIPT" ]; then
    DIR="$(cd "$(dirname "$0")/../../../" && pwd)"
    SCRIPT="$DIR/whisper_hotkey.py"
fi

# Find Python — priority: venv > anaconda > homebrew > system
for PY in \
    "$HOME_DIR/.whisper-hotkey/venv/bin/python3" \
    "$HOME_DIR/anaconda3/bin/python3" \
    "$HOME_DIR/miniconda3/bin/python3" \
    "$HOME_DIR/miniforge3/bin/python3" \
    "/opt/homebrew/bin/python3" \
    "/usr/local/bin/python3" \
    "/usr/bin/python3"; do
    if [ -x "$PY" ]; then
        exec "$PY" "$SCRIPT"
    fi
done

echo "Could not find python3" >&2
exit 1
