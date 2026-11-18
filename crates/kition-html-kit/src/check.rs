//! What [`KitDocument::check`](kition_kit::KitDocument::check) reports on a page (KHP A21): what
//! the renderer met while laying out and drawing it: resources it could not load, script errors,
//! and filters the CPU backend does not draw. A page the engine cannot render at all, or whose
//! layout does not settle, is an error of the render call, not a diagnostic: no document within the
//! kit's limits reaches either, so the catalogue holds only what a page can carry.
//!
//! Codes: `html.resource-unavailable`, `html.javascript-error`, `html.unsupported-filter`,
//! `html.unsupported-backdrop-filter`.

use kition_kit::diagnostics::{Diagnostic, Severity};
use serde_json::json;

/// One code the check reports: its severity and a one-sentence summary for the catalogue.
pub struct Entry {
    pub code: &'static str,
    pub severity: Severity,
    pub summary: &'static str,
}

const fn entry(code: &'static str, severity: Severity, summary: &'static str) -> Entry {
    Entry {
        code,
        severity,
        summary,
    }
}

/// Every code [`diagnostics`] reports, errors first: the catalogue shipped as `agent/checks.json`
/// (KHP A38), with one conformance fixture per code (`conformance/check-<name>.html`).
pub const CATALOGUE: &[Entry] = &[
    entry(
        "html.resource-unavailable",
        Severity::Error,
        "The page asks for an image, stylesheet or other file the kit cannot load, so it is missing.",
    ),
    entry(
        "html.javascript-error",
        Severity::Error,
        "A script on the page threw an error, so what it should have done did not happen.",
    ),
    entry(
        "html.unsupported-filter",
        Severity::Warning,
        "An element uses a CSS filter the renderer does not draw, so it shows unfiltered.",
    ),
    entry(
        "html.unsupported-backdrop-filter",
        Severity::Warning,
        "An element uses a backdrop filter the renderer does not draw, so the backdrop shows unfiltered.",
    ),
];

/// The catalogue as `agent/checks.json` holds it: code, severity, summary, and its fixture.
pub fn catalogue_json() -> serde_json::Value {
    serde_json::Value::Array(
        CATALOGUE
            .iter()
            .map(|entry| {
                json!({
                    "code": entry.code,
                    "severity": entry.severity,
                    "summary": entry.summary,
                    "fixture": format!("check-{}.html", entry.code.trim_start_matches("html.")),
                })
            })
            .collect(),
    )
}

/// Where the renderer places the page (`kition_html_engine`'s base URL): a resource's URL under it
/// is shown as the page wrote it, relative to the page.
const BASE: &str = "https://kit.invalid/cases/";

fn sentence(text: &str) -> String {
    let first = text
        .lines()
        .next()
        .unwrap_or(text)
        .trim()
        .trim_end_matches('.');
    format!("{first}.")
}

/// The renderer's messages (`<code>: <detail>`) as diagnostics, errors first.
pub fn diagnostics(messages: Vec<String>) -> Vec<Diagnostic> {
    let mut found: Vec<Diagnostic> = messages
        .into_iter()
        .map(|message| {
            let (code, detail) = message
                .split_once(": ")
                .unwrap_or(("html.render-warning", message.as_str()));
            let code = if code.starts_with("html.") {
                code.to_owned()
            } else {
                format!("html.{code}")
            };
            let entry = CATALOGUE.iter().find(|entry| entry.code == code);
            let severity = entry.map_or(Severity::Warning, |entry| entry.severity);
            let message = match code.as_str() {
                "html.resource-unavailable" => {
                    let shown = detail.strip_prefix(BASE).unwrap_or(detail);
                    format!(
                        "The page asks for `{shown}`, which the kit cannot load, so it is missing."
                    )
                }
                _ => sentence(detail),
            };
            Diagnostic::new(code, severity, message)
        })
        .collect();
    found.sort_by_key(|diagnostic| diagnostic.severity);
    found
}
