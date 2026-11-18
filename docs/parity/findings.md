# Rendering findings

Measured with Chrome for Testing 147.0.7727.15 on macOS ARM64,
Blitz revision `21edc92d90d8f8bedadd3f9ecb14b1ad950952eb` plus the
[documented local patches](../../vendor/README.md), and Vello CPU 0.3.0 through AnyRender 0.18.0.
The original 128 cases use fixed resources and a 640 by 480 viewport at DPR 1.
Supplementary samples add focused HTML/CSS/JavaScript behavior coverage. See
[the methodology and coverage limits](../../conformance/README.md).

## Results

Chrome acceptance is **76/128** for the original cases (39 exact) and **551/843** for the
supplementary corpus (487 exact), which includes **590 JavaScript sources**. Native/WASM passes
**128/128 and 843/843**, with 125 and 835 exact. The supplementary corpus has ten unequal
native/WASM pixels.

Strict Chrome acceptance fails 52 original and 292 supplementary cases. Fonts, glyph coverage,
spacing edges, Canvas overloads and other property conversions remain incomplete.
`conformance/similarity-baseline.json` has the summary of each case;
[`pnpm compare`](../features/compare.md) writes the per-case diagnostics (Chrome, Rust,
difference and overlay images, `report.json`) under `artifacts/<run>/`.

## Remaining compatibility gaps

| Area | Evidence and limitations |
| --- | --- |
| Box model and positioning | All 12 existing cases are pixel-exact |
| Cascade | All eight existing cases are pixel-exact after enabling `:has()` |
| Responsive CSS | The corpus container query is exact; intrinsic-size overrides, style containment, and broader query behavior remain incomplete |
| Flex and grid | Fractional geometry, text, and subgrid differences remain |
| Tables | Captions, definite automatic/percentage column allocation, and tested row stretching are implemented; broader intrinsic constraints, full border conflict resolution, and caption behavior remain incomplete |
| Text | The tested bidi/counter content is present; glyph pixels, upright/mixed-script vertical writing, and broader bidi/counter behavior remain incomplete |
| Effects | Curve coverage and rounding differ; unsupported filters are diagnosed |
| JavaScript | Boa executes DOM scripts, events, timers, supplied modules, and a Canvas 2D subset; Shadow DOM, full form interaction, and many browser APIs remain incomplete |
| Canvas | Images, paths, tangent arcs, gradients, solid text, and image data are implemented; text/curve rasterization, patterns, shadows, filters, and exports remain incomplete |

The pinned Vello CPU backend cannot execute ColorMatrix directly. The adapter
lowers color-only chains to CPU images before replay and still reports
unsupported spatial/color mixed chains, filter graph primitives, and backdrop
filters explicitly. A single blur, offset, flood, or drop shadow retains the
backend path.

## Verification

Validation covers workspace formatting, Clippy with warnings rejected, Rust
behavior/geometry tests, twelve comparison/capture tests, the WASM build, and SDK
contract checks. The manifest passes the SDK checker at levels L1, L2, and L3.
The kit-owned editor and Kition host have embedded E2E coverage. The full
128-case native/Chrome and native/WASM captures are run with the same fixtures
and resources. A successful engine or SDK check does not make the Chrome
conformance gate pass.

## Next implementation order

1. Keep failing fixtures and strict pixel diagnostics. Prioritize table
   column/row layout, shadow geometry, and filter compositing. Extend
   bidi coverage to paragraph boundaries and counter coverage to custom styles
   and reversed CSS counters.
2. Separate text geometry from glyph rasterization. Add CJK and static-font
   coverage, and broaden the current mobile, high-DPR, and designed-page corpus.
3. Keep Native/WASM consistency under the same 99.9% gate while prioritizing
   remaining Chrome failures.
4. Extend the supported subset through focused samples and browser comparisons.
   The headless KHP server, kit-owned editor, and Kition host integration are
   implemented. Keep source persistence, worker isolation, and conflict handling
   covered as engine compatibility expands.

Chrome is a conformance oracle and never runs inside the Rust rendering path.
Complete arbitrary-webpage compatibility, especially JavaScript/browser APIs,
remains a substantially larger project.
