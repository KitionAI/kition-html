#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
TOOLCHAIN="${KITION_RUST_TOOLCHAIN:-1.95.0}"
# The pinned toolchain needs its own wasm32 target; a runner that only adds it to stable has none.
rustup target add --toolchain "$TOOLCHAIN" wasm32-unknown-unknown
export RUSTC="$(rustup which --toolchain "$TOOLCHAIN" rustc)"
export RUSTDOC="$(rustup which --toolchain "$TOOLCHAIN" rustdoc)"
# A release engine: the debug build was 70 MB and rendered a 2 MB page about twelve times slower.
rustup run "$TOOLCHAIN" cargo build --locked --release -p kition-html-kit \
  --target wasm32-unknown-unknown --target-dir target/wasm-build
BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
# Where no wasm-bindgen is installed (a GitHub runner), fetch the released 0.2.129 executable.
if ! command -v "$BINDGEN" >/dev/null 2>&1; then
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) TRIPLE=x86_64-unknown-linux-musl ;;
    Linux-aarch64) TRIPLE=aarch64-unknown-linux-musl ;;
    Darwin-arm64) TRIPLE=aarch64-apple-darwin ;;
    Darwin-x86_64) TRIPLE=x86_64-apple-darwin ;;
    *) echo 'wasm-bindgen 0.2.129 is not installed. Set WASM_BINDGEN to that executable.' >&2; exit 1 ;;
  esac
  TOOLS="target/tools/wasm-bindgen-0.2.129"
  mkdir -p "$TOOLS"
  curl -fsSL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.129/wasm-bindgen-0.2.129-$TRIPLE.tar.gz" \
    | tar -xz --strip-components=1 -C "$TOOLS"
  BINDGEN="$TOOLS/wasm-bindgen"
fi
if [[ "$("$BINDGEN" --version)" != 'wasm-bindgen 0.2.129' ]]; then
  echo 'Cargo.lock requires wasm-bindgen-cli 0.2.129. Set WASM_BINDGEN to that executable.' >&2
  exit 1
fi
"$BINDGEN" --target web --out-dir engine \
  target/wasm-build/wasm32-unknown-unknown/release/kition_html_kit.wasm

# The default font is embedded in the WASM module; retain its redistribution terms.
cp conformance/assets/Inter-OFL.txt engine/Inter-OFL.txt
# Ship the terms for vendored sources and adapted rendering arithmetic.
mkdir -p engine/licenses
for component in blitz-html blitz-vibey-script blitz-dom blitz-paint glifo parley stylo vello_cpu boa_parser; do
  for license in "vendor/$component"/LICENSE*; do
    if [[ -f "$license" ]]; then
      cp "$license" "engine/licenses/$component-$(basename "$license")"
    fi
  done
done
