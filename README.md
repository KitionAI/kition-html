<div align="center">

# Kition HTML

**Kition HTML: an editor and live preview for HTML pages, for people and AI agents**

Source and live preview · A check for what won't load · Undo per person · Focus mode · MCP tools for agents

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-5645d4)](LICENSE)
[![npm: @kition/embed](https://img.shields.io/npm/v/@kition/embed?label=%40kition%2Fembed&color=5645d4)](https://www.npmjs.com/package/@kition/embed)
[![KHP 1.0](https://img.shields.io/badge/KHP-1.0-5645d4)](https://github.com/kitharness/khp)
[![Engine: Rust](https://img.shields.io/badge/engine-Rust%20%2B%20WebAssembly-5645d4)](crates/kition-html-kit)

[Use it in Kition](https://kition.ai) · [Embed it](#embed-it-in-your-app) · [Give it to your agent](#give-it-to-your-agent) · [Showcase](https://github.com/KitionAI/kition-showcase/tree/main/html) · [Developer docs](https://developer.kition.ai) · [KHP](https://github.com/kitharness/khp)

</div>

An HTML page is text, but you rarely get to see the text and the page at once. A page builder
hides the source. A text editor shows only the source. An AI agent can write HTML, but it can't
tell whether the page it wrote draws, or whether it asks for a file that isn't there.

Kition HTML puts the page's source beside a live preview, drawn by the kit's own Rust engine. You
type in the source, and the preview and the check follow. An AI agent edits the same file with the
same command, runs the same check, and fixes what it reports. When it edits a page you have open,
you see the change arrive.

<img src="assets/readme/hero.png" alt="The HTML editor on a bakery's newsletter: the page's source on the left and its rendered preview on the right." width="100%" />

## What it does

### Why HTML

- **The source and the page, side by side.** Type in the source and the preview redraws. The file
  is saved as the text you wrote, nothing more.
- **A check that says what is missing.** It names each file the page links that the kit can't
  load, and each script error with where it happened.
- **Built for agents.** An agent reads the source, writes the new page in one step, checks it and
  fixes what the check found. Its edits to a page you have open show up live.
- **Undo that knows who did what.** Each change is one undo step, and your undo takes back your
  changes, not the agent's.
- **Nothing leaves the page.** The preview fetches nothing from the network or the disk, so a page
  can't reach out while you work on it.
- **Its own renderer, open source.** The preview is drawn by a Rust engine compiled to WebAssembly
  and run in a Web Worker. Apache-2.0, engine and editor.

### See it

<table>
<tr>
<td width="50%"><img src="assets/readme/see-1.png" alt="A new headline typed into the newsletter's source, and the preview showing it in the header." /><br /><b>Edit the source</b>: type, and the preview redraws the page.</td>
<td width="50%"><img src="assets/readme/see-2.png" alt="The newsletter's header turned green by an agent's edit, with the new color selected in the source and Edited by the agent in the status bar." /><br /><b>Agent edits, live</b>: an agent changes the open page, and the editor says so.</td>
</tr>
<tr>
<td width="50%"><img src="assets/readme/see-3.png" alt="An image tag for images/rye-loaf.jpg added to the source, and the diagnostic under the preview saying the kit cannot load it." /><br /><b>The check</b>: what the page asks for and can't get is named under the preview.</td>
<td width="50%"><img src="assets/readme/see-4.png" alt="Focus mode: the source and the preview fill the window, with a small bar to leave it." /><br /><b>Focus</b>: only the source and the preview.</td>
</tr>
</table>

### Made with HTML

<table>
<tr>
<td width="25%"><a href="https://github.com/KitionAI/kition-showcase/tree/main/html/coffee-roaster-landing-hero"><img src="assets/readme/made-1.png" alt="The first screen of an invented coffee roaster's site, with a gradient hero, two buttons and a product card." /></a></td>
<td width="25%"><a href="https://github.com/KitionAI/kition-showcase/tree/main/html/bistro-dinner-menu"><img src="assets/readme/made-2.png" alt="A bistro's dinner menu in two columns: starters, sharing plates, mains and desserts with prices." /></a></td>
<td width="25%"><a href="https://github.com/KitionAI/kition-showcase/tree/main/html/travel-magazine-grid"><img src="assets/readme/made-3.png" alt="A travel magazine's front page with a cover story, a list of short reads and two section cards." /></a></td>
<td width="25%"><a href="https://github.com/KitionAI/kition-showcase/tree/main/html/team-plan-pricing-cards"><img src="assets/readme/made-4.png" alt="Three pricing plans side by side, the middle one marked most popular, with a monthly and yearly switch." /></a></td>
</tr>
</table>

Every sample is built with HTML's own commands. [See all of them](https://github.com/KitionAI/kition-showcase/tree/main/html).

### Formats

| Opens | Creates | Exports |
| --- | --- | --- |
| `.html`, `.htm` | `.html` | — |

A page is its source text. Editing means changing that text: in the editor you type in the source,
and an agent replaces the whole source with the `html.setSource` command, one undo step each time.
The file is saved as that text.

## Quick start

### Use it in Kition

Open a `.html` or `.htm` file in Kition and it opens in this editor. Kition's agent edits the same
file with the kit's `html_*` tools. [Download Kition](https://github.com/KitionAI/kition/releases/latest).

### Embed it in your app

HTML's editor is a static package you host yourself, mounted into your page with the Kition SDK's
[`@kition/embed`](https://www.npmjs.com/package/@kition/embed). You keep the files and the users;
the editor reads and writes through the functions you give it.

1. Download `kition-html-<version>.tgz` from the
   [latest release](https://github.com/KitionAI/kition-html/releases/latest) and serve its
   `html-ui/` folder as static files on its own origin, with the headers in the
   [embed reference](https://github.com/KitionAI/kition-sdk/blob/main/docs/platform/reference/embed.md).
2. Install the SDK and mount the editor:

```sh
npm install @kition/embed
```

```ts
import { KitionEmbed } from '@kition/embed'

const editor = await KitionEmbed.mount(document.getElementById('page')!, {
  kit: { id: 'kit.html', url: 'https://kits.example.com/html-ui/' }, // the package you host
  file: { id: 'newsletter', name: 'Newsletter.html' },
  storage: {
    read: async (id) => api.read(id),                    // { bytes, version }
    write: async (id, bytes, { expectedVersion }) =>     // refuse a stale version
      api.write(id, bytes, expectedVersion),             // { version }
  },
})
```

Every save carries the version it read, so an edit made elsewhere is never written over. The
[embed guide](https://github.com/KitionAI/kition-sdk/blob/main/docs/sdk/embed.md) covers identity,
checks and the rest.

### Give it to your agent

The engine also runs headless as the `kition-html` executable, and the SDK's `kition-kits-mcp`
turns it into MCP tools (`html_inspect`, `html_run`, `html_check`, ...) on a folder of pages, for
any MCP client:

```sh
cargo build --release -p kition-html
cargo install --git https://github.com/KitionAI/kition-sdk kition-kits --bin kition-kits-mcp
kition-kits-mcp --workspace <folder of pages> --kit target/release/kition-html
```

A real session on a copy of the newsletter sample, abridged. The agent changes the headline and
adds a picture, the check finds that the picture can't load, and the agent draws one instead:

```text
> html_inspect {"path":"newsletter.html"}
Opened `newsletter.html`: 640 × 480
{…,"details":{…,"source":"<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\">
  <title>Crumb &amp; Hearth: the November loaf letter</title></head>…"},…,"revision":"vpu0",…}

> html_run {"path":"newsletter.html","basis":"vpu0","steps":[{"command":"html.setSource","params":{"source":
    "<!doctype html>…Rye season is back: book your Saturday loaf…
     <img src=\"images/rye-loaf.jpg\" alt=\"A rye loaf\" width=\"536\"><br>Hello neighbour,…"}}]}
Ran 1 commands on `newsletter.html` (saved): 640 × 480 Changes: objects 1 changed; 1 new diagnostic(s), 0 resolved.

> html_check {"path":"newsletter.html"}
Checked `newsletter.html`: 1 error, no warnings, no notes.
{"diagnostics":[{"code":"html.resource-unavailable","message":"The page asks for `images/rye-loaf.jpg`,
  which the kit cannot load, so it is missing.","severity":"error","target":{}}]}

> html_run {"path":"newsletter.html","basis":"vpu1","steps":[{"command":"html.setSource","params":{"source":
    "<!doctype html>…<svg width=\"536\" height=\"120\" role=\"img\" aria-label=\"A rye loaf\">…</svg>
     <br>Hello neighbour,…"}}]}
Ran 1 commands on `newsletter.html` (saved): 640 × 480 Changes: objects 1 changed; 0 new diagnostic(s), 1 resolved.

> html_check {"path":"newsletter.html"}
Checked `newsletter.html`: the kit found nothing wrong.
```

Every write names the revision it read (`basis`), and each answer says what changed and which
diagnostics are new or resolved. The agent's guide is [agent/guide.md](agent/guide.md).

## Maturity

Preview: early, expect gaps. Version 0.1.0 · KHP 1.0, conformance level `basic`.

| Works today | Partly there | Not yet |
| --- | --- | --- |
| Editing the page's source, with undo and redo | Clicking the preview and moving its clock to run the page's scripts | Loading linked images, style sheets and scripts: write them into the page |
| A live preview drawn by the kit's engine | Looking exactly as in a browser: 76 of 128 test pages match | Choosing the preview's page size: pages are laid out 640 pixels wide |
| A check for missing files and script errors | The browser APIs scripts can use | Editing on the preview itself: you edit the source |
| Agents that edit, check and fix a page | Form controls: date, number and color inputs, select lists | Exporting to other formats |

Known gaps: [docs/gaps.md](docs/gaps.md). What's next: [docs/roadmap.md](docs/roadmap.md).

## Build and contribute

Build and test instructions are in [docs/development.md](docs/development.md). Read
[CONTRIBUTING.md](.github/CONTRIBUTING.md) before opening a pull request, and ask questions in
[Discussions](https://github.com/KitionAI/kition/discussions) or on [Discord](https://discord.gg/bFSjRDjVY).

## License

Apache-2.0 ([LICENSE](LICENSE)). Third-party licenses and notices are listed in [NOTICE](NOTICE),
which names the included projects; [docs/attribution.md](docs/attribution.md) lists the vendored
code and assets with their licenses.
