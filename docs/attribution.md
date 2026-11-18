# Attribution

Third-party code and non-code assets in this repository, with their source and license. Kition
HTML's own code is under the Apache License 2.0 ([LICENSE](../LICENSE)); [NOTICE](../NOTICE) names the
projects it includes. The kit package also carries its Rust dependencies' licenses in
`engine/licenses/` and the bundled Inter font's license in `fonts/` (written when the package is
built; not in this repository).

## Vendored code

Modified copies of upstream crates, kept under `vendor/` with their original license files. What
changed in each is listed in [vendor/README.md](../vendor/README.md).

| Path | Upstream | Version | License | License files |
|---|---|---|---|---|
| `vendor/blitz-dom/` | [Blitz](https://github.com/dioxuslabs/blitz), revision `21edc92d90d8f8bedadd3f9ecb14b1ad950952eb` | 0.3.0-beta.4 | MIT OR Apache-2.0, except `assets/default.css` (MPL-2.0, under Assets) | `LICENSE-MIT`, `LICENSE-APACHE`; `LICENSE.chromium` (BSD-style, The Chromium Authors) for code derived from Chromium |
| `vendor/blitz-html/` | Blitz, same revision | 0.3.0-beta.4 | Apache-2.0 OR MIT | `LICENSE-MIT`, `LICENSE-APACHE` |
| `vendor/blitz-paint/` | Blitz, same revision | 0.3.0-beta.4 | MIT OR Apache-2.0 | `LICENSE-MIT`, `LICENSE-APACHE`; `LICENSE.chromium` (The Chromium Authors), `LICENSE.apple-color` (BSD-style, Apple Inc.), `LICENSE.google-stroke` (BSD-style, Google Inc.) for code derived from those sources |
| `vendor/blitz-vibey-script/` | Blitz, same revision | 0.3.0-beta.4 | Apache-2.0 OR MIT | `LICENSE-MIT`, `LICENSE-APACHE` |
| `vendor/boa_parser/` | [Boa](https://github.com/boa-dev/boa) | 0.22.0 | Unlicense OR MIT | `LICENSE-MIT`, `LICENSE-UNLICENSE` |
| `vendor/glifo/` | [Vello](https://github.com/linebender/vello) | 0.4.0 | Apache-2.0 OR MIT | `LICENSE-MIT`, `LICENSE-APACHE` |
| `vendor/parley/` | [Parley](https://github.com/linebender/parley) | 0.12.0 | Apache-2.0 OR MIT | `LICENSE-MIT`, `LICENSE-APACHE` |
| `vendor/stylo/` | [Stylo](https://github.com/servo/stylo) | 0.22.0 | MPL-2.0 | `LICENSE-MPL-2.0` |
| `vendor/vello_cpu/` | [Vello](https://github.com/linebender/vello) | 0.3.0 | Apache-2.0 OR MIT | `LICENSE-MIT`, `LICENSE-APACHE`; `LICENSE.skia` (BSD-style, Google Inc.) for code derived from Skia |

## Assets

| Path | Author | Source | License | Notes |
|---|---|---|---|---|
| `vendor/blitz-dom/assets/default.css` | Mozilla contributors (the file keeps Mozilla's MPL-2.0 header, the XUL namespace and Mozilla bug references); Blitz contributors; modified by Kition HTML contributors | Blitz, same revision | MPL-2.0, per the file's header (not Blitz's MIT OR Apache-2.0); license text in `vendor/stylo/LICENSE-MPL-2.0` | The user-agent style sheet, compiled into the engine (`DEFAULT_CSS` in `vendor/blitz-dom/src/lib.rs`); changes in vendor/README.md |
| `vendor/blitz-dom/assets/moz-bullet-font.ttf` | Mozilla (designer Mats Palmgren) | Mozilla Bullet 1.1 from [Mozilla Gecko](https://hg.mozilla.org/mozilla-central/rev/7d5056c95c12) (`layout/style/res/Mozilla_Bullet.ttf`, Bug 1903922), a byte copy; `default.css` embeds the same font (`Mozilla_Bullet.woff2`) in its `-moz-bullet-font` `@font-face`, as Gecko's `html.css` does | SIL Open Font License 1.1, named in the font's metadata | `BULLET_FONT` in `vendor/blitz-dom/src/lib.rs`; Kition HTML paints list markers as shapes and does not draw with it; see `vendor/blitz-dom/assets/moz-bullet-font.LICENSE.md` |
| `conformance/assets/Inter-Variable.ttf` | The Inter Project Authors | [Inter](https://github.com/rsms/inter) | SIL Open Font License 1.1 | `conformance/assets/Inter-OFL.txt` |
| `conformance/assets/JetBrainsMono-Variable.ttf` | The JetBrains Mono Project Authors | [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) | SIL Open Font License 1.1 | `conformance/assets/JetBrainsMono-OFL.txt` |
| `conformance/assets/Lora-Variable.ttf` | The Lora Project Authors | [Lora](https://github.com/cyrealtype/Lora-Cyrillic) | SIL Open Font License 1.1 | `conformance/assets/Lora-OFL.txt` |
| `conformance/assets/` (other files: `external.css`, `imported.css`, `sample-canvas.png`, `sample-canvas.svg`, `sample-data.json`, `sample-export*.js`, `sample-palette.js`, `shape.svg`) | Kition HTML contributors | written for the corpus | Apache-2.0 | Inputs the corpus pages load |
| `conformance/cases/`, `conformance/samples/`, `conformance/basic.html` | Kition HTML contributors | written for this repository (`tools/generate-cases.mjs`, `tools/generate-samples.mjs`) | Apache-2.0 | The conformance and supplementary corpora |
| `conformance/golden/` | Kition HTML contributors | rendered by this repository's tools | Apache-2.0 | Reference images and data for the golden cases |
| `cases/**/input/` | Kition HTML contributors | written for the cases | Apache-2.0 | Case inputs |
