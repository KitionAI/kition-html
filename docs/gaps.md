# Gaps

> **Last reviewed:** 2026-11-16 · **Target:** Chrome (see [target-app-parity.md](parity/target-app-parity.md))

Known shortfalls, one entry each, from the [README](../README.md), [findings.md](parity/findings.md) and
the kit's feature catalogue. Update the entry in the same change that closes or changes it.

## G1 Pixel fidelity against Chrome

52 of the 128 original cases do not reach 99.9% content similarity against Chrome (76 pass), and
39 of the 590 supplementary JavaScript samples do not (551 pass). [findings.md](parity/findings.md) has
the results; `pnpm compare` writes the per-case diagnostics.

## G2 Form controls

Specialized number, date and colour input controls are partial; select popups and listboxes are
not implemented; native control antialiasing, indeterminate animation, custom control
pseudo-elements and vertical fieldsets are incomplete.

## G3 Browser APIs

The JavaScript environment (Boa against the kit's DOM) implements part of the browser APIs; pages
that need the rest do not run as in a browser.

## G4 Rendering has no feature entries

[features.yaml](../features.yaml) catalogues the agent features and the page's replica
(`html.collab.replica`). Rendering, scripting and the editor have no feature entries, statuses or gating tests of their own; the corpora measure them.
