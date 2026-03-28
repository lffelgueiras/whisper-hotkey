#!/bin/bash
# Whisper Hotkey launcher
# Copies the venv python3 binary into the .app bundle so macOS grants
# accessibility/microphone permissions to "Whisper Hotkey" instead of "python3".

APP_MACOS="$(cd "$(dirname "$0")" && pwd)"
EMBEDDED_PY="$APP_MACOS/python3"
SCRIPT="$HOME/.whisper-hotkey/whisper_hotkey.py"

# Fallback: check next to .app
if [ ! -f "$SCRIPT" ]; then
    DIR="$(cd "$APP_MACOS/../../../" && pwd)"
    SCRIPT="$DIR/whisper_hotkey.py"
fi

# Find source Python — priority: venv > homebrew > system
SOURCE_PY=""
for PY in \
    "$HOME/.whisper-hotkey/venv/bin/python3" \
    "$HOME/anaconda3/bin/python3" \
    "$HOME/miniconda3/bin/python3" \
    "$HOME/miniforge3/bin/python3" \
    "/opt/homebrew/bin/python3" \
    "/usr/local/bin/python3" \
    "/usr/bin/python3"; do
    if [ -x "$PY" ]; then
        SOURCE_PY="$PY"
        break
    fi
done

if [ -z "$SOURCE_PY" ]; then
    echo "Could not find python3" >&2
    exit 1
fi

# Resolve the real python3 binary (follow symlinks)
REAL_PY="$("$SOURCE_PY" -c "import os,sys; print(os.path.realpath(sys.executable))")"

# Copy python3 binary into .app bundle (if missing or outdated)
# This makes macOS see the process as "Whisper Hotkey" for permissions
if [ ! -f "$EMBEDDED_PY" ] || [ "$REAL_PY" -nt "$EMBEDDED_PY" ]; then
    cp "$REAL_PY" "$EMBEDDED_PY"
    chmod +x "$EMBEDDED_PY"
fi

# Point the embedded python3 to the venv's site-packages
VENV_DIR="$HOME/.whisper-hotkey/venv"
if [ -d "$VENV_DIR" ]; then
    export VIRTUAL_ENV="$VENV_DIR"
    export PATH="$VENV_DIR/bin:$PATH"
    # Ensure the venv site-packages are found
    SITE_PACKAGES="$VENV_DIR/lib/python3.$("$SOURCE_PY" -c 'import sys; print(sys.version_info.minor)')/site-packages"
    if [ -d "$SITE_PACKAGES" ]; then
        export PYTHONPATH="$SITE_PACKAGES${PYTHONPATH:+:$PYTHONPATH}"
    fi
fi

# Use exec so the python3 copy IS the app process (same PID)
exec "$EMBEDDED_PY" "$SCRIPT"
