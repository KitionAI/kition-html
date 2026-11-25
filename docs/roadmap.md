# Roadmap

> **Last reviewed:** 2026-11-16 · **Target:** Chrome (see [target-app-parity.md](parity/target-app-parity.md))

## Where it stands

- Version 0.1.1 (`kit.json`), KHP level `basic`, engine contract 2: the page is a KDM replica, so
  replicas exchange changes in order and undo is per person and per agent. One command edits the
  page: `html.setSource` writes the source as one undo step ([writing-format.md](writing-format.md)).
- Ten catalogued features in [features.yaml](../features.yaml): nine `has`, and one `partial`
  (clicking and advancing time on the rendered page); the rendering corpora are measured in
  [target-app-parity.md](parity/target-app-parity.md).

## Next

1. Close the pixel differences against Chrome, using the per-case diagnostics `pnpm compare` writes ([findings.md](parity/findings.md)) (gap G1).
2. Finish the form controls (G2) and the browser APIs pages depend on (G3).
3. Catalogue rendering, scripting and the editor as features with gating tests (G4).
