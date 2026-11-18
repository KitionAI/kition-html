# Native rendering

```sh
export PATH="$(dirname "$(rustup which --toolchain 1.95.0 rustc)"):$PATH"
cargo build --locked -p kition-html
target/debug/kition-html render conformance/cases/grid-fr-tracks.html \
  --assets conformance/assets --width 640 --height 480 --output preview.png
```

The output must be a new file. The CLI and SDK share a bundled Inter default
font, so plain HTML text renders without an asset directory. A host adds more faces for every page
with `add_font` (`addFont` on the web): the CSS generic families take them by their family names
(serif, monospace, emoji, and a Chinese, Japanese or Korean face for the characters the others
lack). SVG text uses the same faces. Its [OFL license](../../conformance/assets/Inter-OFL.txt)
must accompany redistributed binaries/packages; the WASM build copies it into
`engine/`; vendored source licenses are copied into `engine/licenses/`. Supplied resources can replace the default font. The CLI reads only the supplied HTML and asset
directory. Asset files are exposed under `/assets/`; the engine itself reads
neither the filesystem nor the network. Missing resources and unsupported
filters are reported in the command's JSON result. PNG output is available to
terminal tools, agents, image viewers, and the WASM SDK's `renderPng()` method.

Preview the same raster directly in a terminal:

```sh
target/debug/kition-html preview conformance/cases/grid-fr-tracks.html \
  --assets conformance/assets --columns 100 --max-rows 40
cat page.html | target/debug/kition-html render - --output page.png
```

Preview uses averaged pixels and true-color half-blocks, preserving the image
aspect ratio at a two-pixels-per-cell geometry. Piped output and `NO_COLOR` use
ASCII luminance by default; `--color always` and `--color never` override this.
`--width`, `--height`, `--scale`, `--dark`, and `--assets` work for both commands.
The HTML input is UTF-8 and limited to 16 MiB, including standard input. Preview
diagnostics go to standard error, keeping the image stream on standard output.
