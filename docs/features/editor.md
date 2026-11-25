# Editor and Kition integration

The self-contained editor in `web/` uses `@kition/sdk/editor` and renders
HTML/CSS/JavaScript in a Web Worker. It edits source, previews the engine's raster, reports
rendering diagnostics, and shares undo/redo, save, read-only handling, and
on-disk conflict detection with other kits. HTML source is UTF-8, limited to
8 MiB (a larger page is refused as `too-large`), and may be empty; nonempty source must begin with markup. Binary
control characters are rejected.
JavaScript executes inside the engine Worker. External resources are not fetched automatically;
unavailable resources appear in diagnostics.

The editor installs the published `@kition/*` packages from npm;
[Build and check](../development.md#build-and-check) in the development guide describes the install
and the `KITION_LINK=1` opt-in for a kition-sdk checkout.

From an adjacent Kition checkout, build and open it locally:

```sh
pnpm --dir ../kition-html/web install --frozen-lockfile
bash scripts/build-kit-ui.sh html
make kit-dev
```

Kition's New menu includes HTML when this package is built. It creates
`Untitled HTML.html` with the Worker engine and writes through
`PUT /v1/kits/files`; `.html` and `.htm` open in the kit's own editor in
`KitFrame`. The native `kition-html` also serves the headless Kit Harness Protocol
when launched with `--kition-kit`, preserving its `render` and `preview` CLI
commands. Agent tools use the same `file.setSource`, undo/redo, and diagnostics.

Kition's `kits/kits.lock.json` pins the published package, and Kition's `kits/modules.json` lists
the kit as optional. Local builds are explicitly marked
`uncommitted` until this repository has a commit. KDM collaboration, object-model
editing, agent drafts, and an Agent-ready tier are not claimed. The manifest
checker verifies structural platform levels; headless conformance and engine
contract tests separately verify behavior. Rendering compatibility remains
experimental, with the measured supported subset documented in [compare.md](compare.md) and
[findings.md](../parity/findings.md).

## Eval cases

`cases/html.agent.*` are `layer: eval` cases (docs/quality.md): a real model edits a page through the
kits' MCP server, judged from the file it leaves. This repository has no xtask, so they run through
kition-sdk's runner from the checkout beside this one; only a maintainer starts a real run (F-QA-16),
never a hook, CI or an agent on its own:

```sh
node ../kition-sdk/tools/eval-cases.mjs --repo . --list
node ../kition-sdk/tools/eval-cases.mjs --repo . --dry-run   # no model: the cases and their solutions
```

The inputs are copies of `conformance/basic.html` and `conformance/check-resource-unavailable.html`.

Compatibility work must keep both comparisons separate: Native versus WASM
checks shared-engine consistency; Rust versus Chrome measures web fidelity.
Passing the former never implies the latter. Unsupported features must remain
visible in diagnostics and in the corpus.

Dependencies: [Blitz](https://github.com/DioxusLabs/blitz),
[AnyRender](https://github.com/linebender/anyrender), and
[Kition SDK](https://github.com/KitionAI/kition-sdk).
