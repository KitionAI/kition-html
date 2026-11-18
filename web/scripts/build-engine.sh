#!/usr/bin/env bash
set -euo pipefail
kit_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
bash "$kit_root/tools/build-wasm.sh"
mkdir -p "$kit_root/web/public/engine" "$kit_root/web/public/fonts"
cp -R "$kit_root/engine/." "$kit_root/web/public/engine/"
cp "$kit_root/conformance/assets/Inter-Variable.ttf" "$kit_root/web/public/fonts/"
cp "$kit_root/conformance/assets/Inter-OFL.txt" "$kit_root/web/public/fonts/"
