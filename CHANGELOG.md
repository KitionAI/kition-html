# Changelog

Changes to Kition HTML, newest first. A version is the kit package version in `kit.json`;
each package is published with a `package-v<version>` tag.

## Unreleased

- The engine runs on kition-sdk's plain document model (no CRDT, kition-sdk 91cf510): Kition Desktop is for one person, so pages are no longer merged from concurrent edits; replicas exchange changes in order. A file changed outside Kition opens on the stored replica with what the file holds now written as one import change. Loro is no longer linked.

## 0.1.3

- The same kit as 0.1.2, built on kition-sdk 6672663 (the replica's undo passes over an undo group whose changes came to nothing): the 0.1.2 package was built on kition-sdk 6b4d471.

## 0.1.2

- A host asks an HTML file for its document id (`document.identity`) and forks it (`document.fork`), Kit Process Protocol section 5.6 (kition-sdk 6b4d471): a saved page is its HTML source alone and embeds no replica, so it carries no id and its fork is the same bytes.
- The headless program speaks Kit Process Protocol version 1 (`headless.protocol` is `kition-kit/1`), as hosts on kition-sdk from bb1b07b require.
- kition-sdk 6672663: KDM undo and redo pass over a group whose changes came to nothing, so an undo step that changed nothing no longer stops undo.

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
