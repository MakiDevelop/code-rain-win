#!/usr/bin/env bash
# Cross-compile Code Rain screensaver from macOS to Windows .scr
#
# Prerequisites (one-time):
#   brew install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#
# Output: dist/coderain.scr
#
# Install on Windows 11:
#   1. Copy coderain.scr to C:\Windows\System32\
#   2. Settings → Personalization → Lock screen → Screen saver
#   3. Pick "coderain" from the dropdown, set wait time, OK.

set -euo pipefail

cd "$(dirname "$0")"

if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "ERROR: x86_64-w64-mingw32-gcc not found. Run: brew install mingw-w64" >&2
    exit 1
fi

cargo build --release --target x86_64-pc-windows-gnu

mkdir -p dist
cp target/x86_64-pc-windows-gnu/release/coderain.exe dist/coderain.scr

echo
echo "Built: $(pwd)/dist/coderain.scr"
ls -lh dist/coderain.scr
