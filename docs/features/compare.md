# Compare all 128 cases

The [rendering findings](../parity/findings.md) use **99.9% content similarity per case** for
acceptance. All 128 Native/WASM cases pass; 76 Native/Chrome cases pass. Chrome compatibility
remains incomplete. The separate 843-case supplementary corpus includes 590 JavaScript samples:
551 pass against Chrome.

The commands below write their reports under `artifacts/` on the machine that runs them;
`artifacts/` is not committed.

## Run the comparison

```sh
pnpm install --frozen-lockfile
pnpm exec playwright install chromium
pnpm fixtures
pnpm compare --run my-baseline
```

Capture an additional device profile without changing the fixture files:

```sh
pnpm compare --run dense-baseline --scale 2 --measure-only
pnpm compare --run narrow-baseline --width 320 --height 600 --measure-only
```

Viewport dimensions are CSS pixels; `--scale` selects DPR in `(0, 4]` within the
engine's raster limits. Each profile retains all selected cases and its own
measurements. WASM uses the SDK's region renderer for non-unit scales; scales
below one use the SDK's box-mean downsampling contract.

Open `artifacts/my-baseline/index.html` to inspect Chrome, Rust, difference, and
overlay images. `report.json` contains every result, source/resource hashes,
Chrome version, and element geometry. `--filter <feature>` runs a focused subset;
`--require-exact` additionally makes any non-identical image fail the command.
By default, any case below 99.9% content similarity fails. `--measure-only`
collects measurements without enforcing similarity, but rendering errors still
fail. See [the metric definition](../../conformance/README.md).

Reassess a complete saved native/WASM run without rerendering or changing its
original measurements:

```sh
node tools/assess-similarity.mjs my-baseline native-wasm
node tools/assess-similarity.mjs my-baseline all
```

The first command enforces shared-engine consistency; the second also enforces
Chrome fidelity and currently exits 1. Each creates a separate report directory
and refuses to overwrite it. A passing consistency check does not imply Chrome
compatibility. Add `--no-update-baseline` to the assessment and finalization
commands for supplementary profiles, preserving the canonical machine summaries.
