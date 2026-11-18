#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
TOOLCHAIN="${KITION_RUST_TOOLCHAIN:-1.95.0}"
export RUSTC="$(rustup which --toolchain "$TOOLCHAIN" rustc)"
export RUSTDOC="$(rustup which --toolchain "$TOOLCHAIN" rustdoc)"
# A release engine: the debug build was 70 MB and rendered a 2 MB page about twelve times slower.
rustup run "$TOOLCHAIN" cargo build --locked --release -p kition-html-kit \
  --target wasm32-unknown-unknown --target-dir target/wasm-build
BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
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
