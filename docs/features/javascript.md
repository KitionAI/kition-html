# JavaScript preview

Inline and supplied external classic scripts, ES modules, DOM/CSSOM mutations,
Promise jobs, load events, click listeners, timers, and requestAnimationFrame run
in Boa. The editor forwards clicks and advances virtual time in 100 ms steps.
Native PNG capture includes initial scripts and the first 16 ms of timers. The
SDK exposes non-document-mutating `html.click` and `html.advance` commands; DOM
state persists across frames and zoom changes. Saved bytes remain the HTML source.
Changing source, resources, or viewport starts a fresh script session.

Scripts, module imports, and GET/HEAD fetches can read only supplied `https://kit.invalid`
resources and data URLs. They cannot read local files or make host network requests.
Resource text must be UTF-8 and at most 1 MiB. Each preview session has a 5,000,000
VM instruction budget, a 100,000 loop-iteration limit, and a 256 recursion limit.
Each advance runs at most 128 timer batches; exceeding that pauses timers and
reports a diagnostic. These bounds prevent tested runaway loops and microtasks;
they are not a complete memory or execution-time sandbox for every built-in.

`nomodule` scripts and non-JavaScript data blocks are skipped. Deferred external
scripts and modules follow ordinary scripts. Parsing completes before any script
runs; parser-blocking `document.write`, browser networking, Shadow DOM,
and full framework compatibility are not provided. Keyboard/text input inside
rendered pages is not yet wired into the editor. Unsupported APIs and uncaught
exceptions appear in rendering diagnostics.

After building the WASM engine, run `pnpm test:javascript:wasm` for its script,
event, timer, module, resource-isolation, and execution-limit regressions.

## Additional page APIs

Events support capture, bubbling, cancelation, once/passive listeners, inline handlers,
MouseEvent/CustomEvent, standalone EventTarget subclasses, and AbortSignal listener
removal. AbortController supports reasons, throwIfAborted, any, and virtual-time
timeouts. Dispatch fields are readonly, composedPath retains the dispatch path,
and ErrorEvent supports legacy window.onerror. Supplied-resource fetches and
response body reads honor abort signals. Live DOM collections track subsequent mutations. Form properties retain live
values separately from default HTML; checkbox/radio/label activation and form
reset follow cancelable events. Disabled fieldsets preserve the first legend's
exception. Import maps resolve only supplied resources, including scoped and
prefix mappings. matchMedia uses the preview viewport's Stylo device.

Canvas 2D supports paths, tangent arcs, curves, rounded rectangles, transforms, clipping,
solid/gradient paints, line styles, compositing, and ImageData read/write in the
engine. `drawImage` accepts canvas snapshots and supplied raster/SVG images,
including cropping, scaling, and image smoothing. Path2D supports retained paths,
valid SVG path strings, copying, transformed addition, and fill/stroke hit tests.
Its bitmap persists across preview frames and resets on width/height assignment.
Text supports fillText, strokeText, measureText, supplied fonts, alignment,
direction, baselines, maxWidth, fontKerning, letterSpacing, and wordSpacing.
Text paints currently require solid colors;
glyph rasterization still differs from Chrome. Patterns, shadows, filters,
and canvas export remain unavailable. Canvas
has a 16-megapixel bitmap/readback limit, an 8192-pixel per-axis limit, and bounded
path/clip/state stacks. Unsupported method calls and exceeded limits produce
diagnostics; assigning unsupported drawing properties may have no visual effect.
This is not complete browser compatibility.

Collapsed selects render selected labels, follow live option/value changes and
form reset, and share intrinsic metrics with native rendering. Single-line
inputs retain font baselines, line-height minima, text alignment, and spaces.
Input types are case-insensitive with the standard text fallback; specialized
number/date/color controls remain partial. Select popups and listboxes are not
implemented. Progress and meter elements resolve numeric constraints and render
native tracks. Fieldsets reserve space for their first rendered legend and leave the
matching border gap. Native control antialiasing, indeterminate animation, custom
control pseudo-elements, and vertical fieldsets remain incomplete.

Supplementary samples are generated with `node tools/generate-samples.mjs` and
captured with `node tools/compare.mjs --manifest samples/manifest.json --run
<new-run-id> --measure-only`. Compare the same run with `node
tools/compare-wasm.mjs <new-run-id>`. The original corpus and the supplementary
corpus have separate counts and reports.
