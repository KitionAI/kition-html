# Vendored rendering changes

`blitz-dom` and `blitz-paint` contain runtime sources from
[Blitz revision 21edc92d90d8f8bedadd3f9ecb14b1ad950952eb](https://github.com/DioxusLabs/blitz/tree/21edc92d90d8f8bedadd3f9ecb14b1ad950952eb/packages).
Their original MIT and Apache licenses are retained. `glifo` contains a separate
glyph expansion fix documented below; `parley` contains logical-pixel metric
quantization. `stylo` enables the existing container-query parser in Servo builds.
`vello_cpu` contains low-precision layer compositing and glyph coverage patches described below.
`boa_parser` contains the EOF handling fix for named module exports described below.
Other Blitz packages and all transitive versions stay pinned through Cargo.lock.

The local manifests expand inherited workspace values, pin sibling packages to
that revision, and declare independent workspaces. The root Cargo patch ensures
that every dependency uses the same local DOM implementation on native and WASM.
No Cargo cache files are modified.

Modified upstream files:

| File | Change |
| --- | --- |
| `blitz-dom/assets/default.css` | Use border-box 13px native checkbox/radio defaults, browser margins, automatic appearance, and HTML yellow/black mark defaults; embed Mozilla Bullet 1.1 as the `-moz-bullet-font` WOFF2 |
| `blitz-dom/assets/moz-bullet-font.ttf` | Mozilla Bullet 1.1 from mozilla-central `7d5056c95c12` (TrueType, SIL Open Font License 1.1) replaces upstream's `moz-bullet-font.otf` (Bullet 1.0 with CFF outlines); the `-moz-bullet-font` WOFF2 in `default.css` is version 1.1 too. Provenance and license in `blitz-dom/assets/moz-bullet-font.LICENSE.md` |
| `blitz-dom/src/lib.rs` | `BULLET_FONT` embeds `moz-bullet-font.ttf` |
| `blitz-dom/Cargo.toml` | Standalone manifest with explicit pinned dependencies |
| `blitz-dom/src/stylo_to_parley.rs` | Resolve automatic optical size before shaping; preserve explicit axis overrides |
| `blitz-dom/src/font_metrics.rs` | Use the same optical-size settings for font metrics |
| `blitz-dom/src/document.rs` | Cache generated counter text per element and pseudo-element; deliver loaded border images; track style/layout dependency convergence |
| `blitz-dom/src/util.rs` | Add the border image resource kind; an SVG image's linked pictures are drawn only from `data:` URLs, never read by path, and kept for `take_unread_svg_images` |
| `blitz-dom/src/node/text.rs` | Carry optional geometric-marker metadata through layout brushes |
| `blitz-dom/src/node/element.rs` | Store, clone, and debug border image resources; retain transient subgrid layout styles |
| `blitz-dom/src/layout/damage.rs` | Load border image sources through the shared image cache |
| `blitz-dom/src/stylo.rs` | Invalidate border image resources after style changes; supply container content sizes |
| `blitz-dom/src/resolve.rs` | Resolve counters in document order and invalidate changed text before layout; iterate style/layout after container dimensions or inherited grid tracks change |
| `blitz-dom/src/layout/mod.rs` | Register generated-counter resolution, table captions, and content-weighted automatic column allocation; apply size containment before descendant layout |
| `blitz-dom/src/layout/table.rs` | Preserve caption construction; include captions in layout children; reserve resolved half borders in collapsed-cell layout styles; allow per-layout column track overrides; retain fixed-layout mode and cell width preferences before grid stretching |
| `blitz-dom/src/node/node.rs` | Select collapsed-cell and subgrid layout styles without changing primary computed styles |
| `blitz-dom/src/node/stylo_data.rs` | Allow layout-only computed style references |
| `blitz-dom/src/layout/construct.rs` | Supply counter text to pseudo-elements; honor signed and reversed list starts |
| `blitz-dom/src/layout/inline/build.rs` | Translate inline bidi boundaries and block overrides into Unicode controls in layout text; reserve inside symbol advances without fallback fonts; insert discretionary wbr controls with local wrapping |
| `blitz-dom/src/layout/inline/layout.rs` | Keep intrinsic text widths at 1/64 CSS pixel precision instead of rounding every item to a whole pixel; scale atomic inline baseline lengths |
| `blitz-dom/src/layout/list.rs` | Render Roman and padded decimal markers; preserve suffix spacing and handle zero, negative, reversed, and explicit list values; shape tabular marker text and geometric-symbol advances |
| `blitz-dom/src/layout/text_transform.rs` | Keep shaping-only wbr controls out of capitalization word context; upstream locale-specific tests are not included in this runtime source subset |
| `blitz-paint/Cargo.toml` | Standalone manifest with explicit pinned dependencies |
| `blitz-paint/src/lib.rs` | Register blend conversion and expose painting diagnostics |
| `blitz-paint/src/text.rs` | Correct synthetic italic direction, honor font-synthesis-style, use symmetric stem widening, transform vertical selections per line, and snap axis-aligned device baselines; share glyph painting with text shadows; use cubic decoration waves and typographic overline/strike metrics; dispatch geometric markers |
| `blitz-paint/src/render.rs` | Include visible overflow, outlines, and shadows in effect groups; apply mix blending; keep table backgrounds within the grid box; rotate sideways vertical text per column; dispatch border images; scale spatial filters; retain visible ink from offscreen subtrees; position geometric outside markers |
| `blitz-paint/src/render/background.rs` | Isolate and blend background layers; tile SVG backgrounds, clip individual tiles, and report SVG/gradient tile-budget omissions; suppress hidden empty-cell backgrounds through row layers; share default SVG sizing |
| `blitz-paint/src/render/border.rs` | Apply outline offset before painting; translate collapsed borders below top captions; suppress hidden empty-cell borders |
| `blitz-paint/src/render/form_controls.rs` | Paint native checkbox/radio geometry, accent colors, contrast marks, disabled colors, and appearance suppression |
| `blitz-paint/src/filters.rs` | Scale blur and drop-shadow lengths into device pixels |
| `blitz-paint/src/render/box_shadow.rs` | Correct sigma, transform order, inset stacking, transparent entries, zero-area shapes, and rounded spread geometry |
| `blitz-paint/src/kurbo_css/non_uniform_radii.rs` | Remove obsolete radius averaging |
| `blitz-paint/src/kurbo_css/css_box.rs` | Build offset outline and shadow geometry with adjusted individual corner radii |

`blitz-paint/src/blend.rs`, `blitz-dom/src/layout/counters.rs`, and
`blitz-dom/src/layout/table_caption.rs`, `blitz-dom/src/layout/table_width.rs`,
`blitz-paint/src/render/table.rs`, `blitz-paint/src/text_shadows.rs`,
`blitz-paint/src/text_markers.rs`, and `blitz-paint/src/render/border_image.rs` are new Kition code. Engine integration tests cover
selector matching, group opacity, explicit clipping, transforms, outline offsets,
element/background blending, SVG repetition, optical size, synthetic slant,
synthesis suppression, glyph expansion, bidi overrides, counter
scopes, pseudo-element ordering, list numbering, caption text and geometry, and
collapsed border accounting for both inline and block cell contents. The upstream project remains the
source for its full test suite; these vendored crates are excluded from the root
workspace's test membership.

Known limits remain visible in the conformance corpus. In particular, rounding
and curve coverage still differ from Chrome, SVG/gradient explicit tiling is
limited to 500 tiles per layer, and container-query and JavaScript browser API
coverage remain partial. See [the findings](../docs/parity/findings.md).

Counter resolution follows the inheritance and instantiation rules in
[CSS Lists section 4.4](https://www.w3.org/TR/css-lists-3/#inheriting-counters).
The implemented named representations are decimal, decimal-leading-zero, Latin
alphabetic, and Roman (outside its range, Roman falls back to decimal). Custom
`@counter-style`, reversed CSS counters, `counter-set` (absent in this Stylo
build), and the implicit `list-item` counter in generated content remain
unsupported. HTML ordered-list numbering is handled separately.

Bidi controls follow [CSS Writing Modes section 2.4.2](https://www.w3.org/TR/css-writing-modes-3/#bidi-control-codes-injection).
Only the transient layout text receives controls; HTML source and saved document
bytes do not. Paragraph-separator reinsertion, source-to-layout offset mapping
for future editing, and the complete bidi Web Platform Tests suite still need
work. The current acceptance evidence covers the tested single-paragraph cases.

Table captions follow the wrapper/grid distinction in
[CSS 2.2 section 17.4](https://www.w3.org/TR/CSS22/tables.html#model).
Captions participate in text construction and take space above or below the grid;
explicit widths, auto horizontal margins, padding, borders, and table grid height
are accounted for separately. Backgrounds and collapsed border painting use the
grid bounds. Collapsed cells reserve half of each winning shared border in both
their grid styles and layout-only computed styles, so nested blocks and inline
text use the same content box. Primary styles remain available to the painter.

This is not a complete CSS table implementation. Caption intrinsic minimum widths,
vertical table writing modes, caption overflow/positioning edge cases, table row
and group geometry, constrained automatic columns, and full border conflict
resolution remain incomplete. Collapsed cell geometry includes its resolved half
borders. The tested definite row-height and
percentage-column cases now stretch/distribute their tracks; broader intrinsic
constraints remain incomplete. Caption integration tests and the existing table fixtures
cover the behaviors measured here, not the full tables specification.

## Glifo 0.4.0

`glifo/` contains the runtime sources, normalized manifest, README, and licenses
from the published [Glifo 0.4.0 crate](https://crates.io/crates/glifo/0.4.0), used by
Vello CPU 0.3.0. Original crate checksum:
`9f1218dedd5348c1b166ec3b8b3cbca49d2f527bd43b3fa072ea5123c523ed3f`.
The root Cargo patch selects this copy for native and WASM builds. Transitive
versions remain unchanged. The upstream MIT and Apache licenses are retained.

In `glifo/src/glyph.rs`, outline expansion and its
curve tolerance are converted from rendered font pixels to cached outline units.
Unhinted outlines are cached at units-per-em and scaled down during drawing;
without the conversion, expansion almost disappears. Hinted outlines already use
rendered font pixels and retain a conversion factor of one. Integration tests
measure visible expansion at 16, 32, and 64 pixels and compare hinted/unhinted
bounds through the actual Vello CPU renderer.

The root enables Blitz's `font-embolden` feature on every target, using the same
Mac-style stem-widening profile for native and WASM. The profile's horizontal and
vertical total widening are `min(0.015125 * raster_size, 0.3)` and
`min(0.0121 * raster_size, 0.3)` rendered pixels. The raster-size continuation
converts this amount to local outline units before transforming it; the original
profile used CSS size. Each outline edge receives half.
This is an approximation for the fixed Chrome reference environment, not a
claim that all operating systems or fonts rasterize identically. No screenshot
postprocessing, comparison masks, or per-fixture adjustments are used.

Synthetic slant uses the downward-y drawing convention: positive CSS angles
move the top of the glyph right. `font-synthesis-style: none` suppresses this
fallback skew; actual italic font faces/axes remain eligible. The engine enables
Stylo's synthesis-property parser before parsing CSS. This does not add synthetic
bold, small caps, or superscript/subscript synthesis. Signed oblique angles and
synthesis suppression are tested at DPR 1 and 2.

## Inline intrinsic widths

Text intrinsic widths use upward rounding at 1/64 CSS pixel precision before
returning to Taffy. The previous whole-pixel ceiling accumulated excess width
across adjacent flex items and changed space distribution. The same CSS precision
is used at fractional and integer device scales. Tests compare six glyph widths
and their accumulated extent with fixed-font Chrome measurements at DPR 1, 1.5,
and 2. Constrained automatic table columns remain incomplete.


## Automatic tables and empty cells

Unconstrained automatic columns use measured min/max content widths. Extra width
is distributed in proportion to max-content width; intermediate widths interpolate
between intrinsic bounds. Spanning cells contribute to every covered column, and
border spacing is removed before distribution. The collapsed model includes the
two adjacent half borders in column proportions. Proportional widths use 1/64 CSS
pixel precision with the final column receiving the remainder. This follows the
content-weighted approach discussed in the
[CSS Tables draft](https://www.w3.org/TR/css-tables-3/#width-distribution) and the
fixed-point allocation behavior in
[Blink's table layout](https://chromium.googlesource.com/chromium/src/+/7080062aac0539720300be8a6a052d14e50506b7/third_party/blink/renderer/core/layout/table/table_layout_utils.cc).
Fixed and constrained columns retain the previous path.

In separated-border tables, `empty-cells: hide` suppresses the empty cell's border
and backgrounds from the cell, row, and column layers. Collapsible whitespace and
out-of-flow children do not make a cell nonempty; preserved whitespace, in-flow
boxes, and generated content do. Positioned descendants continue to paint. The
collapsed model ignores this property. Empty-row height suppression remains
incomplete. See [CSS 2.2 tables](https://www.w3.org/TR/CSS22/tables.html#empty-cells).

## Sideways vertical text

Latin text in vertical writing modes now paints through a per-column rotation,
including inline backgrounds, selections, and decorations. Vertical right-to-left
and left-to-right modes have opposite column progression; `sideways-lr` reverses
the inline direction. Stylo's sideways writing-mode parser is enabled. Tests
compare a vertical run with rotated horizontal output at DPR 1 and 2 and verify
four writing-mode column directions. Upright glyph shaping emits an explicit
diagnostic and remains unsupported, as do complete mixed-script vertical font
metrics and vertical atomic-inline placement. This is partial support for
[CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/#text-orientation).


## Parley 0.12.0

`parley/` contains the runtime sources, normalized manifest, README, and original
MIT and Apache licenses from [Parley 0.12.0](https://crates.io/crates/parley/0.12.0).
Original crate checksum:
`61bd0c0c1737f0b3d3726dc06b225083af213b61913e4fbae1670613a106c6ce`.
The root Cargo patch uses this copy on native and WASM. Upstream snapshot tests
and their test-only module registration are omitted from this runtime subset;
the upstream project remains the source for its complete test suite.

Modified files: `src/builder.rs`, `src/layout/data.rs`,
`src/layout/line_break.rs`, `src/layout/style_metrics.rs`, and `src/style/mod.rs`
carry the layout scale through internal quantization. `src/layout/mod.rs`
registers the new `src/layout/quantization.rs` helper. `src/lib.rs` omits the
snapshot test module. Public builder options and layout APIs remain unchanged.

Quantized font metrics, normal line heights, and line baselines round in CSS
pixels before device scaling. Previously these rounded in device pixels, so
changing DPR could change CSS layout height or baseline. Integration tests cover
three fonts, five sizes, normal and explicit line heights, and DPR 1, 1.5, 2, and 3.
The painter separately snaps the device-space coordinate perpendicular to an
axis-aligned baseline, preserving fractional positions along it and arbitrary
rotations. This follows the axis rounding distinction in
[Skia's glyph positioning](https://github.com/google/skia/blob/main/src/core/SkGlyph.cpp).
The paint adjustment does not change DOM geometry or glyph advances.

## Border images

The painter implements nine-slice gradient, raster, and SVG border images,
including percentage/number slices, optional center fill, widths and outsets,
and stretch/repeat/round/space tiling. Border images replace ordinary borders
without changing layout dimensions. Missing resources retain the normal border.
Raster and SVG URLs use the same resource cache and style invalidation path as
background and mask images. SVG sources use CSS default object sizing and their
viewport transform. Every tile is clipped to its patch; an explicit diagnostic
reports a patch that exceeds the 500-tile budget.

The behavior follows [CSS Backgrounds section 5](https://www.w3.org/TR/css-backgrounds-3/#border-images).
Tests exercise all repeat modes, slice overlap, percentage and automatic widths,
center fill, outset, shared background resources, missing images, and PNG/SVG
sources at DPR 1, 1.5, and 2. Other source functions such as `image-set()` and
`cross-fade()` are not implemented. The full CSS border image WPT suite has not
been run.


## Stylo and container queries

`stylo/` contains the style crate runtime sources from
[Stylo revision d6502e0002e9c40c43a0a4776f9e7eb93be360e3](https://github.com/DioxusLabs/stylo/tree/d6502e0002e9c40c43a0a4776f9e7eb93be360e3/style).
Sibling crates stay pinned to that revision. Source copyright notices remain
intact, and `LICENSE-MPL-2.0` includes the
[Mozilla Public License 2.0 text](https://www.mozilla.org/MPL/2.0/).
The normalized manifest expands workspace dependencies and uses the upstream
workspace README locally. The root patch selects the same source on both targets.

Modified upstream files:

| File | Change |
| --- | --- |
| `stylo/Cargo.toml` | Standalone manifest with explicit pinned sibling dependencies |
| `stylo/README.md` | Use the upstream workspace README referenced by the original manifest |
| `stylo/stylesheets/rule_parser.rs` | Permit the existing container-rule parser when the Servo container-query preference is enabled |
| `stylo/values/specified/length.rs` | Permit the existing six container-relative units under that preference |
| `stylo/properties/longhands.toml` | Expose the existing accent-color property in Servo builds |
| `stylo/properties/shorthands.toml` | Make the container shorthand available to Servo behind the preference |
| `stylo/properties/shorthands.rs` | Compile the existing shorthand parser and serializer for Servo |
| `stylo/values/computed/box.rs` | Remove an emoticon from a comment; no behavior change |

Kition enables the preference before parsing CSS. The new
`blitz-dom/src/layout/container_queries.rs` supplies content-box sizes on the
contained axes, preserving unrounded dimensions and writing-mode conversion.
Size containment measures an empty box on those axes, then lays descendants out
inside it. Non-atomic inlines, contents/none, tables, and internal table boxes
cannot provide size queries. Container snapshots trigger another style/layout
pass when dimensions change; nested queries settle within a bounded 32-pass
loop, with an explicit engine diagnostic if that limit is reached.

Nine integration tests cover content-box sizing, named nested containers, both
axes, uncontained axes, container units and viewport fallback, empty intrinsic
measurement, block fill, and vertical writing modes. An independent Chrome probe
covers 11 cases at DPR 1 and 2, including table exclusions: all 22 images are exact.
The original container-query corpus fixture is also exact. This is partial
coverage of [CSS Conditional Rules 5](https://www.w3.org/TR/css-conditional-5/)
and [CSS Containment 2](https://www.w3.org/TR/css-contain-2/#containment-size),
not completion of their WPT suites. Intrinsic-size overrides, counter isolation
from style containment, scroll-state queries, and animated query transitions
still need dedicated implementation or verification.

## Shadows and Gaussian filtering

Box shadows use half the CSS blur radius as Gaussian sigma, as specified in
[CSS Backgrounds section 6.1](https://www.w3.org/TR/css-backgrounds-3/#box-shadow).
Offsets and blur lengths scale with DPR and compose before the element's CSS
transform. Inset shadows paint back to front, skip transparent entries, and
use the padding-edge perimeter. Contracted shapes stop at zero area. Spread
adjusts each corner radius independently, including the cubic transition for
small outward radii. Inset masks are the exterior of the shifted hole, clipped
to the padding edge; outset masks retain the border-edge knockout.

Blurred shapes use an actual isolated filter instead of the backend's approximate
rounded-rectangle shader. Kition-owned `src/blur.rs` in the engine uses three
box convolutions, including the identity window for small subpixel sigmas,
retaining integer sums until the end of each axis. The window construction
follows the method described in
[SVG Gaussian blur](https://www.w3.org/TR/SVG11/filters.html#feGaussianBlurElement)
and used by [Skia's raster blur engine](https://skia.googlesource.com/skia/+/c6e63919c318/src/core/SkBlurEngine.cpp).
This is an independent Rust implementation; it does not link Skia or Chrome.

The scene adapter handles transparent-edge single blurs up to device sigma 135
with axis-aligned transforms, including separate horizontal and vertical scales.
Other blurs retain the Vello path. Temporary images retain offscreen input within
the blur's support. Layout-only subtree culling is removed because scrollable
overflow does not bound shadow, filter, outline, or border-image ink.

Coverage still differs at curved antialiased edges, translucent compositing,
arbitrary rotations and large blurs. The 34-case shadow
probe spans 17 independent cases at DPR 1 and 2; it supplements the unchanged
128-case corpus and does not replace its failures.

A separate 32-case blur probe covers sigma 0 through 32, DPR 1 and 2, nonuniform
scale, quarter-turn rotation, and offscreen input. All cases reach 99.9%, with
a minimum of 99.922673%. This only describes the measured probe, not all filters.

## Text shadows

The text painter now renders inherited shadow lists behind foreground glyphs,
resolves currentColor per run, honors `none`, applies scaled offsets before CSS
transforms, and paints lists back to front. Shadows and foreground text share
one glyph painter, including font synthesis, stem widening, and baseline phase.
Adjacent runs with identical resolved shadow lists share a blur mask, preserving
continuity across equal inline styles. Shadow color is independent of transparent
foreground color. Tests cover DPR 1, 1.5, and 2, ordering, inheritance, suppression,
inline boundaries, and rotation.

Decorations are not yet included in shadow masks, and vertical-writing shadow
offsets and overlapping groups with distinct lists need further implementation
and verification. Font rasterization remains approximate. See
[CSS Text Decoration section 3](https://www.w3.org/TR/css-text-decor-3/#text-shadow-property).

## Path precision and native controls

The engine refines non-glyph fill and clip curves at 1/16 device-pixel tolerance,
using the largest transform singular value. Strokes are expanded before that
refinement. Glyph runs retain their separate font rasterization path. Non-finite,
excessive-coordinate, and excessive-element paths retain the existing backend
path. Tighter 1/64 and 1/256 tolerances were measured independently and did not
consistently improve Chrome agreement; this is a bounded geometric tolerance,
not a per-fixture selection. Curved-edge antialiasing remains different.

The UA stylesheet now supplies border-box 13px checkbox/radio defaults and native
margins. The engine enables appearance parsing before the default stylesheet is
constructed. The painter honors appearance:none, inherited accent-color and
currentColor, bright-accent contrast, and explicit disabled states. Rectangular
controls center an inscribed square indicator. Checked ticks use butt ends and
miter joins. Geometry and colors follow the platform-independent reference in
[Chromium's native theme](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/ui/native_theme/native_theme_base.cc)
and its [HTML stylesheet](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/third_party/blink/renderer/core/html/resources/html.css).

Six integration tests and a separate 40-case Chrome probe cover default and
explicit sizes, checked/unchecked/disabled states, colors, and DPR 1 and 2.
Inherited fieldset disabling, indeterminate state, focus/hover, dark themes, and
forced colors are not yet fully implemented or verified. Native control curve
coverage remains approximate.

## Text decoration geometry

Wavy decorations use repeated cubic waves, half-pixel control coordinates,
line-specific vertical offsets, overhang, and endpoint clipping. Overlines use
the rounded typographic ascent rather than the font's Windows clipping ascent;
strikes use floating ascent from the rounded text-top. The reference algorithms
are in Chromium's [decoration painter](https://chromium.googlesource.com/chromium/src/+/147.0.7727.15/third_party/blink/renderer/core/paint/decoration_line_painter.cc)
and [decoration offsets](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/third_party/blink/renderer/core/layout/text_decoration_offset.cc).
Three integration tests cover wave period across DPR, endpoint clipping, and
line placement. An independent 18-case probe spans three line types, three
thicknesses, and two DPRs. Thin-stroke coverage and fractional strike placement
still differ from Chrome; this does not claim complete decoration conformance.

## Baseline shifts and list markers

Subscript and superscript shifts include one CSS pixel beyond the parent font's
one-fifth or one-third fraction. The pixel is scaled with DPR. Atomic inline
baseline lengths are resolved from CSS afresh on each layout, preventing missing
DPR scaling or repeated multiplication. The reference is Chromium's
[inline box positioning](https://chromium.googlesource.com/chromium/src/+/147.0.7727.15/third_party/blink/renderer/core/layout/inline/inline_box_state.cc).
Tests compare keyword shifts to explicit lengths across font sizes and DPRs.

Disc, circle, square, and disclosure markers use geometric painting and the
principal font's metrics. A neutral layout-only placeholder reserves their
advance without introducing a fallback font's ascent, glyph, or bidi direction.
Inside and outside spacing follow the symbol metrics; disclosure arrows follow
the inline direction. Marker text receives the UA tabular-figure feature, which
also changes figure punctuation in fonts such as Inter. Source HTML is unchanged.
The references are Chromium's [marker layout](https://chromium.googlesource.com/chromium/src/+/147.0.7727.15/third_party/blink/renderer/core/layout/list/list_marker.cc)
and [symbol painting](https://chromium.googlesource.com/chromium/src/+/147.0.7727.15/third_party/blink/renderer/core/paint/text_fragment_painter.cc).

Tests cover principal-text preservation, line heights, RTL disclosure direction,
and marker suppression. A separate 60-case Chrome probe covers five symbols,
inside/outside positioning, three font sizes, and DPR 1 and 2. Curved-edge
antialiasing, the full ::marker author-style cascade, vertical marker layouts,
and source-to-layout editing offsets remain incomplete.

## Inherited subgrid tracks

The engine enables Stylo's existing subgrid parser. New Kition code in
`blitz-dom/src/layout/subgrid.rs` resolves a subgrid's spanned columns and rows
from the parent's used track positions and placement. Transient layout styles
carry those sizes, normal inherited gaps, explicit gap adjustments, edge padding,
borders and margins, and inherited/local line names. Opposite inline directions
reverse the inherited column sequence. Nested subgrids settle in the same bounded
style/layout loop as container sizes; stale overrides are discarded on restyle
or when an element stops being a grid. Source and primary computed styles remain
unchanged. The reference is [CSS Grid 2 subgrids](https://www.w3.org/TR/css-grid-2/#subgrids).

Nine integration tests cover both axes, nested grids, fractional DPR, named lines,
explicit gaps, edge insets, RTL, and container-query transitions. A separate
36-case Chrome probe covers 18 scenarios at DPR 1 and 2: 34 are pixel-exact. The
two intrinsic-parent cases remain at 84.033613%, because descendant intrinsic
contributions are not yet integrated into the parent's track-sizing algorithm.
Orthogonal writing modes and nonuniform collapsed auto-fit gutters retain the
upstream fallback. Full subgrid intrinsic sizing, overflow placement constraints,
and all grid fragmentation behavior remain incomplete.


## Uniform-opacity compositing

`vello_cpu` is vendored from crates.io version 0.3.0 (Apache-2.0 OR MIT), with
upstream licenses and its registry manifest retained. Modified upstream files:

| File | Change |
| --- | --- |
| `vello_cpu/src/fine/mod.rs` | Dispatch normal uniform-opacity layer composition to a numeric-kernel hook before separately rounding the source |
| `vello_cpu/src/fine/lowp/mod.rs` | Combine premultiplied source and backdrop in fixed-point arithmetic, including clip coverage and zero-alpha handling |

The low-precision normal source-over path quantizes group alpha to a byte and
uses a 0..256 scale, retaining both products until their final truncation.
Non-normal blend modes, explicit masks, and the floating-point kernel retain the
upstream path. Kition's scene adapter folds a lone solid draw's opacity into its
paint while retaining the clip stack and nested opacity boundaries. Solid-paint
alpha is quantized before RGB premultiplication. These operations happen during
rendering; no comparison image is adjusted.

Reference arithmetic: Skia's [32-bit row compositing](https://github.com/google/skia/blob/main/src/core/SkBlitRow_D32.cpp)
and [alpha conversion](https://github.com/google/skia/blob/main/src/core/SkColorPriv.h).
The 50-color/opacity sample combinations match Chrome for both single draws and
multiple-draw groups. An independent 72-case probe covers nested groups, colored
backdrops, translucent children, clipping, transforms, gradients, and blending
at DPR 1 and 2: 71 pass 99.9%, with 50 exact. The remaining rotated-edge case is
99.855948%. Chrome's tile-local single-draw optimization still produces a one-level
difference in part of the corpus opacity case. Inner translucent source-over
rounding and paint ownership across static stacking contexts remain incomplete.


## Out-of-flow paint ownership

`blitz-dom/src/layout/paint_tree.rs` keeps a positioned descendant in the nearest
intervening stacking context even when Taffy lays it out against an outer
containing block. The positioning-chain offset also handles this inverse
relationship. Paint lists are rebuilt when redirected ownership is present,
preventing stale topology from bypassing atomic effects. `node/node.rs` treats
mix blending and explicit isolation as atomic paint contexts. `blitz-paint/src/render.rs`
uses the corresponding offsets, paints explicit isolation, and temporarily
suspends its current overflow clip for an externally positioned child without
removing opacity, masks, or clip paths.

References: [CSS overflow exceptions](https://www.w3.org/TR/CSS22/visufx.html#overflow-clipping),
[opacity grouping](https://www.w3.org/TR/css-color-3/#transparency), and
[blend-mode stacking contexts](https://www.w3.org/TR/compositing-1/#mix-blend-mode).
Eight integration tests cover coordinate preservation, nested groups, viewport
positioning, overflow, mixing, isolation, and explicit unsupported diagnostics.
The independent 40-case probe has 38 passes and 25 exact results. Escaping an
overflow clip outside the current paint context still needs a more complete clip
ancestry model; those two failures are retained and reported by the renderer.

## Horizontal multi-column layout

The implementation follows the model in [CSS Multi-column Layout Level 1](https://www.w3.org/TR/css-multicol-1/)
and the line-break constraints in [CSS Fragmentation Level 3](https://www.w3.org/TR/css-break-3/#widows-orphans).
A transient narrow layout feeds bounded line/block fragments; the painter replays
that flow through each fragmentainer. Chrome remains an external test oracle.

Modified upstream files for this change:

- `stylo/properties/longhands.toml`: expose column rule/fill and break-inside
  properties to Servo; add inherited positive-integer widows/orphans properties.
- `stylo/properties/shorthands.toml` and `shorthands.rs`: expose the existing
  column-rule shorthand parser to Servo.
- `stylo/properties/counted_unknown_properties.py`: remove the two newly parsed
  widow/orphan properties from the unknown-property counter list.
- `blitz-dom/src/node/element.rs` and `node.rs`: retain temporary column layout
  styles and resolved fragments without changing primary computed styles.
- `blitz-dom/src/layout/mod.rs`: dispatch multi-column layout and clear stale
  fragments when the element returns to ordinary flow.
- `blitz-dom/src/document.rs`: map descendant bounding rectangles through nested
  fragmentainers before returning their viewport-relative union.
- `blitz-dom/src/lib.rs`: expose fragment metadata to the painter.
- `blitz-paint/src/render.rs`: factor flow painting, retain unrounded fragment
  positions, snap box edges in each column, and diagnose unsupported spanning,
  vertical fragmentation, and fragment-aware positioned containing blocks.

`blitz-dom/src/layout/multicol.rs` and `blitz-paint/src/render/multicol.rs` are new
Kition code. They support count/width selection, normal/percentage gutters,
RTL progression, balancing, sequential filling, min/max/fixed heights, line
widows/orphans, break-inside avoidance, empty-block splitting, nested columns,
sliced box decoration, and solid/double/dashed/dotted rules. They bound each
layout to 1,024 fragments and total painting to 4,096 fragments and eight nested
levels, with explicit diagnostics when a limit is reached.

This is a partial fragmentation implementation. Spanners, forced column breaks,
orthogonal columns, fragment-aware positioned containing blocks, and full
fragmented hit testing remain unsupported. Oversized avoided/replaced boxes,
complex floats, cloned box decorations, minimum-content sizing, and constrained
nested fragmentation still need broader coverage. A three-dimensional rule style
is painted solid with a diagnostic. The existing strict corpus and independent
DPR 1/2 probes retain every failure, including glyph rasterization differences.

## Border and rule patterns

`blitz-paint/src/render/line_pattern.rs` adapts the spacing and narrow-dot
endpoint arithmetic from Chromium 147's
[styled stroke data](https://chromium.googlesource.com/chromium/src/+/refs/tags/147.0.7727.15/third_party/blink/renderer/platform/graphics/styled_stroke_data.cc)
and [box border painter](https://chromium.googlesource.com/chromium/src/+/refs/tags/147.0.7727.15/third_party/blink/renderer/core/paint/box_border_painter.cc).
The original copyright notices and BSD terms are retained in
`blitz-paint/LICENSE.google-stroke` and `blitz-paint/LICENSE.chromium`.
No Chromium runtime or browser dependency is introduced.

`blitz-paint/src/render.rs` registers the helper; `render/border.rs` uses it for
straight sides and starts rounded contours after the upper-left arc. Dashes keep
their nominal length while the gap is selected to fit the path. Narrow dotted
sides preserve complete integral endpoints. Matching straight dashed/thin-dotted
sides share one fill to preserve translucent corners. `render/multicol.rs` uses
the same helper for column rules. Thick circular dots, closed-contour seams,
curved clipping, and transformed coverage still differ from Chrome's rasterizer.


## Foreground-dependent grayscale glyph coverage

The existing outline-widening profile approximates black text. Its coverage is
converted back through the inverse black sRGB preblend, then through the requested
foreground's preblend. This leaves black masks unchanged and gives white/colored
text its own edge coverage. It is an approximate shared-engine rendering profile,
not a CoreText implementation or an operating-system font dependency.

The color selection follows Skia's macOS non-LCD smoothing path: canonicalize
channels to three bits, halve them, canonicalize again, compute integer luminance,
and select the correcting table. Four generated 256-byte tables preserve zero,
full coverage, monotonicity, and the identity black table. Regenerate or verify
with `python3 tools/generate-glyph-coverage.py [--check]`. Runtime does no floating
point transfer-function evaluation, and native/WASM use the same table bytes.

| File | Change |
| --- | --- |
| `glifo/src/interface.rs` | Add an outline-fill hook with the original path-fill behavior as its default |
| `glifo/src/renderer.rs` | Route uncached outline glyph fills through that hook; color and bitmap glyph drawing remains separate |
| `vello_cpu/src/lib.rs` | Register the internal coverage module |
| `vello_cpu/src/glyph_gamma.rs` | Select the color table and apply it only to newly generated glyph alpha bytes |
| `vello_cpu/src/glyph_gamma_tables.rs` | Store generated integer lookup tables |
| `vello_cpu/src/text.rs` | Apply the profile to solid, unfiltered outline glyph fills |
| `vello_cpu/src/render.rs` | Carry an optional coverage-table index through the existing path fill |
| `vello_cpu/src/dispatch/mod.rs` | Extend path dispatch with the optional table index |
| `vello_cpu/src/dispatch/single_threaded.rs` | Transform only the current glyph's appended coverage range |
| `vello_cpu/src/dispatch/multi_threaded.rs` | Carry the optional index in worker path tasks |
| `vello_cpu/src/dispatch/multi_threaded/worker.rs` | Apply the same transformation after worker rasterization |
| `vello_cpu/LICENSE.skia` | Retain the BSD terms for adapted arithmetic |

Reference sources are the exact Skia revision used by Chrome 147.0.7727.15:
[macOS color reduction](https://skia.googlesource.com/skia/+/fd139e79fd4553ba16556c1bca6c0f4dd9bc6fa5/src/ports/SkTypeface_mac_ct.cpp),
[mask preblend](https://skia.googlesource.com/skia/+/fd139e79fd4553ba16556c1bca6c0f4dd9bc6fa5/src/core/SkMaskGamma.cpp),
and [canonical channel selection](https://skia.googlesource.com/skia/+/fd139e79fd4553ba16556c1bca6c0f4dd9bc6fa5/src/core/SkMaskGamma.h).
Original Android and Google copyright notices and BSD terms are retained.

The kit continues to use direct glyph rendering. Enabling the upstream optional
atlas uses its upstream coverage behavior; the kit does not enable that mode.
Ordinary paths, images, bitmap/color glyphs, gradient glyph paint, and explicitly
filtered glyph paths retain their previous behavior. CoreText dilation, scan
conversion, fractional clipping, and other operating systems remain imperfect.


## Outline styles and bevel quantization

`blitz-paint/src/render/outline.rs` replaces the solid fallback for explicit
outline styles. It constructs an outer border frame at the outline offset and
reuses dashed/dotted border painting. Double stripes use integral CSS widths;
groove/ridge halves round toward the top/left side. Curved dash/dot contours
truncate half-widths in CSS pixels before measuring their perimeter, matching
Chromium's `CenterOutsets`; fractional halves can select a different dash count.
Equal-color paths merge
before painting so narrow translucent corners are covered once. The existing
solid-outline path is preserved.

`blitz-paint/src/render.rs` registers this module and makes the paint context
clonable for the transient outline frame. Outlines paint above their own box
background and borders, preserving negative-offset visibility; ordinary-flow
descendant outline phases remain incomplete. `render/border.rs` exposes the shared
pattern/shading helpers and quantizes generated light/dark shades with the
representable float immediately below 256, following Chromium's
[pinned color implementation](https://chromium.googlesource.com/chromium/src/+/147.0.7727.15/third_party/blink/renderer/platform/graphics/color.cc).
Odd-width shading follows its
[box-side painter](https://chromium.googlesource.com/chromium/src/+/147.0.7727.15/third_party/blink/renderer/core/paint/box_border_painter.cc).
`LICENSE.chromium` retains the box-side terms; `LICENSE.apple-color` retains
the original Apple notice and terms for color quantization.

Rounded dotted/dashed contours, fractional clipping, and curved double/bevel
edges remain approximate. Auto focus rings and disconnected inline outlines
remain unsupported. `tools/build-wasm.sh` includes vendored licenses in the
output package's `licenses/` directory.


## HTML break opportunities and highlight defaults

`blitz-dom/src/layout/inline/build.rs` inserts a zero-width space into the shaping
stream for a rendered `wbr`. Its local wrapping mode stays enabled inside
`nowrap`, `pre`, and `nobr`, without adding letter spacing or an obligatory line
break. Positioned/floating controls retain their break opportunity; hidden and
`display:contents` controls do not create one. The source HTML is unchanged.

`blitz-dom/src/layout/text_transform.rs` records the byte positions of these
inserted controls and removes only those positions from capitalization context.
A `wbr` therefore does not capitalize the rest of the same word; real whitespace
and mandatory breaks still start words. Literal text controls are not rewritten.

`blitz-dom/assets/default.css` now gives `mark` the HTML yellow background and
black foreground. The previous platform `Mark` system color resolved to an
orange-yellow value. Explicit CSS system colors keep their existing behavior,
and author styles continue to override the UA defaults. Abbreviation and `nobr`
rules already existed and are covered by regression tests rather than duplicated.

The reference is the HTML Standard's
[phrasing rendering section](https://html.spec.whatwg.org/multipage/rendering.html#phrasing-content-3),
with edge cases verified against the pinned Chrome oracle. Glyph rasterization,
ruby layout, fieldset legends, and progress/meter rendering remain incomplete.

## Constrained automatic table columns

`blitz-dom/src/layout/table_width.rs` now distributes automatic table widths
between content minima, definite column preferences, and content maxima.
Automatic columns receive surplus before definite columns; empty automatic
columns share it equally. Cell width preferences survive the grid adapter and
include later rows. Spanning cells use the same distribution stages, rather
than growing each column independently of its preferred width. Definite column
widths can shrink toward content minima, and table boxes grow when those minima
exceed the requested width. Fixed layout retains its separate track path.

The algorithm follows the staged distribution described in the
[CSS Tables draft](https://www.w3.org/TR/css-tables-3/#width-distribution-algorithm)
and was checked against
[Chromium's table layout utilities](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/third_party/blink/renderer/core/layout/table/table_layout_utils.cc),
including its reuse of automatic distribution for spanning-cell measures.
The corresponding BSD notice is retained in `blitz-dom/LICENSE.chromium`.
Percentage/calc column constraints, complete border conflict resolution, and
vertical table sizing remain outside this change. Integration regressions live
in `crates/kition-html-engine/tests/table_constraints.rs`.


## JavaScript and scripting-aware HTML

`blitz-vibey-script/src/` and `blitz-html/src/` are imported from the same pinned
Blitz revision as the DOM. Their standalone Cargo manifests preserve upstream
versions, licenses, and dependency identities. Both run on native and wasm32.
Local modifications to upstream files are:

- `blitz-vibey-script/src/runtime.rs`: enable Boa's instruction counter with a
  5,000,000 instruction session budget, disallow blocking Atomics, and bound loop
  iterations and recursion. Boa's `fuzz` feature supplies the instruction counter;
  no host JavaScript evaluator is used.
- `blitz-vibey-script/src/fetch.rs`: remove implicit filesystem fetching, including
  the non-portable WASM file-URL conversion. Kition installs its supplied-resource
  fetcher before running page code.
- `blitz-vibey-script/src/document.rs`: recognize HTML's legacy JavaScript MIME
  types, skip classic `nomodule`, and order deferred external scripts/modules
  after ordinary scripts. This does not implement parser-blocking execution.
- `blitz-html/src/html_sink.rs`: enable scripting-aware document and fragment
  parsing so `noscript` contents remain raw text.
- `blitz-dom/src/node/serialize.rs`: preserve raw-text `noscript` serialization
  under that parsing policy.
- `stylo/servo/media_features.rs`: implement the `scripting` media feature for
  Kition's always-enabled live renderer, including boolean, `enabled`, `none`,
  and `initial-only` queries. `stylo/queries/feature_expression.rs` accepts
  the lazily initialized feature table, whose new atom is interned at runtime.
  This activates the existing UA `noscript` hiding
  rule. The other scripting values parse but do not match.

The host-independent engine owns the live session and bounded virtual clock.
Native KitDocument servers hold a message-passing handle to an owned script
thread; WASM keeps the same session in its existing engine Worker. No unsafe
Send implementation or host DOM injection is introduced.

## Extended DOM events, forms, and module resources

The following local changes implement common page behaviors against the same
engine DOM used for layout. They do not delegate execution to a host browser.

- `blitz-vibey-script/src/dom/dispatch.rs`, `dom/event.rs`, `events.js`,
  `runtime.rs`, and `state.rs`: Event/CustomEvent/UIEvent/MouseEvent constructors,
  capture/target/bubble dispatch, cancelability, once/removal during dispatch,
  object listeners, inline handlers, reentrancy guards, and shared native events.
- `blitz-vibey-script/src/dom/activation.rs`, `dom/forms.rs`, `dom/element.rs`,
  and `dom/node.rs`: checkbox/radio preactivation and cancel restoration,
  input/change notifications, label forwarding, details toggling, cancelable
  form reset, live versus default values, option selection, indeterminate state,
  and cloning of live form values without rewriting source defaults.
- `blitz-vibey-script/src/dom/html_mutation.rs` and `dom/mod.rs`: contextual
  adjacent/outer HTML insertion and innerText newline replacement.
- `blitz-vibey-script/src/collections.js` and `dom/document.rs`: live children,
  childNodes, class/tag collections, and static query-selector NodeLists.
- `blitz-vibey-script/src/media.js` and `blitz-dom/src/resolved_style.rs`:
  matchMedia evaluates against the engine's Stylo device.
- `blitz-vibey-script/src/import_map.rs`, `document.rs`, `lib.rs`, and
  `Cargo.toml`: supplied-resource import maps, URL/scoped/prefix mappings,
  blocked mappings, and registration before module execution. JSON parsing uses
  serde_json; resources retain the existing host-network isolation.
- `blitz-dom/src/control_state.rs`, `lib.rs`, `document.rs`, `mutator.rs`,
  `node/element.rs`, `node/node.rs`, and `stylo.rs`: live value and checkedness
  storage, radio ownership, form reset, current DOM owner resolution, label
  resolution, disabled fieldset/legend rules, and state selector invalidation.
- `blitz-dom/src/layout/construct.rs` and `events/keyboard.rs`: pristine values
  follow default text while edits set the dirty-value flag.
- `blitz-paint/src/render/form_controls.rs`: disabled ancestors affect native
  control colors, and indeterminate checkboxes paint a horizontal mark.

EventTarget subclasses, passive listeners, AbortSignal, full form validation,
complete select UI, and keyboard forwarding from the kit editor remain incomplete.

## Percentage tables and repeated backgrounds

- `blitz-dom/src/layout/table.rs` and `layout/table_width.rs` preserve percentage
  column preferences, distribute competing percentage/fixed/automatic widths,
  and round layout targets to 1/64 CSS pixels. Explicit cell heights are minima,
  allowing cell backgrounds to fill taller rows.
- `blitz-dom/src/layout/inline/layout.rs` measures empty inline contexts normally
  so an empty descendant cannot erase its parent's explicit/minimum dimensions.
- `blitz-paint/src/gradient.rs` centers angle gradients on each repeated tile.
  `render/background.rs` composes tile translations before element transforms,
  bounds round-repeat counts, and rejects empty or non-finite tile sizes.
- `blitz-paint/src/render.rs` clamps replaced-image edge texels. Background and
  border-image tiles retain their separate repetition behavior.

Fractional repeat-space filtering, percentage calc constraints, and full table
border conflict resolution remain under comparison.

## Canvas 2D backing bitmaps

`blitz-vibey-script/src/canvas.js` and `canvas.rs` implement an engine-backed 2D
context: rectangles, paths, quadratic/cubic curves, arcs/ellipses, transforms,
state save/restore, clipping, line dashes, solid/gradient paints, compositing,
and ImageData read/write. `runtime.rs`, `lib.rs`, `state.rs`, `document.rs`, and
`Cargo.toml` register it and let the embedder reuse its CPU scene preparation.
The engine applies its shared path, gradient, and opacity pipeline to canvas.

`blitz-dom/src/node/element.rs` owns transient bitmap data; `mutator.rs` resets
it on width/height changes, including assignment of the same size. Cloning a
canvas keeps its attributes and produces an empty bitmap. Rendering consumes
these images through the existing replaced-element painter. Source serialization
and document dirty tracking never include the pixels.

Canvas geometry, path/clip/state counts, and bitmap dimensions have explicit
preview limits. Drawn curves and alpha rounding can differ from Chrome. Patterns,
shadows, filters, export, WebGL, and OffscreenCanvas are not provided;
unsupported method calls remain visible as script diagnostics. Assigning an
unsupported property can have no visual effect.

## Fieldset and legend layout

`blitz-dom/src/layout/fieldset.rs` separates the first rendered legend from the
content formatting context, reserves its block size above the content, keeps
specified/intrinsic outer sizing, and handles margins, RTL, flex, and grid.
`layout/mod.rs`, `node/element.rs`, `node/node.rs`, and `node/mod.rs` supply the
transient layout style and decoration metadata. DOM ancestry stays unchanged.

`blitz-paint/src/render/fieldset.rs`, `render.rs`, and `render/border.rs` place
backgrounds/borders below an oversized legend and cut the top border around its
border box. Ordinary borders use their previous path. Vertical writing modes,
fragmentation, and all combinations of fieldsets with columns remain incomplete.

## Progress and meter controls

- `blitz-dom/src/range_control.rs` resolves numeric defaults, clamping, progress
  position, and meter quality bands. `node/element.rs`, `node/mod.rs`, `stylo.rs`,
  `lib.rs`, `layout/construct.rs`, and the UA stylesheet hide fallback children,
  expose indeterminate state, and reserve native replaced-element dimensions.
- `blitz-vibey-script/src/dom/ranges.rs`, `dom/mod.rs`, and `dom/element.rs`
  implement numeric IDL properties, preserving the distinct progress/meter
  assignment rules and rejecting non-finite values.
- `blitz-paint/src/render/range_controls.rs` and `render.rs` paint native tracks,
  fractional fills, accent colors, direction, and meter quality colors. Chromium
  geometry attribution and the existing BSD license are retained. Native edge
  coverage, indeterminate animation, dark controls, and custom pseudo styling
  remain incomplete.

## Canvas images and retained paths

- `blitz-vibey-script/src/canvas_images.rs` snapshots canvas/raster sources and
  draws supplied SVG images through AnyRender. Source clipping, negative sizes,
  self-copy, smoothing, alpha, transforms, and compositing share the CPU pipeline.
  Empty source/destination rectangles do not composite an empty group.
- `canvas_paths.rs`, `canvas.rs`, and `canvas.js` implement Path2D copy/addition,
  SVG path parsing, elliptical roundRect radii, and geometric hit testing.
  Hit tests include boundary points and implicitly close fill subpaths.
  Malformed SVG prefix parsing is not implemented; invalid SVG path strings
  produce an empty path. Geometry and path sizes remain bounded.
- `Cargo.toml` and `lib.rs` register these modules and use the existing pinned
  AnyRender SVG dependency. The host browser still supplies no preview pixels.

## Masked compositing and bilinear image sampling

`vello_cpu/src/fine/lowp/mod.rs` quantizes a masked source consistently with its
inverse alpha before source-over compositing. This prevents translucent curved
edges from reducing an opaque backdrop's alpha to 254. The regression test
checks rounded fills, rounded borders, and transformed translucent boxes.

`vello_cpu/src/fine/lowp/image.rs` uses four-bit fractional sampling weights and
one final eight-bit shift for the complete four-tap bilinear sum. Both general
and axis-aligned sampling use this shared fixed-point reference profile. Image
scaling/cropping cases retain native, WASM, and Chrome comparisons; rotated
image coverage can still differ.

`vello_cpu/src/fine/lowp/compose.rs` and `mod.rs` share the eight-bit source-over
backdrop scale `(256 - sourceAlpha) / 256`, with truncation after multiplication.
This fixes one-level color errors on colored backdrops while retaining opaque
alpha. Regression tests exercise translucent Canvas paint and CSS layers.

## Canvas text and tangent arcs

- `blitz-vibey-script/src/canvas_text.rs`, `canvas.rs`, `canvas.js`, `lib.rs`, and
  `Cargo.toml` shape Canvas text with Parley and measure supplied font bounds
  through Skrifa. `blitz-dom/src/document.rs` exposes the document font context.
  Font state, alignment, direction, baselines, maxWidth, empty strings, and
  preserved whitespace share the engine on native and WASM. Solid fill/stroke
  text is implemented; kerning is added in the later Canvas font kerning patch.
  Gradients, letter/word spacing controls, font-size keywords, and Chrome glyph
  scan-conversion parity remain incomplete.
- `canvas_paths.rs`, `canvas.rs`, and `canvas.js` construct tangent arcs for
  arcTo, including degenerate lines, retained paths, and current points recorded
  under an earlier transform. Negative radii throw IndexSizeError. Curve and
  diagonal-stroke coverage still differ from Chrome.

## Listener options, standalone targets, and cancellation signals

`blitz-vibey-script/src/dom/listeners.rs`, `dom/dispatch.rs`, `dom/event.rs`,
`dom/mod.rs`, `dom/node.rs`, `state.rs`, `runtime.rs`, and `event_targets.js`
share registration and dispatch across nodes, window, and standalone EventTarget
subclasses. Registration identity survives dispatch snapshots: deleting and
re-adding the same callback cannot revive the old entry. Passive listeners do
not cancel default actions. Window load uses the same dispatcher.

AbortController/AbortSignal provide reasons, throwIfAborted, dependent signals,
virtual-time timeouts, and listener cancellation. Dependent cancellation is
independent of stopImmediatePropagation on the public abort event. Native flags
remove aborted registrations before invocation and allow subsequent registration
of the same callback. These APIs do not grant external network access.

## Named module exports at EOF

`boa_parser` is the crates.io `boa_parser` 0.22.0 source package. Its MIT and
Unlicense terms are retained from the Boa v0.22 release. The root Cargo patch
selects it for native and WASM without changing transitive versions.

The only modified upstream source is
`boa_parser/src/parser/statement/declaration/export.rs`: after a named export
list, EOF enters the local-export branch and normal automatic semicolon
insertion. It no longer fails while checking for an optional `from` clause.
Integration tests cover EOF, a trailing newline, and line/block comments.
Invalid module syntax and missing imports still produce diagnostics.


## Event state and supplied-resource fetch cancellation

`blitz-vibey-script/src/dom/event_fields.rs`, `dom/event.rs`, `dom/dispatch.rs`,
`dom/mod.rs`, `events.js`, and `runtime.rs` keep dispatch fields in native slots.
Readonly fields and JavaScript property shadowing cannot redirect native event
routing or cancellation. The composed path is copied from a frozen dispatch
path and cleared afterward. Passive cancellation, cancelBubble, and returnValue
share those native flags. ErrorEvent and legacy window.onerror use the common
dispatcher, including object listeners, once, abort, and stopImmediatePropagation.

`event_targets.js` and `runtime.rs` defer supplied-resource
fetch completion through the microtask queue. Branded signals can reject an
already aborted request, a pending request, or a response body read after abort;
cloned responses retain cancellation. These APIs still cannot read the network
or host filesystem. Streaming fetch bodies remain unsupported.

## Collapsed select controls

`blitz-dom/src/select_control.rs` shapes the selected label and longest option
with supplied fonts. Native single-line selects reserve disclosure space and
use font metrics for intrinsic height and inline baseline. Option label/text,
selectedness, disabled groups, direction, appearance:none, and device scale are
retained in engine state. `lib.rs`, `node/element.rs`, `layout/construct.rs`,
`layout/replaced.rs`, `layout/mod.rs`, `layout/damage.rs`, `resolve.rs`,
`mutator.rs`, and `assets/default.css` construct and invalidate these controls.
`blitz-vibey-script/src/dom/forms.rs` uses the same option selection helpers.

`blitz-paint/src/render/select_control.rs` and `render.rs` paint clipped labels
and disclosure marks. Live value/index/option mutations and form reset repaint
without changing saved source. Native visual styling remains approximate;
listboxes, open popup menus, and keyboard selection are not implemented.

## Individual collapsed table edges

The continuation replaces first-cell border replication and uniform gutters
with per-segment cell conflict resolution. `hidden` suppresses both paint and
used width; otherwise width and style rank select a winner, with upper/inline-start
cells winning equal ties. Transparent winners remain transparent. Each winning
edge resolves `currentColor` from its owner, and spanning cells contribute only
their perimeter. The row-span cursor now keeps occupied columns reserved through
the final covered row. Colspan values are constrained to the HTML range 1–1000.

| File | Continuation change |
| --- | --- |
| `blitz-dom/src/layout/table_borders.rs` | New sparse logical boundary maps, conflict precedence, per-cell and outer used widths |
| `blitz-dom/src/layout/table.rs` | Store spans and resolved edges; reserve half borders in cells/table; account for fixed first-row widths; correct occupied-column lifetime |
| `blitz-dom/src/layout/mod.rs`, `blitz-dom/src/lib.rs` | Register the resolver and expose edge metadata to paint |
| `blitz-paint/src/render/table_borders.rs` | Paint only winning segments, preserve spans and RTL, resolve owner colors, and snap CSS coordinates before device scaling |
| `blitz-paint/src/render.rs` | Paint shared borders after cell backgrounds; remove unused physical-track extent helpers |
| `blitz-paint/src/render/border.rs` | Suppress independent cell border painting while retaining caption borders |

Nine native regressions and twenty Chrome samples cover footer separators,
hidden/absent/transparent edges, owner colors, unequal widths, spans, RTL,
fixed widths, caption borders, JavaScript updates, and DPR. Two older geometry
assertions now use Chrome-confirmed half-border boxes rather than the removed
border-free representation. Patterned borders, complete table/row/column/group
conflicts, complex junction precedence, and vertical table writing modes remain
incomplete. No fixture-specific paint or comparison exceptions are introduced.


## Single-line input metrics and alignment

`blitz-dom/src/layout/input.rs` (new), `layout/mod.rs`, and
`layout/construct.rs` derive intrinsic input height and inline baseline from
shaped font metrics. Author line-height can enlarge the control but cannot
shrink its normal text height. This removes the extra descender gap after an
inline input. Layout supplies the content width to text alignment; single-line
values preserve spaces and remain unwrapped. Direction, centered/right text,
explicit heights, and repeated scale changes have native and Chrome samples.
Default intrinsic input width and platform-specific input appearance remain
approximate.


## Enumerated input types

`blitz-dom/src/node/element.rs` resolves input types case-insensitively and maps
missing/empty/unknown types to text without rewriting the attribute.
`layout/construct.rs`, `layout/mod.rs`, `mutator.rs`, `form.rs`,
`events/pointer.rs`, and `events/keyboard.rs` share the canonical state for
construction, number-input sizing, button labels, checked selectors, activation,
and form submission. `blitz-paint/src/render/form_controls.rs` uses it to choose
radio geometry. `blitz-vibey-script/src/dom/element.rs` reflects the canonical
input.type while retaining raw getAttribute bytes and reflects checkable values
for uppercase attributes. Number spinners, value sanitization, and specialized
date/color/file editors remain incomplete.


## Persistent device scale changes

`blitz-dom/src/document.rs` marks the layout tree damaged and recascades styles
when device scale changes. Rebuilding only text left cached device-space
transforms and overflow bounds at the old scale; fractional border rounding
also depended on the old cascade. Existing script and Canvas state survives the
rebuild. Tests compare repeated 1 / 1.5 / 2 scale changes with fresh native
renders, retain click state, and exercise the WASM SDK render method directly.
The initial failing DPR captures are retained in the campaign artifacts.


## Native checkable baselines and grouped select sizing

The form continuation uses Chrome measurements for native checkbox/radio
border-box baselines, optgroup indentation, and narrow collapsed select labels.

| File | Change |
| --- | --- |
| `blitz-dom/assets/default.css` | Leave checkbox/radio dimensions auto so appearance:none does not retain native intrinsic dimensions |
| `blitz-dom/src/layout/input.rs` | Measure native checkables as 13 CSS pixels and export their border-box bottom as the baseline; unthemed checkables have no intrinsic size or special baseline |
| `blitz-dom/src/layout/mod.rs` | Route both checkbox and radio through shared checkable layout |
| `blitz-dom/src/select_control.rs` | Cache option grouping, reserve four shaped spaces for grouped menu options, use normal weight for grouped intrinsic sizing, and invalidate width after regrouping |
| `blitz-paint/src/render/select_control.rs` | Clip narrow labels to the content edge so text can extend beneath the disclosure arrow in both directions |

Seven native regressions cover margins and line heights, author dimensions,
appearance:none, font sizes/weights, ignored group-label width, live regrouping,
narrow label pixels, and repeated device-scale changes. Twenty-four independent
Chrome fixtures expose remaining control/glyph antialiasing, fractional baseline,
and dynamic option-selection differences in that capture. The following state
continuation fixes the tested reparenting transitions while preserving that
earlier evidence. Popup/listbox interaction and full platform-native styling
remain incomplete.


## Live select state and mutation ordering

Option selectedness is now persistent state, separate from its dirty flag and
serialized selected attribute. Native layout, script properties, and :checked
selectors read the same state. The select reset algorithm runs for relevant
option insertions/removals and default/size/multiple changes, rather than being
inferred when a script reads selectedIndex or the renderer prepares a label.
Explicit selectedIndex/value clearing remains empty until a relevant mutation;
changing text inside an option does not restore a selection.

| File | Change |
| --- | --- |
| `blitz-dom/src/select_state.rs` | New state helpers, owner-scoped option traversal, single/multiple selection normalization, dirty/default handling, and mutation/attribute hooks |
| `blitz-dom/src/lib.rs` | Register the state module |
| `blitz-dom/src/select_control.rs` | Move selection queries to the shared state module; retain label shaping and measurements |
| `blitz-dom/src/node/element.rs` | Track option dirtiness and synchronize selectedness with the CHECKED pseudo-class state |
| `blitz-dom/src/mutator.rs` | Normalize old owners after detaching options, preserve incoming selection across batch insertion, and apply selected/size/multiple attribute updates |
| `blitz-dom/src/control_state.rs` | Reset option selectedness and dirtiness from source defaults |
| `blitz-dom/src/document.rs` | Deep clones initialize options and selects from source defaults rather than copying live selection |
| `blitz-vibey-script/src/dom/forms.rs` | Route selected/value/index properties through shared engine state |
| `blitz-vibey-script/src/dom/node.rs` | Shallow option clones retain fresh source-default state |

An option hidden by a native select or display:none may have no primary style,
so Stylo cannot snapshot it. When CHECKED-dependent rules exist, changing such
an option conservatively recascades an already-styled document root. This keeps
relational selectors such as body:has(option:checked) current. Ordinary option
state writes that do not change CHECKED skip style invalidation.

Twenty-four independently recorded Chrome state traces cover moving/removing
options, regrouping, fragment insertion, explicit clearing, dirty defaults,
size/multiple attributes, disabled options, cloning, cross-select transfer,
reset, and :checked queries. Four native regression tests cover those traces,
transient source preservation, repeated DPR, clone defaults, and ancestor style
invalidation. Popup/listbox drawing, keyboard interaction, options collection
APIs, and the full HTML select algorithm remain incomplete.

### Fractional inline text origins and horizontal glyph phases

- `blitz-paint/src/text.rs`: horizontal glyph origins use four subpixel phases
  per device pixel, after the complete paint transform. Perpendicular baseline
  snapping remains; rotated glyph advances retain their previous behavior.
- `blitz-dom/src/document.rs` and `blitz-dom/src/layout/writing_mode.rs`: expose
  the existing physical unrounded geometry helper to the painter in both
  writing-mode feature configurations. Layout and CSSOM metrics are unchanged.
- `blitz-paint/src/render.rs` and new `blitz-paint/src/render/text_origin.rs`:
  accumulate fractional containing-block positions through ancestor transforms
  and restore fractional padding for ordinary inline text. Box edge snapping,
  clips, and control sizing retain their existing paths. Column fragments
  already use precise text coordinates and bypass this correction.
- `tools/check-glyph-phases.mjs` independently observes the horizontal phase
  classes in Chrome for Inter, Lora, and JetBrains Mono at 12/16/24 px and DPR
  1/1.5/2. Diagnostic mask grouping is confined to that Chrome-only experiment;
  acceptance uses every pixel of each full screenshot without alignment.
- Twenty-four new supplementary samples cover fractional origins, shaped
  advances, foreground colors, and transforms. Native regressions distinguish
  equal and unequal phases and compare nested/scaled/padded text origins with
  equivalent flat placement. General rotated-layer rasterization, glyph outline
  dilation/coverage, and transformed-font metrics remain incomplete.

### Canvas bitmap text origins

- `blitz-dom/src/util.rs`: host the shared `snap_glyph_baseline` paint-origin
  helper previously private to the DOM text painter. Its DOM behavior is
  unchanged, including perpendicular snapping and arbitrary rotation handling.
- `blitz-paint/src/text.rs`: import that shared helper instead of maintaining
  a private copy.
- `blitz-vibey-script/src/canvas_text.rs`: horizontal `fillText` and `strokeText`
  origins use the shared helper after the Canvas bitmap transform, alignment,
  baseline anchor, and maxWidth compression. This quantizes bitmap x to quarter
  pixels and bitmap y to whole pixels. Canvas bitmap resolution is independent
  of the page DPR. Stroke widths, outline widening, and text metrics are not
  changed; nonhorizontal transforms retain their previous positioning.
- `tools/check-canvas-origins.mjs` observes 108 independent Chrome bitmap phase
  profiles across three fonts, three sizes, three transforms, two axes, and
  filled/stroked text. Both operations have the same origin phases; the initial
  assumption that strokes retained continuous positions was rejected by the
  oracle. Diagnostic mask grouping never changes full-image acceptance.
- Added 26 supplementary JavaScript samples and three native tests covering
  bitmap phase identity, transform-independent measurements, and alignment with
  maxWidth. Six direct WASM checks exercise both drawing operations at three
  bitmap scales. Full glyph coverage and rotated rasterization remain partial.

### Raster-size glyph widening

- `blitz-dom/src/util.rs`: compute the existing widening profile at the final
  raster font size along each transformed axis and return local outline units.
  Degenerate/nonfinite axes return zero widening. No profile constants change.
- `blitz-paint/src/text.rs` and `blitz-vibey-script/src/canvas_text.rs`: use that
  shared helper for DOM and filled Canvas text. Stroke outlines retain zero
  widening. DOM DPR and Canvas bitmap transforms use their own raster spaces.
- `glifo/src/glyph.rs`: when uniform scale is absorbed into the drawing font
  size, convert local expansion amounts by the absorbed factor too. Curve
  tolerance remains in final draw units before the existing cache conversion.
- Three rendered native tests and 48 direct WASM comparisons exercise equivalent
  uniform scales, DOM DPR against Canvas raster sizes, and collapsed text. The independent
  Chrome oracle checks 24 Canvas and 24 DOM whole-image equivalences with Lora
  and JetBrains Mono, which have no optical-size axis. Inter remains in the
  36-source conformance corpus but is not assumed to have that invariant.
- The first helper-only implementation and a rejected tolerance-scaling
  experiment remain recorded. Nonuniform and rotated glyph fidelity still
  require independent coverage; no browser-equivalent rasterizer is claimed.

### Canvas font kerning

- `blitz-vibey-script/src/canvas.js`: expose the `fontKerning` enum with `auto`,
  `normal`, and `none`, with string coercion, invalid-value handling, Symbol
  rejection, receiver validation, and an enumerable prototype accessor. The property participates in the shared save/restore,
  reset, bitmap resize, and per-context state mechanisms. Font assignment does
  not reset kerning, and element CSS does not supply this Canvas state.
- `blitz-vibey-script/src/canvas_text.rs`: apply explicit OpenType `kern` zero or
  one in the shared shaping path for measureText, fillText, and strokeText. Auto
  retains the default shaping features; optional ligatures remain enabled.
  Alignment and maxWidth consume the resulting shaped width.
- Added 15 independent state cases and 72 recorded Chrome metric profiles
  spanning three supplied fonts, six sizes, and four strings. The oracle never
  rewrites its expected values. Twenty-seven new JavaScript samples cover state
  and visible drawing, with four native tests and direct WASM checks.
- The original sub-millipixel metric assumption failed for 21 Inter values at
  intermediate optical sizes. Full measurements are retained; behavior tests
  use one CSS layout unit (1/64 px) for numeric metrics. Screenshot acceptance
  stays at 99.9% with every RGBA pixel included. The variable-font advance
  precision gap is not fixed by this change.

### Canvas letter and word spacing

- `blitz-vibey-script/src/canvas.js`: add independent letterSpacing and
  wordSpacing state with CSS dimension validation, canonical getters, string
  coercion, receiver checks, Symbol rejection, and reentrant bitmap resets.
  Shared save/restore, reset, and resize mechanisms preserve or clear spacing.
- `blitz-vibey-script/src/canvas_spacing.rs`: parse one CSS dimension token,
  including comments and escaped units; resolve physical and font-relative
  lengths from supplied fonts. Bound resolved coordinates before shaping.
  Numeric state uses strings so an accepted large spacing assignment does not
  invalidate unrelated drawing operations. Viewport, container, and line-height
  units have zero resolution in this Canvas context. Ideographic units currently
  use the em fallback; supplied CJK glyph advances are not yet consulted.
- `blitz-vibey-script/src/lib.rs` and `src/canvas.rs`: register the spacing module
  and route dimension parsing through the existing Canvas bridge.
- `blitz-vibey-script/Cargo.toml`: declare cssparser 0.38.0 directly, reusing the
  existing dependency version. The workspace Cargo.lock records this edge.
- `blitz-vibey-script/src/canvas_text.rs`: feed both spacings to shared Parley
  shaping for measureText, fillText, and strokeText. Preserve negative advances,
  grapheme boundaries, whitespace handling, and optional-ligature suppression
  for nonzero letter spacing; alignment and maxWidth consume the same width.
  Font-relative probes run before locking the supplied-font context.
- Added 42 supplementary samples, 24 independently checked state cases, 62
  recorded grammar inputs applied to both properties, and 288 Chrome widths
  spanning three fonts, whitespace, ligatures, and combining characters.
  Numeric checks use 0.001 CSS px; full-image acceptance remains 99.9% with
  every RGBA pixel included. Chrome context-dependent leading-space behavior
  and font-relative spacing after font changes remain explicit limitations.

### Reentrant Canvas text assignments

- `blitz-vibey-script/src/canvas.js`: convert font values after receiver and
  Symbol validation, then reacquire current bitmap state before parsing and
  assigning. Reacquire state for valid fontKerning assignments after conversion
  as well. A user-defined toString may resize the canvas or assign another
  font; the outer valid assignment must apply to the resulting current state.
  Exceptions preserve conversion side effects and propagate unchanged.
- Added 16 independent Chrome cases and matching native, WASM, and HTML samples
  for resize during conversion, thrown values, nested assignments, Symbol
  rejection, receiver validation order, and resulting text widths. Five cases
  fail on the previous engine and are retained as before/after evidence.

### Canvas text method argument conversion

- `blitz-vibey-script/src/canvas.js`: validate required argument counts and
  DOMString Symbol rejection for measureText, fillText, and strokeText. Convert
  drawing coordinates with WebIDL ToNumber semantics, rejecting primitive and
  boxed BigInt values. Conversion stays left-to-right, even when a preceding
  coordinate is nonfinite. Reacquire bitmap state after conversion callbacks.
  Preserve omitted/undefined maxWidth and ignore extra arguments; expose the
  expected method lengths (one for measurement, three for drawing).
- Added 29 independently verified Chrome state cases, six visible reset-during-
  argument-conversion samples, native/WASM regressions, and a Worker interaction
  test. Fourteen initial state failures are retained in before/after evidence.

### Canvas rectangle and basic transform arguments

- `blitz-vibey-script/src/canvas.js`: share required-count, receiver, ordered
  ToNumber, and finite-value validation across fillRect, strokeRect, clearRect,
  translate, scale, rotate, and transform. BigInt coordinates now throw, including
  boxed values. Drawing and matrix composition use state after argument callbacks
  have run, so bitmap resets cannot leave stale paint or matrices in use.
- Basic transforms now compose internally rather than dispatch through the
  script-overridable context.transform property. The six-argument transform
  method exposes length six and ignores extra arguments without converting them.
- Added 47 Chrome-verified state cases, seven visible rectangle/transform sources,
  native and WASM regressions, and a Worker click/source-save integration test.
  The initial native test reproduces 27 failures. Other Canvas methods and
  setTransform dictionary/overload semantics remain separate work.

## Canvas numeric state and dash sequence conversion

`vendor/blitz-vibey-script/src/canvas.js` validates receivers before converting
`globalAlpha`, `lineWidth`, `miterLimit`, and `lineDashOffset`. Unary ToNumber
rejects primitive and boxed BigInt while retaining finite/range handling and
state changes made by conversion callbacks.

`setLineDash` requires an iterable object, caches the iterator's next method,
and converts each yielded element before requesting another. Like Chrome's
WebIDL sequence conversion, it does not call iterator.return on conversion
failure. Invalid sequences leave the dash state intact; valid odd sequences
are duplicated. Assignment observes bitmap resets and nested state changes.
Iterator invocation uses Reflect.apply, independently of a callable's own
call property. Other Canvas overloads and property conversions remain partial.

Regression coverage lives in `conformance/canvas-state-cases.json`,
`conformance/samples-canvas-state.mjs`, the native
`javascript_canvas_state.rs` test, `tools/check-canvas-state.mjs`, and the direct
WASM JavaScript checks. The embedded host test exercises conversion during an
initial render and a preview click, bitmap reset, and source-only persistence.

## The page's fonts for SVG text, and inline edges (2026-11-09)

SVG `<text>` drew with a process-global usvg font database that loaded the machine's fonts, so
renders differed between machines and `font-family` named only system faces
(kition-html#2). `blitz-dom/src/config.rs` adds `DocumentConfig::svg_fonts`; `document.rs` keeps
it per document (an empty database without one, never the machine's) and `iframe.rs` passes it
to sub-documents; `util.rs` drops the global and parses with the document's database, which
`net.rs` (`ImageHandler`), `mutator.rs`, `layout/damage.rs` and `layout/construct.rs` (inline
`<svg>`) hand it. The engine builds the database from the page's fonts and the host's, with the
generic families set to the faces that serve them in HTML text.

Horizontal margin, border and padding of non-replaced inline elements (and of `::before` and
`::after`) took no room on the line (kition-html#3). `layout/inline/build.rs` adds them to the
layout text as word joiners widened by letter-spacing: the margin outside the element's style
span, the border and padding inside it, so the inline background covers the padding. A border of
style none or hidden is 0 wide; percentages count as 0.
