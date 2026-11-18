# Shared WASM engine

The SDK uses `kition_kit::KitDocument` and `export_kit_document!`. The engine
contract includes source replacement, undo/redo, exact box-mean downsampling,
PNG snapshots, diagnostics, and saving that stays dirty until `markSaved()`.
Fonts and other resource bytes are supplied explicitly through `setResource`.
The browser binding belongs in a Worker.

```sh
rustup toolchain install 1.95.0
rustup target add wasm32-unknown-unknown --toolchain 1.95.0
# Install wasm-bindgen-cli 0.2.129 to a tools directory, or use its release binary.
cargo install wasm-bindgen-cli --version 0.2.129 --locked --root target/tools
WASM_BINDGEN="$PWD/target/tools/bin/wasm-bindgen" bash tools/build-wasm.sh
bash tools/check-sdk.sh
node tools/compare-wasm.mjs my-baseline
node tools/finalize-report.mjs my-baseline
```

`tools/check-sdk.sh` uses the adjacent `kition-sdk` checkout; override
`KITION_SDK_DIR` when necessary. Rust 1.95 is required by that SDK. The native
executable also requires Rust 1.95 for its headless KHP server. If Cargo is configured to use a
different compiler, set `RUSTC` to the result of `rustup which --toolchain 1.95.0
rustc` when testing the SDK; the WASM build script does this automatically.
