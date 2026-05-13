#!/usr/bin/env bash
# Downloads prebuilt llama.cpp binaries for the current platform and
# places them in src-tauri/binaries/llama-runtime/. CI and local builds
# both call this before `pnpm tauri build`.
#
# Override the pinned release with LLAMA_CPP_VERSION=bXXXX.

set -euo pipefail

VERSION="${LLAMA_CPP_VERSION:-b9131}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/src-tauri/binaries/llama-runtime"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  Darwin-arm64)
    ASSET="llama-${VERSION}-bin-macos-arm64.tar.gz"
    KIND="tar"
    ;;
  Darwin-x86_64)
    ASSET="llama-${VERSION}-bin-macos-x64.tar.gz"
    KIND="tar"
    ;;
  *)
    echo "Unsupported platform for this script: $OS-$ARCH" >&2
    echo "Windows hosts: use scripts/fetch-llama-runtime.ps1" >&2
    exit 1
    ;;
esac

URL="https://github.com/ggml-org/llama.cpp/releases/download/${VERSION}/${ASSET}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Fetching $URL"
curl -fL --retry 3 -o "$TMP/asset" "$URL"

rm -rf "$DEST"
mkdir -p "$DEST"

if [ "$KIND" = "tar" ]; then
  # macOS tarballs wrap everything in a llama-bXXXX/ directory.
  tar -xzf "$TMP/asset" -C "$DEST" --strip-components=1
fi

echo "Installed $(ls "$DEST" | wc -l | tr -d ' ') files to $DEST"
