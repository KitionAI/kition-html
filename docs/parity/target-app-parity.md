# Parity with Chrome

> **Last reviewed:** 2026-11-16 · **Target:** Chrome (Chrome for Testing 147.0.7727.15 on macOS ARM64, the oracle in [findings.md](findings.md))

Kition HTML renders a page with its own Rust engine (Blitz, Stylo, Parley and Vello CPU; see
[vendor/README.md](../../vendor/README.md)) and runs its scripts in Boa. Chrome is the reference it is
compared with, pixel by pixel; Chrome is used only for testing.

## Measured

From the [development guide](../development.md#rendering-measurements) and [findings.md](findings.md),
accepting a case at 99.9% content similarity:

| Corpus | Cases | Pass |
|---|---:|---:|
| Original corpus, native against WASM | 128 | 128 |
| Original corpus, native against Chrome | 128 | 76 |
| Supplementary corpus, JavaScript samples against Chrome | 590 of 843 | 551 |

## Catalogue

[features.yaml](../../features.yaml) has ten features for version 0.1.1 (`kit.json`). Nine are
`has`: the four agent features (observe, act, recover, P0; converse, P1), the page as a KDM
replica, editing the page's HTML with undo, save and reopen (P0), the display modes, the faces
text draws in, and inline elements' margins, borders and paddings. One is `partial`: clicking and
advancing time on the rendered page. Beyond those two rendering entries, rendering's coverage is
the corpora above, described in [conformance/README.md](../../conformance/README.md).

## Known differences

Recorded in [javascript.md](../features/javascript.md): specialized number, date and colour input controls are
partial; select popups and listboxes are not implemented; native control antialiasing,
indeterminate animation, custom control pseudo-elements and vertical fieldsets are incomplete.
Browser APIs are incomplete.
The ranked list is [gaps.md](../gaps.md).
