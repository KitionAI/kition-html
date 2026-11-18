# Rendering conformance

The corpus contains 128 individually named feature fixtures across 19 categories.
The four source modules (`layout.mjs`, `paint.mjs`, `text.mjs`, and `platform.mjs`)
generate the checked-in HTML and `manifest.json`. Each case exercises a distinct
behavior, rather than repeating one page at different colors or sizes.

The fixture baseline is a 640 by 480 CSS-pixel viewport at device scale 1.
`compare.mjs --width <css-pixels> --height <css-pixels> --scale <dpr>` captures
a separate profile without rewriting fixture sources or the manifest.
Reports and Worker comparisons retain those actual dimensions. Use
`--no-update-baseline` when finalizing/assessing supplementary profiles. The dark
media-query case explicitly uses a dark color preference. Inter, Lora, and
JetBrains Mono are copied from Kition's font repository, with their licenses.
System font discovery is disabled in Rust. Chrome loads the same font bytes.
All resources are local; unexpected Chrome network requests fail the case.

Chrome is the oracle only. The Rust executable never launches it. The separate
WASM comparison runs the Rust SDK in a dedicated browser Worker and compares its
pixels to both the native output and Chrome. The host page never inserts the
fixture HTML into its DOM for that comparison.

## What the measurements mean

- `changedPixels`: any unequal RGBA pixel, including a one-level color change.
- `significantPixels`: a pixel whose largest channel difference exceeds 16/255.
  This is a diagnostic measurement, not a passing threshold.
- `foregroundChangedRatio`: unequal foreground pixels divided by the union of
  non-white pixels in both images. White margins cannot inflate this measure.
- `meanAbsoluteError`, `rootMeanSquareError`, `maxChannelDelta`: raw channel errors.
- `differenceBounds`: the smallest rectangle containing every unequal pixel.
- `maxChannelErrorSum`: the sum of each pixel's largest absolute RGBA channel
  error. `similarity = 1 - maxChannelErrorSum / (255 * pixels)` is the canvas
  color similarity; `foregroundSimilarity` uses `foregroundPixels` instead.
  Identical all-white images have foreground similarity 1.
- `chromeGeometry` and `rustGeometry`: element rectangles for diagnosis. The
  upstream Rust geometry API does not consistently include CSS transforms; do
  not treat these rectangle lists as interchangeable DOM implementations.

There are no alignment adjustments or antialias exclusions. Every fixture remains in the denominator,
including JavaScript and browser-API cases. Each Chrome
reference is captured twice and must be stable before comparison.

## Acceptance: 99.9% per case

Every case must have `foregroundSimilarity >= 0.999`. The foreground is the
union of non-white pixels in both images, including missing and added content.
This reduces inflation from empty white margins; it is not semantic background
segmentation. Full-canvas similarity is also reported but does not decide the
gate. This defined color-error metric is not SSIM, a perceptual guarantee, a
fraction of exactly equal pixels, or a measure of functional compatibility.

The gate compares integer error sums (`maxChannelErrorSum * 1000 <=
foregroundPixels * 255`) before formatting percentages. Every case must pass
independently; suite averages cannot hide a failed case. Rendering errors and
missing measurements fail. Unsupported browser-API cases remain in the suite.
Exact changed pixels, significant pixels, geometry, and amplified differences
remain available for diagnosis.

`pnpm compare` enforces this threshold against Chrome. `--measure-only`
collects results without gating similarity. `compare-wasm.mjs` enforces it for
Native/WASM consistency and also reports Chrome scores. Both accept
`--require-exact` for optional bit-identical checking. Rendering errors always
fail.

`node tools/assess-similarity.mjs <run> [all|chrome|native-wasm]` reassesses
existing PNGs from a completed run. It defaults to enforcing all comparisons,
requires at least 100 cases, hashes every input PNG, and writes new reports to
`artifacts/<run>/similarity-999-<target>/`. Existing reports are not overwritten.
The compact latest summary is `conformance/similarity-baseline.json`.

Every run gets a new directory under `artifacts/`. Final runs preserve the exact
HTML, resource bytes, their SHA-256 hashes, the executable hash, Cargo.lock hash,
Chrome version, screenshots, raw metrics, and a filterable HTML gallery. Old
runs are retained, including experiments that exposed renderer failures.

## Coverage limits

This is an initial compatibility corpus, not the Web Platform Tests suite and
not proof of arbitrary-webpage compatibility. The canonical profile uses one operating
system, a 640 by 480 viewport at DPR 1, Latin text, three bundled fonts, and SVG
image resources. Supplementary profiles exercise higher DPR and alternate
viewports using the same 128 features; they do not increase the count of distinct
features. CJK shaping, raster image codecs, broader responsive layouts, a broader range of real
application pages, animations, accessibility, and broader
JavaScript/browser APIs need additional corpora. Four dynamic cases cover DOM mutation, generated content, Canvas 2D, and custom
elements. They do not establish complete JavaScript or framework compatibility.
Engine tests additionally exercise events, timers, modules, resource isolation,
and bounded execution.


## Supplementary compatibility samples

`tools/generate-samples.mjs` expands `samples.mjs` and the `samples-*.mjs` modules
into `samples/manifest.json`. They exercise events, modules, form state,
Canvas 2D, tables, backgrounds, and HTML layout independently of the original
128-case corpus. New failing examples are captured before compatibility fixes.
The original cases are rerun to detect regressions; neither corpus replaces the
other's denominator.

Use a new run directory for every capture:

```sh
node tools/generate-samples.mjs
node tools/compare.mjs --manifest samples/manifest.json --run samples-next --measure-only
node tools/compare-wasm.mjs samples-next
```

A JavaScript fixture uses an explicit state assertion before painting its success
marker. Visible control and layout samples complement those behavior checks.
Capture errors, script diagnostics, unstable references, and pixel differences
remain in reports. A malformed fixture is corrected in its generator, while its
failed historical capture remains unchanged. Native/WASM equality measures the
shared engine; only the separate Chrome comparison measures browser fidelity.

The supplementary corpus also includes twelve designed HTML/CSS/JavaScript pages:
a dashboard, board, invoice, article, pricing, settings, filtered catalog, tabs,
form summary, sorted ledger, Canvas chart, and hero. Their desktop and mobile
captures use the same engine, resources, and strict metric. They test practical
compositions but do not establish arbitrary website or framework compatibility.


## Comparing progress between runs

```sh
node tools/progress.mjs earlier-run later-run
```

This creates immutable `progress-from-earlier-run.json` and `.html` files in
the later run. Only identical fixture hashes, device profiles, and Chrome PNG
bytes qualify for improvement/regression counts. Additions, removals, changed
references, and errors remain separate visible entries. Passing-case gains and
losses use the same integer 99.9% gate as the capture reports. This avoids
claiming improvement merely because the corpus grew or a reference changed.

## Raster-size text probes

The `samples-raster-size.mjs` generator adds 36 independent source fixtures:
DOM and Canvas text, Inter/Lora/JetBrains Mono, 16/32 px, and transforms
0.75/1.25/2. Capture each source at DPR 1, 1.5, and 2 with separate run IDs.
A Canvas bitmap keeps its own resolution even when the page DPR changes.

Two standalone checks distinguish rendering invariants from Chrome similarity:

```sh
node tools/check-raster-size.mjs
node tools/check-raster-size-wasm.mjs
```

The first compares complete Chrome bitmaps for 24 Canvas size/scale pairs and
24 DOM DPR/raster-size pairs. The second checks the corresponding 48 pairs in
the built WASM SDK, including source preservation and diagnostics. Both use
Lora and JetBrains Mono because those fonts have no optical-size axis; Inter
is intentionally retained in the fixture corpus without assuming size-invariant
outlines. Exactly representable origins avoid confusing fractional CSS layout
quantization with raster-size behavior. Neither check substitutes for the full
native/Chrome and native/WASM conformance captures or changes their threshold.

## Canvas kerning probes

`canvas-kerning-cases.json` contains 15 independent state checks, and
`canvas-kerning-metrics.json` records widths for 72 Chrome profiles (three fonts,
8/12/16/20/32/48 px, and four strings), each with auto/normal/none. Run
`node tools/check-canvas-kerning.mjs` to verify these expectations without
rewriting them. The sample generator adds 15 state markers and 12 visible
fill/stroke/maxWidth/transformed-text sources, all with JavaScript enabled.

Native and WASM behavior tests compare every metric within 1/64 CSS px. This
separate numeric test tolerance does not replace the full-image 99.9% Chrome
similarity gate. Inter at intermediate optical sizes still has fractional
advance errors. Optional ligature shaping is preserved.

## Canvas letter and word spacing probes

`canvas-spacing-cases.json` holds 24 state checks, `canvas-spacing-grammar.json`
holds 62 recorded CSS dimension expectations for both properties, and
`canvas-spacing-metrics.json` holds 288 Chrome widths for three supplied fonts,
16 texts, and six spacing configurations. Unicode behavior uses named code-point
arrays. Run `node tools/check-canvas-spacing.mjs` to check the records without
rewriting them. Native and WASM numeric tests use 0.001 CSS px; screenshot
acceptance remains 99.9% with every RGBA pixel retained.

`samples-canvas-spacing.mjs` adds 24 state markers and 18 visible sources spanning
letter spacing, word spacing, negative spacing, stroke, centered maxWidth, and
scaled drawing. Context-sensitive Chrome leading-space behavior remains in the
visible samples and exploratory probes. Font-relative resolution after font
changes and ideographic units remain partial.

## Reentrant Canvas text conversions

`canvas-text-coercion-cases.json` contains 16 independent cases for font,
fontKerning, letterSpacing, and wordSpacing conversion order. Run
`node tools/check-canvas-text-coercion.mjs` to validate expectations in Chrome
without rewriting them. The corresponding HTML samples make the final state
and measured widths visible as a color marker. Native and WASM tests reuse
these independently checked cases; the host E2E also draws text, resets the
bitmap from conversion callbacks, clicks the preview, and saves only source.

## Canvas text method arguments

`canvas-text-arguments-cases.json` contains 29 Chrome-verified cases for method
argument counts, conversion order, scalar values, Symbol/BigInt rejection,
nonfinite inputs, and bitmap resets during conversion. Run
`node tools/check-canvas-text-arguments.mjs` to verify the expectations without
rewriting them. `samples-canvas-text-arguments.mjs` adds those state markers
and six visible fill/stroke samples. The native and WASM checks compare bitmap
results with a freshly reset reference context. The host regression verifies
preview interaction and source-only saves. Default-font differences remain.

## Rectangle and basic transform arguments

`canvas-numeric-cases.json` contains 47 independent Chrome cases for seven
rectangle and basic transform methods. Run `node tools/check-canvas-numeric.mjs`
to verify the expectations without rewriting them. The fixture generator adds
those state markers plus seven visible sources. Cases cover missing arguments,
BigInt coordinates, receiver checks, conversion/exception order, bitmap resets,
script-overridden transform methods, extra arguments, and nested side effects.
Native and WASM checks compare pixels or matrices with a fresh reference context;
the host test also verifies preview interactions and source-only saves.

## Results

The current results are in [the rendering findings](../docs/parity/findings.md).
