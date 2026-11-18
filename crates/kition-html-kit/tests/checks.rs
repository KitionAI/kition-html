//! `html_check` against its catalogue (`agent/checks.json`, KHP A38): the catalogue file is the
//! check's own (`check::CATALOGUE`), errors first, one sentence each; every fixture in
//! `conformance/` reports its code and only catalogued codes, and every code has one; `basic.html`
//! is clean; checking changes nothing. The fixtures are written
//! by the crate's `conformance` test.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use kition_html_kit::HtmlDocument;
use kition_html_kit::check::{CATALOGUE, catalogue_json};
use kition_kit::KitDocument;
use kition_kit::diagnostics::{Severity, check_diagnostics};
use serde_json::Value;

fn repo() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn open(name: &str) -> HtmlDocument {
    let bytes = std::fs::read(repo().join("conformance").join(name))
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    HtmlDocument::open(name, &bytes).unwrap()
}

#[test]
fn the_catalogue_file_is_the_checks_own() {
    let file: Value =
        serde_json::from_str(&std::fs::read_to_string(repo().join("agent/checks.json")).unwrap())
            .unwrap();
    assert_eq!(
        file,
        catalogue_json(),
        "regenerate with KITION_WRITE_CONFORMANCE=1 cargo test -p kition-html-kit conformance"
    );
    let mut previous = Severity::Error;
    for entry in CATALOGUE {
        assert!(
            kition_kit::diagnostics::valid_code(entry.code) && entry.code.starts_with("html."),
            "{}",
            entry.code
        );
        assert!(entry.severity >= previous, "errors first: {}", entry.code);
        previous = entry.severity;
        assert!(
            entry.summary.ends_with('.') && !entry.summary.contains('\n'),
            "{}: one sentence",
            entry.code
        );
    }
    let without: Vec<&str> = file
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e.get("fixture").is_none())
        .map(|e| e["code"].as_str().unwrap())
        .collect();
    assert_eq!(without, Vec::<&str>::new(), "every code has a fixture");
}

#[test]
fn every_fixture_reports_its_code_and_only_catalogued_codes() {
    let known: Vec<&str> = CATALOGUE.iter().map(|entry| entry.code).collect();
    for entry in CATALOGUE {
        let name = format!("check-{}.html", entry.code.trim_start_matches("html."));
        let found = open(&name).check();
        check_diagnostics(&found).unwrap();
        assert!(
            found.iter().any(|diagnostic| diagnostic.code == entry.code),
            "{name}: {found:?}"
        );
        for diagnostic in &found {
            assert!(
                known.contains(&diagnostic.code.as_str()),
                "{name}: {} is not catalogued",
                diagnostic.code
            );
            assert!(!diagnostic.message.contains('\n'), "{name}: one line");
            assert!(
                !diagnostic.message.contains("kit.invalid"),
                "{name}: the page's own paths, not the renderer's base URL"
            );
        }
    }
}

#[test]
fn basic_is_clean_and_checking_changes_nothing() {
    let page = open("basic.html");
    let before = page.inspect().unwrap();
    assert_eq!(page.check(), Vec::new());
    assert_eq!(page.inspect().unwrap(), before);
    assert!(!page.dirty());
}
