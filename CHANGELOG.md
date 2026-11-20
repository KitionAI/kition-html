# Changelog

Changes to Kition HTML, newest first. A version is the kit package version in `kit.json`;
each package is published with a `package-v<version>` tag.

## 0.1.1

- `kit.json` declares the engine's editor state (`engine.editorState`), so a host that does not run the engine's Rust code reads `inspect()` without it.
- The package is built and published on GitHub with Kition's `kit-package.yml`.
- README: an agent connects with `npm install -g @kition/cli` and `kition agent add`.

## 0.1.0

First public release of the source, under the Apache License 2.0. It contains:

- The Rust renderer (`kition-html-engine`): HTML and CSS laid out with Blitz and drawn with Vello
  CPU, with no browser, GPU, network or operating-system fonts at runtime. JavaScript runs in Boa
  against the same DOM, with events, timers, supplied modules and a Canvas 2D subset. Text draws
  in the faces the page and the host supply (generic families, Chinese, emoji, SVG text).
- The kit engine (`kition-html-kit`), native and WebAssembly: the page is a KDM replica, so the
  source round-trips byte for byte, replicas merge concurrent edits, and undo is per person and
  per agent. `html.setSource` writes the whole source as one undo step; a check catalogue reports
  what a page cannot carry.
- The headless kit (`kition-html`) for agents over the Kit Protocol. It reads nothing outside its
  workspace and connects to nothing.
- The kit's editor (`web/`): the source and the rendered preview, where you can click elements and
  advance time, in every display mode, in kit-editor's frame.
- The Chrome conformance corpus (`conformance/`): 128 feature fixtures and an 843-case
  supplementary corpus, accepted at 99.9% content similarity per case
  ([rendering findings](docs/parity/findings.md)).
- NOTICE and docs/attribution.md list the modified vendored copies (Blitz, Boa's parser, Glifo,
  Parley, Stylo, Vello CPU) and Loro (MIT), bundled by the editor and linked by the engine.
