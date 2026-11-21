# Developing Kition HTML

How to find your way around the repository, build HTML from source, and run its checks. To use
HTML in your own app, start with the [README](../README.md).

## Repository layout

| Path | What |
|---|---|
| `crates/` | The engine (Rust): `kition-html-engine` (the renderer, with JavaScript in Boa) and `kition-html-kit` (the shared WASM engine behind the `KitDocument` contract) |
| `apps/` | `kition-html`: the native executable (`render`, `preview`, and the headless KHP server) |
| `web/` | The kit's editor on `@kition/sdk/editor` |
| `vendor/` | Modified copies of upstream crates ([vendor/README.md](../vendor/README.md)) |
| `agent/` | The agent guide and the check catalogue |
| `conformance/` | The pixel comparison corpus against Chrome ([conformance/README.md](../conformance/README.md)) |
| `cases/` | The test cases of the feature catalogue (`features.yaml`, beside `kit.json`, the kit manifest) |
| `docs/` | This file, gaps, roadmap, the writing format, the quality reference, attribution; `features/` (native rendering, the comparison, the WASM engine, the editor, JavaScript), `parity/` (parity with Chrome and the rendering findings) |
| `tools/` | The comparison, sample, and WebAssembly build scripts (`package.json` at the root runs them) |

This kit keeps its native executable in `apps/kition-html` and its WASM engine in
`crates/kition-html-kit`, where other kits use `bindings/headless` and `bindings/wasm`.

## Build and check

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm install && pnpm test             # node --test tools/*.test.mjs
cd web && pnpm install && pnpm engine
```

The first three commands are the check every change passes ([quality.md](quality.md)). The
guides in [features/](features/) describe the native renderer, the WebAssembly build, the Chrome
comparison and the editor.

By default `pnpm install` in `web/` installs the published `@kition/sdk` and `@kition/tessera`
from npm, at the versions the committed `web/pnpm-lock.yaml` pins, so the editor needs no other
checkout. Until their first release on the `@kition` scope, `web/package.json` names them as npm
aliases of the rehearsal copies (`npm:@kitionstage/sdk@...`, `npm:@kitionstage/tessera@...`), so
the code already imports the final names. To work against local checkouts instead, run
`KITION_LINK=1 pnpm install` in `web/`: `web/.pnpmfile.cjs` then links `@kition/sdk` from the
kition-sdk checkout next to this repository (`packages/sdk`, or the checkout `KITION_SDK` names)
and `@kition/tessera` from the kition-tessera checkout (`packages/tessera`, or `KITION_TESSERA`),
each one whose package is there. Both packages' exports point at their `dist/`, so build them
first (`pnpm --filter @kition/sdk build` in kition-sdk, `pnpm build` in kition-tessera's
`packages/tessera`). The linked install rewrites `web/pnpm-lock.yaml`; do not commit that lockfile.

`pnpm engine` runs `tools/build-wasm.sh`, which builds the WebAssembly engine
(`crates/kition-html-kit`) into `web/public/engine/` and needs wasm-bindgen-cli 0.2.129 (set
`WASM_BINDGEN` to that executable); [features/wasm-engine.md](features/wasm-engine.md) describes
building it and comparing it with the native renderer. `pnpm dev` in `web/` serves the editor in
the development host (`web/dev.html`).

### Guides

- [Native rendering](features/native-rendering.md): the `kition-html` executable renders a page
  to PNG (`render`) or previews it in a terminal (`preview`), with a bundled Inter font and no file
  or network access beyond the page and its asset directory.
- [Compare all 128 cases](features/compare.md): the Chrome comparison (`pnpm compare`), extra
  device profiles, the saved-run assessment, and the measured results.
- [Shared WASM engine](features/wasm-engine.md): the `KitDocument` engine contract and the
  WebAssembly build (`tools/build-wasm.sh`, `tools/check-sdk.sh`).
- [Editor and Kition integration](features/editor.md): the editor in `web/`, building it into a
  Kition checkout, the headless Kit Harness Protocol server, and the `layer: eval` cases.
- [JavaScript preview](features/javascript.md): what runs in Boa, its resource and execution
  limits, and the supported page, Canvas 2D and form-control APIs.

### Render a page from the command line

The native executable renders a page to a PNG, or previews it in the terminal:

```sh
cargo build --locked -p kition-html
target/debug/kition-html render conformance/cases/grid-fr-tracks.html \
  --assets conformance/assets --width 640 --height 480 --output preview.png
target/debug/kition-html preview conformance/cases/grid-fr-tracks.html \
  --assets conformance/assets --columns 100 --max-rows 40
```

### The headless executable and MCP

Launched with `--kition-kit`, `kition-html` is the kit's headless executable (`headless` in
`kit.json`); it speaks the Kit Process Protocol to a host. kition-sdk's `kition-kits-mcp` is such a
host: it serves the kit's tools (`html_create`, `html_inspect`, `html_commands`, `html_run`,
`html_check`) to any MCP client over stdio.

```sh
cargo build --release -p kition-html                   # target/release/kition-html
# in a kition-sdk checkout:
cargo build -p kition-kits --bin kition-kits-mcp
target/debug/kition-kits-mcp --workspace <folder> --kit <kition-html>/target/release/kition-html
```

[agent/guide.md](../agent/guide.md) explains the tools; [writing-format.md](writing-format.md)
lists the commands.

## Rendering measurements

The renderer uses Blitz and Vello CPU. It needs no browser executable, browser window, GPU,
network access, or operating-system fonts at runtime; Chrome is installed only for conformance
testing. JavaScript runs in Boa against the same DOM on native and WASM. Browser APIs and Chrome
pixel fidelity remain incomplete.

The [rendering findings](parity/findings.md) use **99.9%
content similarity per case** for acceptance. All 128 Native/WASM cases pass; 76 Native/Chrome
cases pass. The separate 843-case supplementary corpus includes 590 JavaScript samples: 551 pass
against Chrome. The engine contract is 2: the page is a KDM replica, schema `kit.html` v1 in
[model/schema.json](../crates/kition-html-kit/model/schema.json).

## Take the README screenshots

The pictures under `assets/readme/` are taken by `web/scripts/readme-shots.mjs`. It follows the
rules of kition-sdk's shared runner (`tools/readme-shots`) but does not use it: that runner takes
Playwright from `web/node_modules`, and this repository installs `@playwright/test` at its root.
The script opens the showcase sample `bakery-newsletter-email` from a
[kition-showcase](https://github.com/KitionAI/kition-showcase) checkout next to this repository
(or the one `KITION_SHOWCASE` names) in the development host, in Classic display mode at
1440 × 900 and device scale 2, and changes it only through the editor (see-2 plays an agent's edit
through the host's `ai/run`, as Kition's agent edits an open file). With the engine built in
`web/public/engine/`, the root's and `web/`'s dependencies installed, and no HTTP proxy set:

```sh
node web/scripts/readme-shots.mjs              # hero, see-1 … see-4
node web/scripts/readme-shots.mjs see-3        # only the named ones
```

`CHROME` names a Chrome or Chromium executable to use instead of Playwright's. When `pngquant` is
installed the script shrinks each picture with `--quality 70-90` (a README picture is at most
400 KB). `made-1.png` … `made-4.png` are copies of showcase samples' `thumb.png`.

## Publish

Kition ships the kit's package, `kition-html-<version>.tgz` (the `html-ui/` folder), a release
asset of this repository tagged `package-v<version>`. It is built on GitHub, never on a developer
machine: the Package workflow (`.github/workflows/package.yml`, which calls Kition's reusable
`kit-package.yml`) builds the commit it is started on, with `gh workflow run package.yml -R
KitionAI/kition-html`, and publishes the Release with the package's lock entry,
`kition-html-<version>.json`. Kition pins it with `node scripts/pin-kits.mjs html` (see
`docs/kits.md` there). A version is published once: bump `version` in `kit.json` to publish a
change.
